//! # Multi-API Manager Module
//!
//! This module provides the `MultiApiKiteTickerManager` which extends the single-API
//! manager to support multiple Kite Connect API credentials simultaneously.
//!
//! ## Features
//!
//! - **Multiple API Keys**: Manage multiple Kite Connect accounts in a single manager
//! - **Per-API Connection Pools**: Each API key can have up to 3 WebSocket connections
//! - **Flexible Symbol Distribution**: Round-robin or manual assignment of symbols
//! - **Unified Message Stream**: All messages available through unified channels
//! - **Aggregate Monitoring**: Health and statistics across all API keys
//! - **Backward Compatible**: Works alongside existing single-API manager

use crate::manager::{
  ApiCredentials, ApiKeyId, ApiKeyStats, ChannelId,
  DistributionStrategy, KiteManagerConfig, ManagedConnection, MessageProcessor,
  MultiApiConfig, MultiApiStats,
};
use crate::models::{Mode, TickerMessage};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc};

/// Connection group for a single API key
#[derive(Debug)]
struct ApiConnectionGroup {
  api_key_id: ApiKeyId,
  credentials: ApiCredentials,
  connections: Vec<ManagedConnection>,
  processors: Vec<MessageProcessor>,
  subscribed_symbols: HashMap<u32, (usize, Mode)>, // symbol -> (connection_index, mode)
  next_connection_index: usize,
}

impl ApiConnectionGroup {
  fn new(api_key_id: ApiKeyId, credentials: ApiCredentials) -> Self {
    Self {
      api_key_id,
      credentials,
      connections: Vec::new(),
      processors: Vec::new(),
      subscribed_symbols: HashMap::new(),
      next_connection_index: 0,
    }
  }

  /// Find connection with available capacity using round-robin
  fn find_available_connection(
    &mut self,
    max_symbols_per_connection: usize,
  ) -> Option<usize> {
    let start_index = self.next_connection_index;

    for _ in 0..self.connections.len() {
      let connection = &self.connections[self.next_connection_index];

      if connection.can_accept_symbols(1, max_symbols_per_connection) {
        let result = self.next_connection_index;
        self.next_connection_index =
          (self.next_connection_index + 1) % self.connections.len();
        return Some(result);
      }

      self.next_connection_index =
        (self.next_connection_index + 1) % self.connections.len();
    }

    // Reset to start position if no connection found
    self.next_connection_index = start_index;
    None
  }

  /// Get total number of subscribed symbols
  fn total_symbols(&self) -> usize {
    self.subscribed_symbols.len()
  }

  /// Get statistics for this API key
  async fn get_stats(&self) -> ApiKeyStats {
    let mut stats = ApiKeyStats {
      api_key_id: self.api_key_id.0.clone(),
      active_connections: 0,
      total_symbols: self.total_symbols(),
      total_messages_received: 0,
      total_messages_parsed: 0,
      total_errors: 0,
      connection_stats: Vec::new(),
    };

    for connection in &self.connections {
      let conn_stats = connection.stats.read().await;
      stats.connection_stats.push(conn_stats.clone());
      
      if conn_stats.is_connected {
        stats.active_connections += 1;
      }
      
      stats.total_messages_received += conn_stats.messages_received;
      stats.total_messages_parsed += conn_stats.messages_parsed;
      stats.total_errors += conn_stats.errors_count;
    }

    stats
  }
}

