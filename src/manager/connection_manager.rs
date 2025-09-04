//! # Connection Manager Module
//!
//! This module contains the main `KiteTickerManager` which provides high-performance
//! multi-connection WebSocket management for the Kite Connect ticker API.
//!
//! ## Features
//!
//! - **Multi-Connection Support**: Utilizes up to 3 WebSocket connections
//! - **Dynamic Load Balancing**: Automatic symbol distribution across connections
//! - **High-Performance Processing**: Dedicated parser tasks per connection
//! - **Dynamic Subscriptions**: Runtime symbol addition/removal without reconnection
//! - **Health Monitoring**: Real-time connection health tracking
//! - **Error Resilience**: Comprehensive error handling and recovery

use crate::manager::{
  ChannelId, ConnectionStats, HealthMonitor, HealthSummary, KiteManagerConfig,
  ManagedConnection, ManagerStats, MessageProcessor, ProcessorStats,
};
use crate::models::{Mode, TickerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc, RwLock};

/// High-performance multi-connection WebSocket manager for Kite ticker data
///
/// This manager creates 3 independent WebSocket connections and distributes symbols
/// across them using round-robin allocation. Each connection has its own dedicated
/// parser task for maximum performance.
#[derive(Debug)]
pub struct KiteTickerManager {
  /// Manager configuration
  config: KiteManagerConfig,

  /// API credentials
  api_key: String,
  access_token: String,

  /// WebSocket connections (up to 3)
  connections: Vec<ManagedConnection>,

  /// Message processors (one per connection)
  processors: Vec<MessageProcessor>,

  /// Output channels (one per connection)
  output_channels: Vec<broadcast::Receiver<TickerMessage>>,

  /// Symbol to connection mapping
  symbol_mapping: HashMap<u32, ChannelId>,

  /// Health monitor
  health_monitor: Option<HealthMonitor>,

  /// Next connection index for round-robin distribution
  next_connection_index: usize,

  /// Manager start time for uptime tracking
  #[allow(dead_code)]
  start_time: Instant,
  /// If true, underlying connections operate in raw-only mode (no tick parsing)
  raw_only: bool,
}

/// Builder for `KiteTickerManager` providing a fluent API for configuration.
#[derive(Debug, Clone)]
pub struct KiteTickerManagerBuilder {
  api_key: String,
  access_token: String,
  config: KiteManagerConfig,
  raw_only: bool,
}

impl KiteTickerManagerBuilder {
  /// Create a new builder with mandatory credentials and default config
  pub fn new(
    api_key: impl Into<String>,
    access_token: impl Into<String>,
  ) -> Self {
    Self {
      api_key: api_key.into(),
      access_token: access_token.into(),
      config: KiteManagerConfig::default(),
      raw_only: false,
    }
  }

  pub fn max_connections(mut self, n: usize) -> Self {
    self.config.max_connections = n;
    self
  }
  pub fn max_symbols_per_connection(mut self, n: usize) -> Self {
    self.config.max_symbols_per_connection = n;
    self
  }
  pub fn connection_timeout(mut self, d: std::time::Duration) -> Self {
    self.config.connection_timeout = d;
    self
  }
  pub fn health_check_interval(mut self, d: std::time::Duration) -> Self {
    self.config.health_check_interval = d;
    self
  }
  pub fn reconnect_attempts(mut self, attempts: usize) -> Self {
    self.config.max_reconnect_attempts = attempts;
    self
  }
  pub fn reconnect_delay(mut self, d: std::time::Duration) -> Self {
    self.config.reconnect_delay = d;
    self
  }
  pub fn enable_dedicated_parsers(mut self, enable: bool) -> Self {
    self.config.enable_dedicated_parsers = enable;
    self
  }
  pub fn default_mode(mut self, mode: Mode) -> Self {
    self.config.default_mode = mode;
    self
  }
  pub fn heartbeat_liveness_threshold(
    mut self,
    d: std::time::Duration,
  ) -> Self {
    self.config.heartbeat_liveness_threshold = d;
    self
  }
  pub fn connection_buffer_size(mut self, sz: usize) -> Self {
    self.config.connection_buffer_size = sz;
    self
  }
  pub fn parser_buffer_size(mut self, sz: usize) -> Self {
    self.config.parser_buffer_size = sz;
    self
  }
  pub fn raw_only(mut self, raw: bool) -> Self {
    self.raw_only = raw;
    self
  }

  /// Override entire config (advanced)
  pub fn config(mut self, config: KiteManagerConfig) -> Self {
    self.config = config;
    self
  }

  /// Build the manager (not started yet)
  pub fn build(self) -> KiteTickerManager {
    KiteTickerManager::new(self.api_key, self.access_token, self.config)
      .with_raw_only(self.raw_only)
  }
}

