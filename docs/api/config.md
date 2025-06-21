# Configuration API Reference

Configuration options for the KiteTicker Async library.

## KiteManagerConfig

The main configuration struct for `KiteTickerManager`.

```rust
pub struct KiteManagerConfig {
    pub max_symbols_per_connection: usize,
    pub max_connections: usize,
    pub connection_buffer_size: usize,
    pub parser_buffer_size: usize,
    pub connection_timeout: Duration,
    pub health_check_interval: Duration,
    pub max_reconnect_attempts: usize,
    pub reconnect_delay: Duration,
    pub enable_dedicated_parsers: bool,
    pub default_mode: Mode,
}
```

## Configuration Options

### Connection Settings

#### `max_connections: usize`
- **Default:** `3`
- **Range:** `1-3`
- **Description:** Number of WebSocket connections to maintain
- **Note:** KiteConnect allows maximum 3 concurrent connections

#### `max_symbols_per_connection: usize`
- **Default:** `3000`
- **Range:** `1-3000`
- **Description:** Maximum symbols per WebSocket connection
- **Note:** KiteConnect limit is 3000 symbols per connection

#### `connection_timeout: Duration`
- **Default:** `Duration::from_secs(30)`
- **Description:** Timeout for WebSocket connection establishment

### Buffer Settings

#### `connection_buffer_size: usize`
- **Default:** `5000`
- **Description:** Buffer size for WebSocket message queue
- **Performance:** Higher values improve throughput but use more memory

#### `parser_buffer_size: usize`  
- **Default:** `10000`
- **Description:** Buffer size for parsed message queue
- **Performance:** Larger buffers prevent message dropping under high load

### Health & Reconnection

#### `health_check_interval: Duration`
- **Default:** `Duration::from_secs(5)`
- **Description:** Interval for connection health monitoring

#### `max_reconnect_attempts: usize`
- **Default:** `5`
- **Description:** Maximum reconnection attempts on connection failure

#### `reconnect_delay: Duration`
- **Default:** `Duration::from_secs(2)`
- **Description:** Delay between reconnection attempts

### Processing Options

#### `enable_dedicated_parsers: bool`
- **Default:** `true`
- **Description:** Enable dedicated parser tasks for each connection
- **Performance:** Improves CPU utilization and prevents I/O blocking

#### `default_mode: Mode`
- **Default:** `Mode::LTP`
- **Description:** Default subscription mode when not specified
- **Options:** `LTP`, `Quote`, `Full`

## Preset Configurations

### High Performance Configuration

```rust
use std::time::Duration;
use kiteticker_async_manager::{KiteManagerConfig, Mode};

let config = KiteManagerConfig {
    max_symbols_per_connection: 3000,
    max_connections: 3,
    connection_buffer_size: 20000,
    parser_buffer_size: 50000,
    connection_timeout: Duration::from_secs(30),
    health_check_interval: Duration::from_secs(5),
    max_reconnect_attempts: 10,
    reconnect_delay: Duration::from_secs(1),
    enable_dedicated_parsers: true,
    default_mode: Mode::LTP,
};
```

**Use case:** High-frequency trading, maximum throughput

### Memory Optimized Configuration

```rust
let config = KiteManagerConfig {
    max_symbols_per_connection: 1000,
    max_connections: 2,
    connection_buffer_size: 1000,
    parser_buffer_size: 2000,
    connection_timeout: Duration::from_secs(15),
    health_check_interval: Duration::from_secs(10),
    max_reconnect_attempts: 3,
    reconnect_delay: Duration::from_secs(5),
    enable_dedicated_parsers: false,
    default_mode: Mode::LTP,
};
```

**Use case:** Resource-constrained environments

### Development Configuration

```rust
let config = KiteManagerConfig {
    max_symbols_per_connection: 100,
    max_connections: 1,
    connection_buffer_size: 500,
    parser_buffer_size: 1000,
    connection_timeout: Duration::from_secs(10),
    health_check_interval: Duration::from_secs(15),
    max_reconnect_attempts: 2,
    reconnect_delay: Duration::from_secs(3),
    enable_dedicated_parsers: true,
    default_mode: Mode::Full,
};
```

**Use case:** Development and testing

## Configuration Examples

### Trading Strategy Configuration

```rust
// For algorithmic trading with dynamic symbol rotation
let config = KiteManagerConfig {
    max_symbols_per_connection: 2000,
    max_connections: 3,
    connection_buffer_size: 15000,
    parser_buffer_size: 30000,
    enable_dedicated_parsers: true,
    default_mode: Mode::Quote, // Price + volume data
    ..Default::default()
};
```

### Market Scanner Configuration

```rust
// For scanning large number of symbols
let config = KiteManagerConfig {
    max_symbols_per_connection: 3000,
    max_connections: 3,
    connection_buffer_size: 25000,
    parser_buffer_size: 50000,
    enable_dedicated_parsers: true,
    default_mode: Mode::LTP, // Minimal data for scanning
    ..Default::default()
};
```

### Portfolio Monitoring Configuration

```rust
// For monitoring a portfolio of stocks
let config = KiteManagerConfig {
    max_symbols_per_connection: 500,
    max_connections: 1,
    connection_buffer_size: 2000,
    parser_buffer_size: 5000,
    health_check_interval: Duration::from_secs(10),
    default_mode: Mode::Full, // Complete market depth
    ..Default::default()
};
```

## Performance Tuning Guidelines

### Buffer Sizing

| **Load Level** | **Connection Buffer** | **Parser Buffer** | **Description** |
|----------------|----------------------|-------------------|-----------------|
| **Light** | 1,000-2,000 | 2,000-5,000 | <100 symbols, infrequent updates |
| **Medium** | 5,000-10,000 | 10,000-20,000 | 100-1000 symbols, regular updates |
| **Heavy** | 15,000-25,000 | 30,000-50,000 | >1000 symbols, high-frequency |

### Connection Strategy

| **Use Case** | **Connections** | **Symbols/Conn** | **Rationale** |
|--------------|----------------|------------------|---------------|
| **Single Strategy** | 1 | Up to 3000 | Simple management |
| **Multiple Strategies** | 2-3 | 1000-1500 | Isolation between strategies |
| **Maximum Throughput** | 3 | 3000 | Utilize all available capacity |

### Memory vs Performance Trade-offs

```rust
// Memory optimized (lower buffers)
let memory_config = KiteManagerConfig {
    connection_buffer_size: 1000,
    parser_buffer_size: 2000,
    enable_dedicated_parsers: false, // Shared parsing
    ..Default::default()
};

// Performance optimized (higher buffers)
let performance_config = KiteManagerConfig {
    connection_buffer_size: 20000,
    parser_buffer_size: 50000,
    enable_dedicated_parsers: true, // Dedicated parsing
    ..Default::default()
};
```

## Validation

The configuration is validated on manager creation:

```rust
// This will return an error if invalid
let result = KiteTickerManager::new(api_key, access_token, config);
```

**Validation rules:**
- `max_connections` must be 1-3
- `max_symbols_per_connection` must be 1-3000
- Buffer sizes must be > 0
- Timeouts must be > 0