/// High-performance multi-API WebSocket manager for Kite ticker data
///
/// This manager supports multiple Kite Connect API credentials, allowing you to
/// subscribe to symbols across multiple accounts. Each API key maintains its own
/// connection pool (up to 3 connections), and symbols can be distributed automatically
/// or manually assigned to specific API keys.
///
/// # Example
///
/// ```rust,no_run
/// use kiteticker_async_manager::{MultiApiKiteTickerManager, Mode};
///
/// #[tokio::main]
/// async fn main() -> Result<(), String> {
///     let mut manager = MultiApiKiteTickerManager::builder()
///         .add_api_key("account1", "api_key_1", "access_token_1")
///         .add_api_key("account2", "api_key_2", "access_token_2")
///         .max_connections_per_api(3)
///         .build();
///
///     manager.start().await?;
///
///     // Subscribe symbols (auto-distributed across API keys)
///     let symbols = vec![408065, 738561];
///     manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
///
///     // Or assign to specific API key
///     manager.subscribe_symbols_to_api("account1", &symbols, Some(Mode::LTP)).await?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct MultiApiKiteTickerManager {
  config: MultiApiConfig,
  api_groups: HashMap<ApiKeyId, ApiConnectionGroup>,
  
  // Unified output channel across all API keys
  unified_output_tx: broadcast::Sender<(ApiKeyId, TickerMessage)>,
  
  // Global symbol mapping: symbol -> API key
  symbol_to_api: HashMap<u32, ApiKeyId>,
  
  // Round-robin index for API key selection
  next_api_index: usize,
  api_key_order: Vec<ApiKeyId>, // For consistent round-robin
  
  start_time: Instant,
}

/// Builder for `MultiApiKiteTickerManager` providing a fluent API for configuration.
#[derive(Debug, Clone)]
pub struct MultiApiKiteTickerManagerBuilder {
  api_credentials: HashMap<ApiKeyId, ApiCredentials>,
  config: MultiApiConfig,
}

impl MultiApiKiteTickerManagerBuilder {
  /// Create a new builder with default configuration
  pub fn new() -> Self {
    Self {
      api_credentials: HashMap::new(),
      config: MultiApiConfig::default(),
    }
  }

  /// Add an API key with credentials
  pub fn add_api_key(
    mut self,
    id: impl Into<ApiKeyId>,
    api_key: impl Into<String>,
    access_token: impl Into<String>,
  ) -> Self {
    let api_key_id = id.into();
    let credentials = ApiCredentials::new(api_key, access_token);
    self.api_credentials.insert(api_key_id, credentials);
    self
  }

  /// Set maximum connections per API key (default: 3)
  pub fn max_connections_per_api(mut self, n: usize) -> Self {
    self.config.max_connections_per_api = n.min(3); // Kite limit
    self
  }

  /// Set symbol distribution strategy
  pub fn distribution_strategy(mut self, strategy: DistributionStrategy) -> Self {
    self.config.distribution_strategy = strategy;
    self
  }

  /// Set base configuration for connections
  pub fn base_config(mut self, config: KiteManagerConfig) -> Self {
    self.config.base_config = config;
    self
  }

  /// Set maximum symbols per connection
  pub fn max_symbols_per_connection(mut self, n: usize) -> Self {
    self.config.base_config.max_symbols_per_connection = n;
    self
  }

  /// Set connection timeout
  pub fn connection_timeout(mut self, d: std::time::Duration) -> Self {
    self.config.base_config.connection_timeout = d;
    self
  }

  /// Set health check interval
  pub fn health_check_interval(mut self, d: std::time::Duration) -> Self {
    self.config.base_config.health_check_interval = d;
    self
  }

  /// Enable or disable health monitoring
  pub fn enable_health_monitoring(mut self, enable: bool) -> Self {
    self.config.enable_health_monitoring = enable;
    self
  }

  /// Set default subscription mode
  pub fn default_mode(mut self, mode: Mode) -> Self {
    self.config.base_config.default_mode = mode;
    self
  }

  /// Build the multi-API manager
  pub fn build(self) -> MultiApiKiteTickerManager {
    MultiApiKiteTickerManager::new(self.api_credentials, self.config)
  }
}

impl Default for MultiApiKiteTickerManagerBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl MultiApiKiteTickerManager {
  /// Create a builder for constructing a multi-API manager
  pub fn builder() -> MultiApiKiteTickerManagerBuilder {
    MultiApiKiteTickerManagerBuilder::new()
  }