impl KiteTickerManager {
  /// Creates a new KiteTickerManager instance with the specified configuration
  ///
  /// This initializes the manager with the provided API credentials and configuration,
  /// but does not start any connections. Call [`start()`](Self::start) to begin operation.
  ///
  /// # Arguments
  ///
  /// * `api_key` - Your Kite Connect API key
  /// * `access_token` - Valid access token from Kite Connect
  /// * `config` - Manager configuration settings
  ///
  /// # Example
  ///
  /// ```rust,no_run
  /// use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode};
  ///
  /// let config = KiteManagerConfig {
  ///     max_connections: 3,
  ///     max_symbols_per_connection: 3000,
  ///     enable_dedicated_parsers: true,
  ///     default_mode: Mode::LTP,
  ///     ..Default::default()
  /// };
  ///
  /// let manager = KiteTickerManager::new(
  ///     "your_api_key".to_string(),
  ///     "your_access_token".to_string(),
  ///     config,
  /// );
  /// ```
  pub fn new(
    api_key: String,
    access_token: String,
    config: KiteManagerConfig,
  ) -> Self {
    Self {
      config,
      api_key,
      access_token,
      connections: Vec::new(),
      processors: Vec::new(),
      output_channels: Vec::new(),
      symbol_mapping: HashMap::new(),
      health_monitor: None,
      next_connection_index: 0,
      start_time: Instant::now(),
      raw_only: false,
    }
  }

  /// Set raw-only mode (builder uses this)
  pub fn with_raw_only(mut self, raw: bool) -> Self {
    self.raw_only = raw;
    self
  }

  /// Initialize all connections and start the manager
  pub async fn start(&mut self) -> Result<(), String> {
    log::info!(
      "Starting KiteTickerManager with {} connections",
      self.config.max_connections
    );

    // Create all connections and processors
    for i in 0..self.config.max_connections {
      let channel_id = ChannelId::from_index(i)
        .ok_or_else(|| format!("Invalid connection index: {}", i))?;

      // Create message channel between connection and processor
      let (connection_sender, processor_receiver) = mpsc::unbounded_channel();

      // Create managed connection
      let mut connection =
        ManagedConnection::new(channel_id, connection_sender);

      // Connect to WebSocket
      if self.raw_only {
        connection
          .connect_with_raw(
            &self.api_key,
            &self.access_token,
            &self.config,
            true,
          )
          .await
          .map_err(|e| format!("Failed to connect WebSocket {}: {}", i, e))?;
      } else {
        connection
          .connect(&self.api_key, &self.access_token, &self.config)
          .await
          .map_err(|e| format!("Failed to connect WebSocket {}: {}", i, e))?;
      }

      // Create message processor
      let (mut processor, output_receiver) = MessageProcessor::new(
        channel_id,
        processor_receiver,
        self.config.parser_buffer_size,
      );

      // Start processor if enabled
      if self.config.enable_dedicated_parsers {
        processor.start();
        log::info!("Started dedicated parser for connection {}", i);
      }

      self.connections.push(connection);
      self.processors.push(processor);
      self.output_channels.push(output_receiver);
    }

    // Start health monitoring
    if self.config.health_check_interval.as_secs() > 0 {
      let connection_stats: Vec<Arc<RwLock<ConnectionStats>>> = self
        .connections
        .iter()
        .map(|c| Arc::clone(&c.stats))
        .collect();

      let mut health_monitor =
        HealthMonitor::new(connection_stats, self.config.health_check_interval);
      health_monitor.start();
      self.health_monitor = Some(health_monitor);

      log::info!("Started health monitor");
    }

    log::info!(
      "KiteTickerManager started successfully with {} connections",
      self.connections.len()
    );

    Ok(())
  }

