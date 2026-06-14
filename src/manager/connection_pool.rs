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
  // Background task that sends periodic keepalive WS pings (see start_ping_task).
  pub heartbeat_handle: Option<JoinHandle<()>>,
  pub message_sender: mpsc::UnboundedSender<TickerMessage>,
  // Store credentials for dynamic operations
  api_key: String,
  access_token: String,
  pub(crate) cmd_tx:
    Option<mpsc::UnboundedSender<tokio_tungstenite::tungstenite::Message>>,
  // Liveness threshold for heartbeats/frames
  heartbeat_liveness_threshold: Duration,
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
      heartbeat_handle: None,
      message_sender,
      api_key: String::new(),
      access_token: String::new(),
      cmd_tx: None,
      heartbeat_liveness_threshold: Duration::from_secs(10),
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
    // Liveness now follows the ticker's all-frame activity clock, refreshed by
    // ANY inbound frame (including the Pong from our keepalive ping), so an idle
    // market connection stays healthy instead of false-timing-out.
    self.last_ping = ticker.last_activity();
    self.ticker = Some(ticker);
    // Set the configured liveness threshold, then start the keepalive ping task
    // (its interval is derived from the threshold).
    self.heartbeat_liveness_threshold = config.heartbeat_liveness_threshold;
    self.start_ping_task();
    self.is_healthy.store(true, Ordering::Relaxed);

    // Update stats
    {
      let mut stats = self.stats.write().await;
      stats.is_connected = true;
      stats.connection_uptime = Duration::ZERO;
    }

    Ok(())
  }

  /// Start a background task that sends a WebSocket Ping at a fixed interval to
  /// keep the connection warm when the market is idle. Kite replies with a Pong,
  /// which (like any inbound frame) refreshes the ticker's activity clock and
  /// stops the liveness check from falsely timing the connection out. The
  /// interval is half the liveness threshold (min 2s) so a Pong always lands
  /// well within the window.
  fn start_ping_task(&mut self) {
    // Drop existing task if any
    if let Some(h) = self.heartbeat_handle.take() {
      h.abort();
    }
    let Some(cmd_tx) = self.cmd_tx.clone() else {
      return;
    };
    let interval =
      (self.heartbeat_liveness_threshold / 2).max(Duration::from_secs(2));
    let id = self.id;
    let handle = tokio::spawn(async move {
      let mut tick = tokio::time::interval(interval);
      tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
      // Skip the immediate first tick — the socket was just opened.
      tick.tick().await;
      loop {
        tick.tick().await;
        if cmd_tx
          .send(tokio_tungstenite::tungstenite::Message::Ping(
            Default::default(),
          ))
          .is_err()
        {
          log::debug!(
            "Ping task: connection {} command channel closed",
            id.to_index()
          );
          break;
        }
      }
    });
    self.heartbeat_handle = Some(handle);
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
      crate::ticker::KiteTickerAsync::connect_with_options(
        api_key,
        access_token,
        raw_only,
      ),
    )
    .await
    .map_err(|_| "Connection timeout".to_string())?
    .map_err(|e| format!("Connection failed: {}", e))?;

    self.cmd_tx = ticker.command_sender();
    // Liveness follows the ticker's all-frame activity clock (refreshed by ANY
    // inbound frame, including the Pong from our keepalive ping).
    self.last_ping = ticker.last_activity();
    self.ticker = Some(ticker);
    // Set the configured liveness threshold, then start the keepalive ping task
    // (its interval is derived from the threshold).
    self.heartbeat_liveness_threshold = config.heartbeat_liveness_threshold;
    self.start_ping_task();
    self
      .is_healthy
      .store(true, std::sync::atomic::Ordering::Relaxed);
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
      for &symbol in symbols {
        self.subscribed_symbols.insert(symbol, mode);
      }
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
      let new: Vec<u32> = symbols
        .iter()
        .copied()
        .filter(|s| !self.subscribed_symbols.contains_key(s))
        .collect();
      if new.is_empty() {
        return Ok(());
      }
      if let Some(tx) = &self.cmd_tx {
        // send subscribe + mode
        let sub = crate::models::Request::subscribe(&new).to_string();
        let mode_msg = crate::models::Request::mode(mode, &new).to_string();
        let _ =
          tx.send(tokio_tungstenite::tungstenite::Message::Text(sub.into()));
        let _ = tx.send(tokio_tungstenite::tungstenite::Message::Text(
          mode_msg.into(),
        ));
      }
      for &s in &new {
        self.subscribed_symbols.insert(s, mode);
      }
      let mut stats = self.stats.write().await;
      stats.symbol_count = self.subscribed_symbols.len();
      log::info!(
        "Incrementally subscribed {} symbols on connection {}",
        new.len(),
        self.id.to_index()
      );
      Ok(())
    } else {
      self.subscribe_symbols(symbols, mode).await
    }
  }

  /// Dynamically remove symbols from existing subscription
  pub async fn remove_symbols(
    &mut self,
    symbols: &[u32],
  ) -> Result<(), String> {
    if self.subscriber.is_some() {
      // Only symbols currently subscribed
      let existing: Vec<u32> = symbols
        .iter()
        .copied()
        .filter(|s| self.subscribed_symbols.contains_key(s))
        .collect();
      if existing.is_empty() {
        return Ok(());
      }
      if let Some(tx) = &self.cmd_tx {
        let unsub = crate::models::Request::unsubscribe(&existing).to_string();
        let _ =
          tx.send(tokio_tungstenite::tungstenite::Message::Text(unsub.into()));
      }
      for s in &existing {
        self.subscribed_symbols.remove(s);
      }
      let mut stats = self.stats.write().await;
      stats.symbol_count = self.subscribed_symbols.len();
      log::info!(
        "Incrementally unsubscribed {} symbols on connection {}",
        existing.len(),
        self.id.to_index()
      );
      Ok(())
    } else {
      Err("No active subscription to remove symbols from".to_string())
    }
  }

  /// Start message processing for the subscriber
  pub async fn start_message_processing(&mut self) -> Result<(), String> {
    if let Some(subscriber) = self.subscriber.take() {
      let message_sender = self.message_sender.clone();
      let stats = Arc::clone(&self.stats);
      let is_healthy = Arc::clone(&self.is_healthy);
      let last_ping = Arc::clone(&self.last_ping);
      let connection_id = self.id;
      let threshold = self.heartbeat_liveness_threshold;

      let handle = tokio::spawn(async move {
        Self::message_processing_loop(
          subscriber,
          message_sender,
          stats,
          is_healthy,
          connection_id,
          last_ping,
          threshold,
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
    last_ping: Arc<AtomicU64>,
    heartbeat_threshold: Duration,
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
          // Timeout waiting for parsed messages; consult heartbeat/frames
          let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
          let last = last_ping.load(Ordering::Relaxed);
          // If we've seen any frame within threshold, consider connection alive
          if last > 0
            && now_sec.saturating_sub(last) <= heartbeat_threshold.as_secs()
          {
            continue;
          }
          // Fallback to parsed message timer if heartbeat missed
          if last_message_time.elapsed() > heartbeat_threshold {
            log::warn!(
              "Connection {} timeout - no frames/heartbeats within {:?}",
              connection_id.to_index(),
              heartbeat_threshold,
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