  /// Create a new multi-API manager with the specified credentials and configuration
  fn new(
    api_credentials: HashMap<ApiKeyId, ApiCredentials>,
    config: MultiApiConfig,
  ) -> Self {
    let (unified_output_tx, _) =
      broadcast::channel(config.base_config.parser_buffer_size);

    let api_key_order: Vec<ApiKeyId> = api_credentials.keys().cloned().collect();

    let mut api_groups = HashMap::new();
    for (id, creds) in api_credentials {
      api_groups.insert(id.clone(), ApiConnectionGroup::new(id, creds));
    }

    Self {
      config,
      api_groups,
      unified_output_tx,
      symbol_to_api: HashMap::new(),
      next_api_index: 0,
      api_key_order,
      start_time: Instant::now(),
    }
  }

  /// Initialize all connections for all API keys and start the manager
  pub async fn start(&mut self) -> Result<(), String> {
    if self.api_groups.is_empty() {
      return Err("No API keys configured".to_string());
    }

    log::info!(
      "Starting MultiApiKiteTickerManager with {} API keys",
      self.api_groups.len()
    );

    // Clone the unified output sender before iterating
    let unified_tx = self.unified_output_tx.clone();

    for (api_key_id, group) in &mut self.api_groups {
      log::info!(
        "Initializing {} connections for API key: {}",
        self.config.max_connections_per_api,
        api_key_id.0
      );

      for i in 0..self.config.max_connections_per_api {
        let channel_id = ChannelId::from_index(i)
          .ok_or_else(|| format!("Invalid connection index: {}", i))?;

        // Create message channel between connection and processor
        let (connection_sender, processor_receiver) = mpsc::unbounded_channel();

        // Create managed connection
        let mut connection =
          ManagedConnection::new(channel_id, connection_sender);

        // Connect to WebSocket
        connection
          .connect(
            &group.credentials.api_key,
            &group.credentials.access_token,
            &self.config.base_config,
          )
          .await
          .map_err(|e| {
            format!(
              "Failed to connect WebSocket {} for API key {}: {}",
              i, api_key_id.0, e
            )
          })?;

        // Create message processor with unified output
        let (mut processor, output_receiver) = MessageProcessor::new(
          channel_id,
          processor_receiver,
          self.config.base_config.parser_buffer_size,
        );

        // Start processor if enabled
        if self.config.base_config.enable_dedicated_parsers {
          processor.start();
          log::info!(
            "Started dedicated parser for API key {} connection {}",
            api_key_id.0,
            i
          );
        }

        // Forward messages from this processor to unified channel
        Self::spawn_message_forwarder_static(
          unified_tx.clone(),
          api_key_id.clone(),
          output_receiver,
        );

        group.connections.push(connection);
        group.processors.push(processor);
      }

      log::info!(
        "Initialized {} connections for API key: {}",
        group.connections.len(),
        api_key_id.0
      );
    }

    log::info!(
      "MultiApiKiteTickerManager started successfully with {} API keys",
      self.api_groups.len()
    );

    Ok(())
  }

  /// Spawn a task to forward messages from a processor to the unified channel (static version)
  fn spawn_message_forwarder_static(
    tx: broadcast::Sender<(ApiKeyId, TickerMessage)>,
    api_key_id: ApiKeyId,
    mut receiver: broadcast::Receiver<TickerMessage>,
  ) {
    tokio::spawn(async move {
      loop {
        match receiver.recv().await {
          Ok(msg) => {
            // Forward to unified channel with API key identifier
            let _ = tx.send((api_key_id.clone(), msg));
          }
          Err(broadcast::error::RecvError::Closed) => {
            log::debug!(
              "Message forwarder closed for API key: {}",
              api_key_id.0
            );
            break;
          }
          Err(broadcast::error::RecvError::Lagged(n)) => {
            log::warn!(
              "Message forwarder lagged by {} messages for API key: {}",
              n,
              api_key_id.0
            );
            continue;
          }
        }
      }
    });
  }