  /// Subscribe to symbols using round-robin distribution
  pub async fn subscribe_symbols(
    &mut self,
    symbols: &[u32],
    mode: Option<Mode>,
  ) -> Result<(), String> {
    let mode = mode.unwrap_or(self.config.default_mode);

    log::info!(
      "Subscribing to {} symbols with mode: {:?}",
      symbols.len(),
      mode
    );

    // Group symbols by connection using round-robin
    let mut connection_symbols: HashMap<ChannelId, Vec<u32>> = HashMap::new();

    for &symbol in symbols {
      // Skip if already subscribed
      if self.symbol_mapping.contains_key(&symbol) {
        log::debug!("Symbol {} already subscribed", symbol);
        continue;
      }

      // Find connection with available capacity
      let connection_id = self.find_available_connection()?;

      // Add to mapping
      self.symbol_mapping.insert(symbol, connection_id);
      connection_symbols
        .entry(connection_id)
        .or_default()
        .push(symbol);
    }

    // Subscribe symbols on each connection
    for (connection_id, symbols) in connection_symbols {
      let connection = &mut self.connections[connection_id.to_index()];
      let mode_clone = mode; // Clone for each connection

      if !symbols.is_empty() {
        // Use dynamic subscription if already has symbols, otherwise initial setup
        if connection.subscribed_symbols.is_empty() {
          // First-time subscription on this connection: create subscriber
          connection
            .subscribe_symbols(&symbols, mode_clone)
            .await
            .map_err(|e| {
              format!(
                "Failed to subscribe on connection {:?}: {}",
                connection_id, e
              )
            })?;

          // IMPORTANT: Start forwarding messages from the subscriber to the processor
          connection.start_message_processing().await.map_err(|e| {
            format!(
              "Failed to start message processing on connection {:?}: {}",
              connection_id, e
            )
          })?;
        } else {
          connection
            .add_symbols(&symbols, mode_clone)
            .await
            .map_err(|e| {
              format!(
                "Failed to add symbols on connection {:?}: {}",
                connection_id, e
              )
            })?;
        }

        log::info!(
          "Subscribed {} symbols on connection {:?}",
          symbols.len(),
          connection_id
        );
      }
    }

    log::info!("Successfully subscribed to {} new symbols", symbols.len());
    Ok(())
  }

  /// Find connection with available capacity using round-robin
  fn find_available_connection(&mut self) -> Result<ChannelId, String> {
    let _start_index = self.next_connection_index;

    // Try round-robin allocation
    for _ in 0..self.config.max_connections {
      let connection = &self.connections[self.next_connection_index];

      if connection
        .can_accept_symbols(1, self.config.max_symbols_per_connection)
      {
        let channel_id = connection.id;
        self.next_connection_index =
          (self.next_connection_index + 1) % self.config.max_connections;
        return Ok(channel_id);
      }

      self.next_connection_index =
        (self.next_connection_index + 1) % self.config.max_connections;
    }

    Err("All connections are at capacity".to_string())
  }

  /// Get output channel for a specific connection
  pub fn get_channel(
    &mut self,
    channel_id: ChannelId,
  ) -> Option<broadcast::Receiver<TickerMessage>> {
    if channel_id.to_index() < self.output_channels.len() {
      Some(self.output_channels[channel_id.to_index()].resubscribe())
    } else {
      None
    }
  }

  /// Get all output channels
  pub fn get_all_channels(
    &mut self,
  ) -> Vec<(ChannelId, broadcast::Receiver<TickerMessage>)> {
    let mut channels = Vec::new();

    for (i, channel) in self.output_channels.iter().enumerate() {
      if let Some(channel_id) = ChannelId::from_index(i) {
        channels.push((channel_id, channel.resubscribe()));
      }
    }

    channels
  }

  /// Get a raw frame receiver (bytes::Bytes per websocket frame) for a connection.
  /// Returns None if the connection is not initialized.
  pub fn get_raw_frame_channel(
    &self,
    channel_id: ChannelId,
  ) -> Option<tokio::sync::broadcast::Receiver<bytes::Bytes>> {
    self
      .connections
      .get(channel_id.to_index())
      .and_then(|mc| mc.ticker.as_ref())
      .map(|t| t.subscribe_raw_frames())
  }

  /// Convenience: create a 184-byte Full-tick raw subscriber for a connection.
  /// Useful when operating in raw_only mode to consume only depth packets.
  pub fn get_full_raw_subscriber(
    &self,
    channel_id: ChannelId,
  ) -> Option<crate::KiteTickerRawSubscriber184> {
    self
      .connections
      .get(channel_id.to_index())
      .and_then(|mc| mc.ticker.as_ref())
      .map(|t| t.subscribe_full_raw())
  }

  /// Convenience: get raw frame receivers for all initialized connections.
  /// Each item is `(ChannelId, broadcast::Receiver<bytes::Bytes>)`.
  pub fn get_all_raw_frame_channels(
    &self,
  ) -> Vec<(ChannelId, tokio::sync::broadcast::Receiver<bytes::Bytes>)> {
    let mut out = Vec::with_capacity(self.connections.len());
    for (i, mc) in self.connections.iter().enumerate() {
      if let Some(ch) = ChannelId::from_index(i) {
        if let Some(t) = mc.ticker.as_ref() {
          out.push((ch, t.subscribe_raw_frames()));
        }
      }
    }
    out
  }

  /// Get manager statistics
  pub async fn get_stats(&self) -> Result<ManagerStats, String> {
    if let Some(health_monitor) = &self.health_monitor {
      Ok(health_monitor.get_manager_stats().await)
    } else {
      Err("Health monitor not available".to_string())
    }
  }

