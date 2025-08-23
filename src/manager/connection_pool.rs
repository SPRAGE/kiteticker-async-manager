use crate::manager::{ChannelId, ConnectionStats, KiteManagerConfig};
use crate::models::{Mode, TickerMessage};
use crate::ticker::KiteTickerAsync;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;

/// Represents a single WebSocket connection with its metadata
#[derive(Debug)]
pub struct ManagedConnection {
  pub id: ChannelId,
  pub ticker: Option<KiteTickerAsync>,
  pub subscriber: Option<crate::ticker::KiteTickerSubscriber>,
  pub subscribed_symbols: HashMap<u32, Mode>,
  pub stats: Arc<RwLock<ConnectionStats>>,
  pub is_healthy: Arc<AtomicBool>,
  pub last_ping: Arc<AtomicU64>, // Unix timestamp
  pub task_handle: Option<JoinHandle<()>>,
  pub message_sender: mpsc::UnboundedSender<TickerMessage>,
  // Store credentials for dynamic operations
  api_key: String,
  access_token: String,
  pub(crate) cmd_tx: Option<mpsc::UnboundedSender<tokio_tungstenite::tungstenite::Message>>,
}

impl ManagedConnection {
  pub fn new(
    id: ChannelId,
    message_sender: mpsc::UnboundedSender<TickerMessage>,
  ) -> Self {
    let stats = ConnectionStats {
      connection_id: id.to_index(),
      ..Default::default()
    };

    Self {
      id,
      ticker: None,
      subscriber: None,
      subscribed_symbols: HashMap::new(),
      stats: Arc::new(RwLock::new(stats)),
      is_healthy: Arc::new(AtomicBool::new(false)),
      last_ping: Arc::new(AtomicU64::new(0)),
      task_handle: None,
      message_sender,
      api_key: String::new(),
      access_token: String::new(),
  cmd_tx: None,
    }
  }

  /// Connect to WebSocket and start message processing
  pub async fn connect(
    &mut self,
    api_key: &str,
    access_token: &str,
    config: &KiteManagerConfig,
  ) -> Result<(), String> {
    // Store credentials for dynamic operations
    self.api_key = api_key.to_string();
    self.access_token = access_token.to_string();

    // Connect to WebSocket
    let ticker = timeout(
      config.connection_timeout,
      KiteTickerAsync::connect(api_key, access_token),
    )
    .await
    .map_err(|_| "Connection timeout".to_string())?
    .map_err(|e| format!("Connection failed: {}", e))?;

  self.cmd_tx = ticker.command_sender();
  self.ticker = Some(ticker);
    self.is_healthy.store(true, Ordering::Relaxed);

    // Update stats
    {
      let mut stats = self.stats.write().await;
      stats.is_connected = true;
      stats.connection_uptime = Duration::ZERO;
    }

    Ok(())
  }

  /// Connect with explicit raw_only flag
  pub async fn connect_with_raw(
    &mut self,
    api_key: &str,
    access_token: &str,
    config: &KiteManagerConfig,
    raw_only: bool,
  ) -> Result<(), String> {
    self.api_key = api_key.to_string();
    self.access_token = access_token.to_string();
    let ticker = tokio::time::timeout(
      config.connection_timeout,
      crate::ticker::KiteTickerAsync::connect_with_options(api_key, access_token, raw_only),
    )
    .await
    .map_err(|_| "Connection timeout".to_string())?
    .map_err(|e| format!("Connection failed: {}", e))?;

    self.cmd_tx = ticker.command_sender();
    self.ticker = Some(ticker);
    self.is_healthy.store(true, std::sync::atomic::Ordering::Relaxed);
    {
      let mut stats = self.stats.write().await;
      stats.is_connected = true;
      stats.connection_uptime = Duration::ZERO;
    }
    Ok(())
  }

  /// Subscribe to symbols on this connection
  pub async fn subscribe_symbols(
    &mut self,
    symbols: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    if let Some(ticker) = self.ticker.as_mut() {
      // Use the existing ticker directly
      let subscriber = ticker.subscribe(symbols, Some(mode)).await?;
      // Track symbols
      for &symbol in symbols { self.subscribed_symbols.insert(symbol, mode); }
      self.subscriber = Some(subscriber);

      // Update stats
      {
        let mut stats = self.stats.write().await;
        stats.symbol_count = self.subscribed_symbols.len();
      }

      Ok(())
    } else {
      Err("Connection not established".to_string())
    }
  }