  /// Subscribe to symbols using automatic distribution (round-robin across API keys)
  pub async fn subscribe_symbols(
    &mut self,
    symbols: &[u32],
    mode: Option<Mode>,
  ) -> Result<(), String> {
    if self.config.distribution_strategy == DistributionStrategy::Manual {
      return Err(
        "Cannot use auto-subscribe with Manual distribution strategy. Use subscribe_symbols_to_api instead.".to_string()
      );
    }

    let mode = mode.unwrap_or(self.config.base_config.default_mode);

    log::info!(
      "Subscribing to {} symbols with mode: {:?} using round-robin distribution",
      symbols.len(),
      mode
    );

    // Distribute symbols across API keys using round-robin
    for &symbol in symbols {
      // Skip if already subscribed
      if self.symbol_to_api.contains_key(&symbol) {
        log::debug!("Symbol {} already subscribed", symbol);
        continue;
      }

      // Find next API key with available capacity
      let api_key_id = self.find_available_api_key()?;

      // Subscribe to this API key
      self
        .subscribe_symbol_to_api(&api_key_id, symbol, mode)
        .await?;
    }

    log::info!("Successfully subscribed to {} symbols", symbols.len());
    Ok(())
  }

  /// Subscribe symbols to a specific API key
  pub async fn subscribe_symbols_to_api(
    &mut self,
    api_key_id: impl Into<ApiKeyId>,
    symbols: &[u32],
    mode: Option<Mode>,
  ) -> Result<(), String> {
    let api_key_id = api_key_id.into();
    let mode = mode.unwrap_or(self.config.base_config.default_mode);

    log::info!(
      "Subscribing {} symbols to API key: {} with mode: {:?}",
      symbols.len(),
      api_key_id.0,
      mode
    );

    for &symbol in symbols {
      self
        .subscribe_symbol_to_api(&api_key_id, symbol, mode)
        .await?;
    }

    log::info!(
      "Successfully subscribed {} symbols to API key: {}",
      symbols.len(),
      api_key_id.0
    );
    Ok(())
  }

  /// Subscribe a single symbol to a specific API key
  async fn subscribe_symbol_to_api(
    &mut self,
    api_key_id: &ApiKeyId,
    symbol: u32,
    mode: Mode,
  ) -> Result<(), String> {
    let group = self
      .api_groups
      .get_mut(api_key_id)
      .ok_or_else(|| format!("API key not found: {}", api_key_id.0))?;

    // Find available connection
    let connection_index = group
      .find_available_connection(
        self.config.base_config.max_symbols_per_connection,
      )
      .ok_or_else(|| {
        format!(
          "All connections at capacity for API key: {}",
          api_key_id.0
        )
      })?;

    let connection = &mut group.connections[connection_index];

    // Subscribe to symbol
    if connection.subscribed_symbols.is_empty() {
      // First subscription on this connection
      connection
        .subscribe_symbols(&[symbol], mode)
        .await
        .map_err(|e| {
          format!(
            "Failed to subscribe symbol {} on API key {}: {}",
            symbol, api_key_id.0, e
          )
        })?;

      connection.start_message_processing().await.map_err(|e| {
        format!(
          "Failed to start message processing on API key {}: {}",
          api_key_id.0, e
        )
      })?;
    } else {
      // Add to existing subscription
      connection.add_symbols(&[symbol], mode).await.map_err(|e| {
        format!(
          "Failed to add symbol {} on API key {}: {}",
          symbol, api_key_id.0, e
        )
      })?;
    }

    // Update mappings
    group
      .subscribed_symbols
      .insert(symbol, (connection_index, mode));
    self.symbol_to_api.insert(symbol, api_key_id.clone());

    Ok(())
  }

