# Models API Reference

Data structures and message types used in the KiteTicker Async library.

## 📋 Table of Contents

- [Subscription Modes](#subscription-modes)
- [Ticker Messages](#ticker-messages)
- [Market Data](#market-data)
- [Statistics & Health](#statistics--health)
- [Channel Types](#channel-types)

## Subscription Modes

### `Mode`

Defines the type of market data subscription.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    LTP,    // Last Traded Price
    Quote,  // Quote (LTP + Volume + OHLC)
    Full,   // Full market depth
}
```

#### Mode Comparison

| **Mode** | **Data Included** | **Bandwidth** | **Use Case** |
|----------|-------------------|---------------|--------------|
| **LTP** | Last price only | Minimal | Price monitoring, scanning |
| **Quote** | Price + Volume + OHLC | Medium | Trading decisions |
| **Full** | Complete market depth | High | Order book analysis |

**Example:**
```rust
use kiteticker_async_manager::Mode;

// Subscribe with different modes
manager.subscribe_symbols(&[256265], Some(Mode::LTP)).await?;   // Minimal data
manager.subscribe_symbols(&[408065], Some(Mode::Quote)).await?; // Standard data  
manager.subscribe_symbols(&[738561], Some(Mode::Full)).await?;  // Complete data
```

## Ticker Messages

### `TickerMessage`

Main message type received from WebSocket connections.

```rust
#[derive(Debug, Clone)]
pub enum TickerMessage {
    Ticks(Vec<TickMessage>),
    Text(TextMessage),
    Error(String),
}
```

#### Message Types

##### `Ticks(Vec<TickMessage>)`
- **Description:** Market data updates for subscribed instruments
- **Frequency:** Real-time, as market data changes
- **Content:** Array of tick data for multiple instruments

##### `Text(TextMessage)`  
- **Description:** Text messages from the server
- **Content:** Connection status, error messages, confirmations

##### `Error(String)`
- **Description:** Error messages and connection issues
- **Usage:** Handle connection problems and invalid operations

**Example:**
```rust
while let Ok(message) = receiver.recv().await {
    match message {
        TickerMessage::Ticks(ticks) => {
            for tick in ticks {
                println!("Instrument {}: Price {}", 
                    tick.instrument_token, 
                    tick.content.last_price.unwrap_or(0.0));
            }
        }
        TickerMessage::Text(text) => {
            println!("Server message: {}", text.message);
        }
        TickerMessage::Error(error) => {
            eprintln!("Error: {}", error);
        }
    }
}
```

## Market Data

### `TickMessage`

Individual market data update for a single instrument.

```rust
#[derive(Debug, Clone)]
pub struct TickMessage {
    pub instrument_token: u32,
    pub content: TickContent,
}
```

### `TickContent`

Market data content with mode-specific fields.

```rust
#[derive(Debug, Clone)]
pub struct TickContent {
    pub mode: Mode,
    pub exchange_timestamp: Option<DateTime<Utc>>,
    pub last_price: Option<f64>,
    pub last_quantity: Option<u32>,
    pub average_price: Option<f64>,
    pub volume: Option<u32>,
    pub buy_quantity: Option<u32>,
    pub sell_quantity: Option<u32>,
    pub ohlc: Option<OHLC>,
    pub net_change: Option<f64>,
    pub depth: Option<MarketDepth>,
}
```

#### Field Availability by Mode

| **Field** | **LTP** | **Quote** | **Full** | **Description** |
|-----------|---------|-----------|-----------|-----------------|
| `last_price` | ✅ | ✅ | ✅ | Last traded price |
| `last_quantity` | ❌ | ✅ | ✅ | Last traded quantity |
| `average_price` | ❌ | ✅ | ✅ | Average traded price |
| `volume` | ❌ | ✅ | ✅ | Total volume |
| `buy_quantity` | ❌ | ✅ | ✅ | Total buy quantity |
| `sell_quantity` | ❌ | ✅ | ✅ | Total sell quantity |
| `ohlc` | ❌ | ✅ | ✅ | Open, High, Low, Close |
| `net_change` | ❌ | ✅ | ✅ | Net change from previous close |
| `depth` | ❌ | ❌ | ✅ | Market depth (order book) |

### `OHLC`

Open, High, Low, Close data.

```rust
#[derive(Debug, Clone)]
pub struct OHLC {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
```

### `MarketDepth`

Order book depth data (Full mode only).

```rust
#[derive(Debug, Clone)]
pub struct MarketDepth {
    pub buy: Vec<DepthItem>,
    pub sell: Vec<DepthItem>,
}

#[derive(Debug, Clone)]
pub struct DepthItem {
    pub quantity: u32,
    pub price: f64,
    pub orders: u32,
}
```

**Example:**
```rust
if let Some(depth) = &tick.content.depth {
    // Best bid/ask
    if let Some(best_bid) = depth.buy.first() {
        println!("Best bid: {} @ {}", best_bid.quantity, best_bid.price);
    }
    if let Some(best_ask) = depth.sell.first() {
        println!("Best ask: {} @ {}", best_ask.quantity, best_ask.price);
    }
}
```

## Statistics & Health

### `ManagerStats`

Overall manager statistics.

```rust
#[derive(Debug, Clone)]
pub struct ManagerStats {
    pub active_connections: usize,
    pub total_symbols: usize,
    pub total_messages_received: u64,
    pub total_errors: u64,
    pub connection_stats: Vec<ConnectionStats>,
}
```

### `ConnectionStats`

Statistics for individual connections.

```rust
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub connection_id: usize,
    pub is_connected: bool,
    pub symbol_count: usize,
    pub messages_received: u64,
    pub errors_count: u64,
    pub connection_uptime: Duration,
    pub last_message_time: Option<Instant>,
}
```

### `HealthSummary`

Health status across all connections.

```rust
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub healthy_connections: usize,
    pub total_connections: usize,
    pub overall_health: bool,
    pub connection_health: Vec<bool>,
}
```

### `ProcessorStats`

Parser performance statistics.

```rust
#[derive(Debug, Clone)]
pub struct ProcessorStats {
    pub messages_processed: u64,
    pub messages_per_second: f64,
    pub processing_latency_avg: Duration,
    pub processing_latency_max: Duration,
    pub last_activity: Instant,
}
```

## Channel Types

### `ChannelId`

Identifier for WebSocket connections.

```rust
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ChannelId {
    Connection1,
    Connection2,
    Connection3,
}

impl ChannelId {
    pub fn to_index(self) -> usize {
        match self {
            ChannelId::Connection1 => 0,
            ChannelId::Connection2 => 1,
            ChannelId::Connection3 => 2,
        }
    }
}
```

## Usage Examples

### Processing Market Data

```rust
use kiteticker_async_manager::{TickerMessage, Mode};

while let Ok(message) = receiver.recv().await {
    if let TickerMessage::Ticks(ticks) = message {
        for tick in ticks {
            match tick.content.mode {
                Mode::LTP => {
                    if let Some(price) = tick.content.last_price {
                        println!("LTP: {} @ {}", tick.instrument_token, price);
                    }
                }
                Mode::Quote => {
                    if let (Some(price), Some(volume)) = (
                        tick.content.last_price, 
                        tick.content.volume
                    ) {
                        println!("Quote: {} @ {} (Vol: {})", 
                            tick.instrument_token, price, volume);
                    }
                }
                Mode::Full => {
                    // Process complete market depth
                    if let Some(depth) = &tick.content.depth {
                        process_market_depth(tick.instrument_token, depth);
                    }
                }
            }
        }
    }
}
```

### Extracting OHLC Data

```rust
fn extract_ohlc(tick: &TickMessage) -> Option<(f64, f64, f64, f64)> {
    tick.content.ohlc.as_ref().map(|ohlc| {
        (ohlc.open, ohlc.high, ohlc.low, ohlc.close)
    })
}

// Usage
if let Some((open, high, low, close)) = extract_ohlc(&tick) {
    println!("OHLC: O:{} H:{} L:{} C:{}", open, high, low, close);
}
```

### Monitoring Connection Health

```rust
// Get health status
let health = manager.get_health().await?;
println!("Healthy connections: {}/{}", 
    health.healthy_connections, health.total_connections);

// Get detailed stats
let stats = manager.get_stats().await?;
for conn_stat in &stats.connection_stats {
    println!("Connection {}: {} symbols, {} messages",
        conn_stat.connection_id,
        conn_stat.symbol_count,
        conn_stat.messages_received);
}
```
