# KiteTickerManager API Reference

The `KiteTickerManager` provides a high-level interface for managing multiple WebSocket connections to the Kite Connect ticker API.

## 📋 Table of Contents

- [Constructor](#constructor)
- [Connection Management](#connection-management)
- [Subscription Management](#subscription-management)
- [Data Access](#data-access)
- [Monitoring & Stats](#monitoring--stats)
- [Configuration](#configuration)

## Constructor

### `KiteTickerManager::new`

```rust
pub fn new(
    api_key: String,
    access_token: String,
    config: KiteManagerConfig,
) -> Self
```

Creates a new manager instance with the specified configuration.

**Parameters:**
- `api_key`: Your Kite Connect API key
- `access_token`: Valid access token
- `config`: Manager configuration (see [Configuration API](config.md))

**Example:**
```rust
let config = KiteManagerConfig {
    max_connections: 3,
    max_symbols_per_connection: 3000,
    default_mode: Mode::LTP,
    ..Default::default()
};

let manager = KiteTickerManager::new(
    "your_api_key".to_string(),
    "your_access_token".to_string(),
    config,
);
```

## Connection Management

### `start()`

```rust
pub async fn start(&mut self) -> Result<(), String>
```

Initializes and starts all WebSocket connections.

**Returns:** `Result<(), String>` - Success or error message

**Example:**
```rust
manager.start().await?;
```

### `stop()`

```rust
pub async fn stop(&mut self) -> Result<(), String>
```

Gracefully stops all connections and cleanup resources.

**Returns:** `Result<(), String>` - Success or error message

## Subscription Management

### `subscribe_symbols()`

```rust
pub async fn subscribe_symbols(
    &mut self,
    symbols: &[u32],
    mode: Option<Mode>,
) -> Result<(), String>
```

Subscribe to symbols with automatic load balancing across connections.

**Parameters:**
- `symbols`: Array of instrument tokens
- `mode`: Subscription mode (LTP, Quote, Full) - uses default if None

**Features:**
- ✅ **Automatic deduplication** - Skips already subscribed symbols
- ✅ **Load balancing** - Distributes across available connections
- ✅ **Dynamic capacity** - Finds connections with available slots

**Example:**
```rust
// Subscribe to NIFTY 50, Reliance, HDFC Bank
let symbols = vec![256265, 738561, 408065];
manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;
```

### `unsubscribe_symbols()`

```rust
pub async fn unsubscribe_symbols(
    &mut self,
    symbols: &[u32],
) -> Result<(), String>
```

Unsubscribe from symbols across all connections.

**Parameters:**
- `symbols`: Array of instrument tokens to unsubscribe

**Features:**
- ✅ **Safe removal** - Ignores non-existent symbols
- ✅ **Automatic cleanup** - Updates internal mappings
- ✅ **Multi-connection** - Handles symbols across different connections

### `change_mode()`

```rust
pub async fn change_mode(
    &mut self,
    symbols: &[u32],
    mode: Mode,
) -> Result<(), String>
```

Change subscription mode for existing symbols.

**Parameters:**
- `symbols`: Array of instrument tokens
- `mode`: New subscription mode

**Example:**
```rust
// Change to Full mode for detailed market depth
manager.change_mode(&[256265, 738561], Mode::Full).await?;
```

## Data Access

### `get_all_channels()`

```rust
pub fn get_all_channels(&mut self) -> Vec<(ChannelId, broadcast::Receiver<TickerMessage>)>
```

Get independent data channels for each connection.

**Returns:** Vector of `(ChannelId, Receiver)` pairs

**Example:**
```rust
let channels = manager.get_all_channels();

for (channel_id, mut receiver) in channels {
    tokio::spawn(async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                TickerMessage::Ticks(ticks) => {
                    println!("Channel {:?}: {} ticks", channel_id, ticks.len());
                }
                _ => {}
            }
        }
    });
}
```

### `get_symbol_distribution()`

```rust
pub fn get_symbol_distribution(&self) -> HashMap<ChannelId, Vec<u32>>
```

Get current symbol distribution across connections.

**Returns:** Mapping of connection ID to subscribed symbols

## Monitoring & Stats

### `get_stats()`

```rust
pub async fn get_stats(&self) -> Result<ManagerStats, String>
```

Get comprehensive manager statistics.

**Returns:** `ManagerStats` with connection and performance data

### `get_health()`

```rust
pub async fn get_health(&self) -> Result<HealthSummary, String>
```

Get health status of all connections.

### `get_processor_stats()`

```rust
pub async fn get_processor_stats(&self) -> Vec<(ChannelId, ProcessorStats)>
```

Get parser performance statistics for each connection.

## Example Usage Patterns

### Basic Multi-Connection Setup

```rust
use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Configure for high-performance
    let config = KiteManagerConfig {
        max_connections: 3,
        max_symbols_per_connection: 3000,
        connection_buffer_size: 10000,
        parser_buffer_size: 20000,
        enable_dedicated_parsers: true,
        default_mode: Mode::LTP,
        ..Default::default()
    };
    
    // Start manager
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // Subscribe to symbols
    let symbols = vec![256265, 738561, 408065]; // NIFTY, Reliance, HDFC
    manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;
    
    // Process data from all connections
    let channels = manager.get_all_channels();
    // ... handle data streams
    
    Ok(())
}
```

### Dynamic Subscription Management

```rust
// Add symbols at runtime
manager.subscribe_symbols(&[884737, 5633], Some(Mode::Quote)).await?;

// Change modes dynamically
manager.change_mode(&[256265], Mode::Full).await?;

// Remove symbols
manager.unsubscribe_symbols(&[738561]).await?;

// Check distribution
let distribution = manager.get_symbol_distribution();
println!("Current distribution: {:?}", distribution);
```

## Performance Characteristics

| **Metric** | **Value** | **Description** |
|------------|-----------|-----------------|
| **Max Connections** | 3 | Utilizes all allowed KiteConnect connections |
| **Symbols per Connection** | 3,000 | Maximum symbols per WebSocket |
| **Total Capacity** | 9,000 | Combined symbol limit |
| **Latency** | ~1-2µs | Message processing latency |
| **Throughput** | High | Dedicated parser tasks |

## Error Handling

All methods return `Result<T, String>` for error handling:

```rust
match manager.subscribe_symbols(&symbols, Some(Mode::LTP)).await {
    Ok(()) => println!("Subscription successful"),
    Err(e) => eprintln!("Subscription failed: {}", e),
}
```

Common error scenarios:
- **Authentication failure** - Invalid API credentials
- **Connection timeout** - Network connectivity issues  
- **Capacity exceeded** - Too many symbols for available connections
- **Invalid symbols** - Non-existent instrument tokens

## Thread Safety

- `KiteTickerManager` is **not thread-safe** - use from single task
- Data channels (`broadcast::Receiver`) **are thread-safe** - can be shared
- Multiple receivers can subscribe to the same channel
