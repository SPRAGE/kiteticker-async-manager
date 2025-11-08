use crate::models::Mode;
use std::time::Duration;

/// Configuration for the KiteTicker multi-connection manager
#[derive(Debug, Clone)]
pub struct KiteManagerConfig {
  /// Maximum symbols per WebSocket connection (Kite limit: 3000)
  pub max_symbols_per_connection: usize,

  /// Number of WebSocket connections to maintain (Kite limit: 3)
  pub max_connections: usize,

  /// Buffer size for each connection's message channel
  pub connection_buffer_size: usize,

  /// Buffer size for each parser's output channel
  pub parser_buffer_size: usize,

  /// Connection timeout for establishing WebSocket connections
  pub connection_timeout: Duration,

  /// Health check interval for monitoring connections
  pub health_check_interval: Duration,

  /// Maximum reconnection attempts per connection
  pub max_reconnect_attempts: usize,

  /// Delay between reconnection attempts
  pub reconnect_delay: Duration,

  /// Enable dedicated parser tasks for each connection
  pub enable_dedicated_parsers: bool,

  /// Default subscription mode for new symbols
  pub default_mode: Mode,

  /// Consider the websocket alive if a frame (including heartbeat) arrived within this duration
  pub heartbeat_liveness_threshold: Duration,
}

impl Default for KiteManagerConfig {
  fn default() -> Self {
    Self {
      max_symbols_per_connection: 3000,
      max_connections: 3,
      connection_buffer_size: 5000, // High buffer for performance
      parser_buffer_size: 10000,    // Even higher for parsed messages
      connection_timeout: Duration::from_secs(30),
      health_check_interval: Duration::from_secs(10),
      max_reconnect_attempts: 5,
      reconnect_delay: Duration::from_secs(2),
      enable_dedicated_parsers: true,
      default_mode: Mode::Quote,
      heartbeat_liveness_threshold: Duration::from_secs(10),
    }
  }
}

/// Connection statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
  pub connection_id: usize,
  pub is_connected: bool,
  pub symbol_count: usize,
  pub messages_received: u64,
  pub messages_parsed: u64,
  pub errors_count: u64,
  pub last_message_time: Option<std::time::Instant>,
  pub average_latency: Duration,
  pub connection_uptime: Duration,
}

/// Manager-wide statistics
#[derive(Debug, Clone, Default)]
pub struct ManagerStats {
  pub total_symbols: usize,
  pub active_connections: usize,
  pub total_messages_received: u64,
  pub total_messages_parsed: u64,
  pub total_errors: u64,
  pub uptime: Duration,
  pub connection_stats: Vec<ConnectionStats>,
}

/// Channel identifier for output channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelId {
  Connection1 = 0,
  Connection2 = 1,
  Connection3 = 2,
}

impl ChannelId {
  pub fn from_index(index: usize) -> Option<Self> {
    match index {
      0 => Some(Self::Connection1),
      1 => Some(Self::Connection2),
      2 => Some(Self::Connection3),
      _ => None,
    }
  }

  pub fn to_index(self) -> usize {
    self as usize
  }

  pub fn all() -> Vec<Self> {
    vec![Self::Connection1, Self::Connection2, Self::Connection3]
  }
}

// ============================================================================
// Multi-API Configuration Types
// ============================================================================

/// Unique identifier for an API key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApiKeyId(pub String);

impl ApiKeyId {
  pub fn new(id: impl Into<String>) -> Self {
    Self(id.into())
  }
}

impl From<&str> for ApiKeyId {
  fn from(s: &str) -> Self {
    Self(s.to_string())
  }
}

impl From<String> for ApiKeyId {
  fn from(s: String) -> Self {
    Self(s)
  }
}

/// API credentials for a single Kite Connect account
#[derive(Debug, Clone)]
pub struct ApiCredentials {
  pub api_key: String,
  pub access_token: String,
}

impl ApiCredentials {
  pub fn new(api_key: impl Into<String>, access_token: impl Into<String>) -> Self {
    Self {
      api_key: api_key.into(),
      access_token: access_token.into(),
    }
  }
}

/// Strategy for distributing symbols across multiple API keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionStrategy {
  /// Automatically distribute symbols across all API keys using round-robin
  RoundRobin,
  /// Manually assign symbols to specific API keys
  Manual,
}

impl Default for DistributionStrategy {
  fn default() -> Self {
    Self::RoundRobin
  }
}

/// Configuration for multi-API manager
#[derive(Debug, Clone)]
pub struct MultiApiConfig {
  /// Base configuration for each API key's connections
  pub base_config: KiteManagerConfig,
  
  /// Maximum connections per API key (Kite limit: 3)
  pub max_connections_per_api: usize,
  
  /// Symbol distribution strategy
  pub distribution_strategy: DistributionStrategy,
  
  /// Enable health monitoring across all API keys
  pub enable_health_monitoring: bool,
}

impl Default for MultiApiConfig {
  fn default() -> Self {
    Self {
      base_config: KiteManagerConfig::default(),
      max_connections_per_api: 3,
      distribution_strategy: DistributionStrategy::RoundRobin,
      enable_health_monitoring: true,
    }
  }
}

/// Statistics for a single API key
#[derive(Debug, Clone, Default)]
pub struct ApiKeyStats {
  pub api_key_id: String,
  pub active_connections: usize,
  pub total_symbols: usize,
  pub total_messages_received: u64,
  pub total_messages_parsed: u64,
  pub total_errors: u64,
  pub connection_stats: Vec<ConnectionStats>,
}

/// Aggregate statistics across all API keys
#[derive(Debug, Clone, Default)]
pub struct MultiApiStats {
  pub total_api_keys: usize,
  pub total_connections: usize,
  pub total_symbols: usize,
  pub total_messages_received: u64,
  pub total_messages_parsed: u64,
  pub total_errors: u64,
  pub uptime: Duration,
  pub per_api_stats: Vec<ApiKeyStats>,
}