  /// Get health summary
  pub async fn get_health(&self) -> Result<HealthSummary, String> {
    if let Some(health_monitor) = &self.health_monitor {
      Ok(health_monitor.get_health_summary().await)
    } else {
      Err("Health monitor not available".to_string())
    }
  }

  /// Get processor statistics for all channels
  pub async fn get_processor_stats(&self) -> Vec<(ChannelId, ProcessorStats)> {
    let mut stats = Vec::new();

    for processor in &self.processors {
      let processor_stats = processor.get_stats().await;
      stats.push((processor.channel_id, processor_stats));
    }

    stats
  }

  /// Get symbol distribution across connections
  pub fn get_symbol_distribution(&self) -> HashMap<ChannelId, Vec<u32>> {
    let mut distribution: HashMap<ChannelId, Vec<u32>> = HashMap::new();

    for (&symbol, &channel_id) in &self.symbol_mapping {
      distribution.entry(channel_id).or_default().push(symbol);
    }

    distribution
  }

  /// Unsubscribe from symbols
  pub async fn unsubscribe_symbols(
    &mut self,
    symbols: &[u32],
  ) -> Result<(), String> {
    log::info!("Unsubscribing from {} symbols", symbols.len());

    // Group symbols by connection
    let mut connection_symbols: HashMap<ChannelId, Vec<u32>> = HashMap::new();

    for &symbol in symbols {
      if let Some(&channel_id) = self.symbol_mapping.get(&symbol) {
        connection_symbols
          .entry(channel_id)
          .or_default()
          .push(symbol);
        self.symbol_mapping.remove(&symbol);
      } else {
        log::debug!("Symbol {} not found in subscriptions", symbol);
      }
    }

    // Unsubscribe from each connection
    for (channel_id, symbols) in connection_symbols {
      let connection = &mut self.connections[channel_id.to_index()];

      if !symbols.is_empty() {
        connection.remove_symbols(&symbols).await.map_err(|e| {
          format!(
            "Failed to unsubscribe from connection {:?}: {}",
            channel_id, e
          )
        })?;

        log::info!(
          "Unsubscribed {} symbols from connection {:?}",
          symbols.len(),
          channel_id
        );
      }
    }

    log::info!("Successfully unsubscribed from {} symbols", symbols.len());
    Ok(())
  }

  /// Dynamically change subscription mode for existing symbols
  pub async fn change_mode(
    &mut self,
    symbols: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    log::info!("Changing mode for {} symbols to {:?}", symbols.len(), mode);

    // Group symbols by connection
    let mut connection_symbols: HashMap<ChannelId, Vec<u32>> = HashMap::new();

    for &symbol in symbols {
      if let Some(&channel_id) = self.symbol_mapping.get(&symbol) {
        connection_symbols
          .entry(channel_id)
          .or_default()
          .push(symbol);
      } else {
        log::debug!("Symbol {} not found in subscriptions", symbol);
      }
    }

    // Change mode on each connection
    for (channel_id, symbols) in connection_symbols {
      let connection = &mut self.connections[channel_id.to_index()];
      if symbols.is_empty() {
        continue;
      }
      // Send mode request directly via command sender if available
      if let Some(ref cmd) = connection.cmd_tx {
        let mode_req = crate::models::Request::mode(mode, &symbols).to_string();
        let _ = cmd.send(tokio_tungstenite::tungstenite::Message::Text(
          mode_req.into(),
        ));
        for &s in &symbols {
          connection.subscribed_symbols.insert(s, mode);
        }
        log::info!(
          "Changed mode for {} symbols on connection {:?}",
          symbols.len(),
          channel_id
        );
      } else if let Some(subscriber) = &mut connection.subscriber {
        // fallback (should normally have command sender)
        subscriber.set_mode(&symbols, mode).await.map_err(|e| {
          format!(
            "Failed to change mode on connection {:?}: {}",
            channel_id, e
          )
        })?;
        for &s in &symbols {
          connection.subscribed_symbols.insert(s, mode);
        }
      }
    }

    log::info!("Successfully changed mode for {} symbols", symbols.len());
    Ok(())
  }

  /// Stop the manager and all connections
  pub async fn stop(&mut self) -> Result<(), String> {
    log::info!("Stopping KiteTickerManager");

    // Stop health monitor
    if let Some(health_monitor) = &mut self.health_monitor {
      health_monitor.stop().await;
    }

    // Stop all processors
    for processor in &mut self.processors {
      processor.stop().await;
    }

    // Stop all connections
    for connection in &mut self.connections {
      if let Some(h) = connection.heartbeat_handle.take() {
        h.abort();
        let _ = h.await;
      }
      if let Some(handle) = connection.task_handle.take() {
        handle.abort();
        let _ = handle.await;
      }
    }

    log::info!("KiteTickerManager stopped");
    Ok(())
  }
}