  /// Find API key with available capacity using round-robin
  fn find_available_api_key(&mut self) -> Result<ApiKeyId, String> {
    if self.api_key_order.is_empty() {
      return Err("No API keys configured".to_string());
    }

    let start_index = self.next_api_index;

    for _ in 0..self.api_key_order.len() {
      let api_key_id = &self.api_key_order[self.next_api_index];

      if let Some(group) = self.api_groups.get_mut(api_key_id) {
        // Check if this API key has capacity
        let has_capacity = group
          .find_available_connection(
            self.config.base_config.max_symbols_per_connection,
          )
          .is_some();

        if has_capacity {
          let result = api_key_id.clone();
          self.next_api_index =
            (self.next_api_index + 1) % self.api_key_order.len();
          return Ok(result);
        }
      }

      self.next_api_index =
        (self.next_api_index + 1) % self.api_key_order.len();
    }

    // Reset to start position
    self.next_api_index = start_index;
    Err("All API keys are at capacity".to_string())
  }

  /// Unsubscribe from symbols
  pub async fn unsubscribe_symbols(
    &mut self,
    symbols: &[u32],
  ) -> Result<(), String> {
    log::info!("Unsubscribing from {} symbols", symbols.len());

    // Group symbols by API key and connection
    let mut api_symbols: HashMap<ApiKeyId, Vec<u32>> = HashMap::new();

    for &symbol in symbols {
      if let Some(api_key_id) = self.symbol_to_api.get(&symbol) {
        api_symbols
          .entry(api_key_id.clone())
          .or_default()
          .push(symbol);
      }
    }

    // Unsubscribe from each API key
    for (api_key_id, symbols) in api_symbols {
      if let Some(group) = self.api_groups.get_mut(&api_key_id) {
        // Group by connection
        let mut conn_symbols: HashMap<usize, Vec<u32>> = HashMap::new();

        for symbol in symbols {
          if let Some((conn_idx, _)) = group.subscribed_symbols.get(&symbol) {
            conn_symbols.entry(*conn_idx).or_default().push(symbol);
          }
        }

        // Unsubscribe from each connection
        for (conn_idx, symbols) in conn_symbols {
          if let Some(connection) = group.connections.get_mut(conn_idx) {
            connection.remove_symbols(&symbols).await.map_err(|e| {
              format!(
                "Failed to unsubscribe from API key {}: {}",
                api_key_id.0, e
              )
            })?;
          }

          // Update group mappings
          for symbol in symbols {
            group.subscribed_symbols.remove(&symbol);
            self.symbol_to_api.remove(&symbol);
          }
        }
      }
    }

    log::info!("Successfully unsubscribed from symbols");
    Ok(())
  }

  /// Change subscription mode for existing symbols
  pub async fn change_mode(
    &mut self,
    symbols: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    log::info!("Changing mode for {} symbols to {:?}", symbols.len(), mode);

    // Group symbols by API key and connection
    let mut api_symbols: HashMap<ApiKeyId, HashMap<usize, Vec<u32>>> =
      HashMap::new();

    for &symbol in symbols {
      if let Some(api_key_id) = self.symbol_to_api.get(&symbol) {
        if let Some(group) = self.api_groups.get(api_key_id) {
          if let Some((conn_idx, _)) = group.subscribed_symbols.get(&symbol) {
            api_symbols
              .entry(api_key_id.clone())
              .or_default()
              .entry(*conn_idx)
              .or_default()
              .push(symbol);
          }
        }
      }
    }

    // Change mode on each connection
    for (api_key_id, conn_symbols) in api_symbols {
      if let Some(group) = self.api_groups.get_mut(&api_key_id) {
        for (conn_idx, symbols) in conn_symbols {
          if let Some(connection) = group.connections.get_mut(conn_idx) {
            if let Some(ref cmd) = connection.cmd_tx {
              let mode_req =
                crate::models::Request::mode(mode, &symbols).to_string();
              let _ = cmd.send(
                tokio_tungstenite::tungstenite::Message::Text(mode_req.into()),
              );

              // Update local tracking
              for &symbol in &symbols {
                connection.subscribed_symbols.insert(symbol, mode);
                group.subscribed_symbols.insert(symbol, (conn_idx, mode));
              }
            }
          }
        }
      }
    }

    log::info!("Successfully changed mode for symbols");
    Ok(())
  }