  /// Dynamically add new symbols to existing subscription
  pub async fn add_symbols(
    &mut self,
    symbols: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    if self.subscriber.is_some() {
      // Filter to truly new symbols
      let new: Vec<u32> = symbols.iter().copied().filter(|s| !self.subscribed_symbols.contains_key(s)).collect();
      if new.is_empty() { return Ok(()); }
      if let Some(tx) = &self.cmd_tx {
        // send subscribe + mode
        let sub = crate::models::Request::subscribe(&new).to_string();
        let mode_msg = crate::models::Request::mode(mode, &new).to_string();
        let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(sub));
        let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(mode_msg));
      }
      for &s in &new { self.subscribed_symbols.insert(s, mode); }
      let mut stats = self.stats.write().await; stats.symbol_count = self.subscribed_symbols.len();
      log::info!("Incrementally subscribed {} symbols on connection {}", new.len(), self.id.to_index());
      Ok(())
    } else { self.subscribe_symbols(symbols, mode).await }
  }

  /// Dynamically remove symbols from existing subscription
  pub async fn remove_symbols(
    &mut self,
    symbols: &[u32],
  ) -> Result<(), String> {
    if self.subscriber.is_some() {
      // Only symbols currently subscribed
      let existing: Vec<u32> = symbols.iter().copied().filter(|s| self.subscribed_symbols.contains_key(s)).collect();
      if existing.is_empty() { return Ok(()); }
      if let Some(tx) = &self.cmd_tx {
        let unsub = crate::models::Request::unsubscribe(&existing).to_string();
        let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(unsub));
      }
      for s in &existing { self.subscribed_symbols.remove(s); }
      let mut stats = self.stats.write().await; stats.symbol_count = self.subscribed_symbols.len();
      log::info!("Incrementally unsubscribed {} symbols on connection {}", existing.len(), self.id.to_index());
      Ok(())
    } else { Err("No active subscription to remove symbols from".to_string()) }
  }

  /// Start message processing for the subscriber
  pub async fn start_message_processing(&mut self) -> Result<(), String> {
    if let Some(subscriber) = self.subscriber.take() {
      let message_sender = self.message_sender.clone();
      let stats = Arc::clone(&self.stats);
      let is_healthy = Arc::clone(&self.is_healthy);
      let connection_id = self.id;

      let handle = tokio::spawn(async move {
        Self::message_processing_loop(
          subscriber,
          message_sender,
          stats,
          is_healthy,
          connection_id,
        )
        .await;
      });

      self.task_handle = Some(handle);
      Ok(())
    } else {
      Err("No subscriber available for message processing".to_string())
    }
  }

  /// Message processing loop for this connection
  async fn message_processing_loop(
    mut subscriber: crate::ticker::KiteTickerSubscriber,
    message_sender: mpsc::UnboundedSender<TickerMessage>,
    stats: Arc<RwLock<ConnectionStats>>,
    is_healthy: Arc<AtomicBool>,
    connection_id: ChannelId,
  ) {
  let mut last_message_time = Instant::now();
  let mut last_stats_flush = Instant::now();
  let mut pending_messages: u64 = 0;

    log::info!(
      "Starting message processing loop for connection {}",
      connection_id.to_index()
    );

    loop {
      match timeout(Duration::from_secs(30), subscriber.next_message()).await {
        Ok(Ok(Some(message))) => {
          last_message_time = Instant::now();

          // Debug: Print incoming message
          if log::log_enabled!(log::Level::Debug) {
            match &message {
              TickerMessage::Ticks(ticks) => {
                log::debug!(
                  "Connection {}: Received {} ticks",
                  connection_id.to_index(),
                  ticks.len()
                );
                for (i, tick) in ticks.iter().take(3).enumerate() {
                  log::debug!(
                    "  Tick {}: Symbol {}, Mode {:?}, LTP {:?}",
                    i + 1,
                    tick.instrument_token,
                    tick.content.mode,
                    tick.content.last_price
                  );
                }
              }
              TickerMessage::Error(err) => {
                log::debug!(
                  "Connection {}: Received error message: {}",
                  connection_id.to_index(),
                  err
                );
              }
              _ => {
                log::debug!(
                  "Connection {}: Received other message: {:?}",
                  connection_id.to_index(),
                  message
                );
              }
            }
          }

          // Update stats
          pending_messages += 1;
          if last_stats_flush.elapsed() >= Duration::from_millis(1000) {
            let mut stats = stats.write().await;
            stats.messages_received += pending_messages;
            stats.last_message_time = Some(last_message_time);
            pending_messages = 0;
            last_stats_flush = Instant::now();
          }

          // Forward message to parser (non-blocking)
          if message_sender.send(message).is_err() {
            log::warn!(
              "Connection {}: Parser channel full, dropping message",
              connection_id.to_index()
            );

            // Update error stats
            let mut stats = stats.write().await;
            stats.errors_count += 1;
          }
        }
        Ok(Ok(None)) => {
          log::info!("Connection {} closed", connection_id.to_index());
          is_healthy.store(false, Ordering::Relaxed);
          break;
        }
        Ok(Err(e)) => {
          log::error!("Connection {} error: {}", connection_id.to_index(), e);

          // Update error stats
          if last_stats_flush.elapsed() >= Duration::from_millis(250) {
            let mut stats = stats.write().await;
            stats.errors_count += 1;
            last_stats_flush = Instant::now();
          }

          // Continue trying to receive messages
        }
        Err(_) => {
          // Timeout - check if connection is still alive
          if last_message_time.elapsed() > Duration::from_secs(60) {
            log::warn!(
              "Connection {} timeout - no messages for 60s",
              connection_id.to_index()
            );
            is_healthy.store(false, Ordering::Relaxed);
            break;
          }
        }
      }
    }

    // Update connection status
    {
      let mut stats = stats.write().await;
      stats.is_connected = false;
    }
    is_healthy.store(false, Ordering::Relaxed);
  }

  /// Check if connection can accept more symbols
  pub fn can_accept_symbols(
    &self,
    count: usize,
    max_per_connection: usize,
  ) -> bool {
    self.subscribed_symbols.len() + count <= max_per_connection
  }

  /// Get current symbol count
  pub fn symbol_count(&self) -> usize {
    self.subscribed_symbols.len()
  }

  /// Check if connection is healthy
  pub fn is_healthy(&self) -> bool {
    self.is_healthy.load(Ordering::Relaxed)
  }
}