  /// Get the unified output channel that receives messages from all API keys
  ///
  /// Messages are tuples of (ApiKeyId, TickerMessage)
  pub fn get_unified_channel(
    &self,
  ) -> broadcast::Receiver<(ApiKeyId, TickerMessage)> {
    self.unified_output_tx.subscribe()
  }

  /// Get output channel for a specific API key and connection
  pub fn get_channel(
    &mut self,
    api_key_id: impl Into<ApiKeyId>,
    channel_id: ChannelId,
  ) -> Option<broadcast::Receiver<TickerMessage>> {
    let api_key_id = api_key_id.into();
    self.api_groups.get_mut(&api_key_id).and_then(|group| {
      group
        .processors
        .get_mut(channel_id.to_index())
        .map(|p| p.output_sender.subscribe())
    })
  }

  /// Get aggregate statistics across all API keys
  pub async fn get_stats(&self) -> MultiApiStats {
    let mut stats = MultiApiStats {
      total_api_keys: self.api_groups.len(),
      total_connections: 0,
      total_symbols: self.symbol_to_api.len(),
      total_messages_received: 0,
      total_messages_parsed: 0,
      total_errors: 0,
      uptime: self.start_time.elapsed(),
      per_api_stats: Vec::new(),
    };

    for group in self.api_groups.values() {
      let api_stats = group.get_stats().await;

      stats.total_connections += api_stats.active_connections;
      stats.total_messages_received += api_stats.total_messages_received;
      stats.total_messages_parsed += api_stats.total_messages_parsed;
      stats.total_errors += api_stats.total_errors;

      stats.per_api_stats.push(api_stats);
    }

    stats
  }

  /// Get statistics for a specific API key
  pub async fn get_api_stats(
    &self,
    api_key_id: impl Into<ApiKeyId>,
  ) -> Result<ApiKeyStats, String> {
    let api_key_id = api_key_id.into();
    self
      .api_groups
      .get(&api_key_id)
      .ok_or_else(|| format!("API key not found: {}", api_key_id.0))?
      .get_stats()
      .await
      .pipe(Ok)
  }

  /// Get symbol distribution across all API keys and connections
  pub fn get_symbol_distribution(
    &self,
  ) -> HashMap<ApiKeyId, HashMap<usize, Vec<u32>>> {
    let mut distribution: HashMap<ApiKeyId, HashMap<usize, Vec<u32>>> =
      HashMap::new();

    for (api_key_id, group) in &self.api_groups {
      let mut api_dist: HashMap<usize, Vec<u32>> = HashMap::new();

      for (&symbol, &(conn_idx, _)) in &group.subscribed_symbols {
        api_dist.entry(conn_idx).or_default().push(symbol);
      }

      distribution.insert(api_key_id.clone(), api_dist);
    }

    distribution
  }

  /// Get list of all configured API key IDs
  pub fn get_api_keys(&self) -> Vec<ApiKeyId> {
    self.api_key_order.clone()
  }

  /// Stop the manager and all connections
  pub async fn stop(&mut self) -> Result<(), String> {
    log::info!("Stopping MultiApiKiteTickerManager");

    for (api_key_id, group) in &mut self.api_groups {
      log::info!("Stopping connections for API key: {}", api_key_id.0);

      // Stop all processors
      for processor in &mut group.processors {
        processor.stop().await;
      }

      // Stop all connections
      for connection in &mut group.connections {
        if let Some(h) = connection.heartbeat_handle.take() {
          h.abort();
          let _ = h.await;
        }
        if let Some(handle) = connection.task_handle.take() {
          handle.abort();
          let _ = handle.await;
        }
      }
    }

    log::info!("MultiApiKiteTickerManager stopped");
    Ok(())
  }
}

// Helper trait for pipe operations
trait Pipe: Sized {
  fn pipe<F, R>(self, f: F) -> R
  where
    F: FnOnce(Self) -> R,
  {
    f(self)
  }
}

impl<T> Pipe for T {}
