# KiteTicker Async Manager

High-performance async WebSocket client for the [Kite Connect API](https://kite.trade/docs/connect/v3/websocket/#websocket-streaming) with multi-connection support and dynamic subscription management.

[![Crates.io][crates-badge]][crates-url]
[![Apache-2.0 Licensed][apache-2-0-badge]][apache-2-0-url]
[![Documentation][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/kiteticker-async-manager.svg
[crates-url]: https://crates.io/crates/kiteticker-async-manager
[apache-2-0-badge]: https://img.shields.io/badge/license-apache-blue.svg
[apache-2-0-url]: https://github.com/SPRAGE/kiteticker-async-manager/blob/master/LICENSE
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg
[docs-url]: https://docs.rs/kiteticker-async-manager/latest/kiteticker-async-manager

**[📚 Documentation](docs/)** |
**[🚀 Getting Started](docs/guides/getting-started.md)** |
**[📝 Examples](examples/)** |
**[🔧 API Reference](docs/api/)**

## ✨ Key Features

- **🚀 Multi-Connection Support** - Utilize all 3 allowed WebSocket connections (9,000 symbol capacity)
- **🔑 Multi-API Support** - Manage multiple Kite Connect accounts simultaneously (18,000+ symbols)
- **⚡ High Performance** - Dedicated parser tasks, optimized buffers, sub-microsecond latency
- **🔄 Dynamic Subscriptions** - Add/remove symbols at runtime without reconnection
- **📊 Load Balancing** - Automatic symbol distribution across connections
- **💪 Production Ready** - Comprehensive error handling, health monitoring, reconnection
- **🔧 Async-First Design** - Built with Tokio, follows Rust async best practices
- **🧩 Zero-Copy Raw Access** - Optional, fully safe, endian-correct views over packet bytes

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
kiteticker-async-manager = "0.2.1"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Setup credentials
    let api_key = std::env::var("KITE_API_KEY").unwrap();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
    
    // Create high-performance manager
    let config = KiteManagerConfig {
        max_connections: 3,
        max_symbols_per_connection: 3000,
        enable_dedicated_parsers: true,
        default_mode: Mode::LTP,
        ..Default::default()
    };
    
    // Start manager
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // Subscribe to symbols (automatically distributed across connections)
    let symbols = vec![256265, 408065, 738561]; // NIFTY 50, HDFC Bank, Reliance
    manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
    
    // Process data from independent channels
    let channels = manager.get_all_channels();
    for (channel_id, mut receiver) in channels {
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    for tick in ticks {
                        println!("Channel {:?}: {} @ ₹{:.2}",
                            channel_id, 
                            tick.instrument_token,
                            tick.content.last_price.unwrap_or(0.0));
                    }
                }
            }
        });
    }
    
    // Add symbols dynamically
    manager.subscribe_symbols(&[5633, 884737], Some(Mode::Full)).await?;
    
    // Remove symbols
    manager.unsubscribe_symbols(&[408065]).await?;
    
    // Change subscription mode
    manager.change_mode(&[256265], Mode::Full).await?;
    
    Ok(())
}
```

### Multi-API Manager (NEW!)

Manage multiple Kite Connect accounts in a single manager:

```rust
use kiteticker_async_manager::{
    MultiApiKiteTickerManager, 
    DistributionStrategy, 
    Mode,
};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Create multi-API manager
    let mut manager = MultiApiKiteTickerManager::builder()
        .add_api_key("account1", "api_key_1", "token_1")
        .add_api_key("account2", "api_key_2", "token_2")
        .max_connections_per_api(3)
        .distribution_strategy(DistributionStrategy::RoundRobin)
        .build();
    
    manager.start().await?;
    
    // Subscribe symbols (auto-distributed across API keys)
    let symbols = vec![256265, 408065, 738561];
    manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
    
    // Or assign to specific API key
    manager.subscribe_symbols_to_api("account1", &symbols, Some(Mode::LTP)).await?;
    
    // Receive unified stream from all API keys
    let mut unified = manager.get_unified_channel();
    while let Ok((api_key_id, message)) = unified.recv().await {
        println!("From {}: {:?}", api_key_id.0, message);
    }
    
    Ok(())
}
```

**[📖 Multi-API Guide](docs/guides/MULTI_API_GUIDE.md)** - Complete multi-API documentation

## 📊 Performance Comparison

| **Feature** | **Single Connection** | **Multi-Connection Manager** | **Multi-API Manager** | **Improvement** |
|-------------|----------------------|------------------------------|----------------------|-----------------|
| **Max Symbols** | 3,000 | 9,000 | 18,000+ (9K × N APIs) | **6x+ capacity** |
| **Max API Keys** | 1 | 1 | Unlimited | **Multi-account** |
| **Throughput** | Limited by 1 connection | 3 parallel connections | 3 × N connections | **N × 3x throughput** |
| **Latency** | ~5-10µs | ~1-2µs | ~1-2µs | **5x faster** |
| **Resilience** | Single point of failure | 3 independent connections | Multi-account redundancy | **High availability** |
| **Dynamic Ops** | Manual reconnection | Runtime add/remove | Runtime add/remove | **Zero downtime** |

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    KiteTickerManager                        │
├─────────────────────────────────────────────────────────────┤
│  📊 Symbol Distribution (9,000 symbols max)                │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │Connection 1 │ │Connection 2 │ │Connection 3 │          │
│  │3,000 symbols│ │3,000 symbols│ │3,000 symbols│          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
├─────────────────────────────────────────────────────────────┤
│  ⚡ Dedicated Parser Tasks (CPU Optimized)                 │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   Parser 1  │ │   Parser 2  │ │   Parser 3  │          │
│  │  ~1µs latency│ │  ~1µs latency│ │  ~1µs latency│          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
├─────────────────────────────────────────────────────────────┤
│  📡 Independent Output Channels                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │  Channel 1  │ │  Channel 2  │ │  Channel 3  │          │
│  │broadcast::Rx│ │broadcast::Rx│ │broadcast::Rx│          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

## 📚 Documentation

- **[📖 Getting Started](docs/guides/getting-started.md)** - Complete beginner's guide
- **[� Multi-API Guide](docs/guides/MULTI_API_GUIDE.md)** - Manage multiple API keys (NEW!)
- **[�🔧 API Reference](docs/api/)** - Detailed API documentation
- **[📝 Examples](examples/)** - Practical code examples
- **[🔄 Dynamic Subscriptions](docs/guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)** - Runtime symbol management
- **[⚡ Performance Guide](docs/guides/PERFORMANCE_IMPROVEMENTS.md)** - Optimization techniques

## 📁 Examples

### 🔰 Basic Examples
- **[Single Connection](examples/basic/single_connection.rs)** - Simple WebSocket usage
- **[Portfolio Monitor](examples/basic/portfolio_monitor.rs)** - Track portfolio stocks
- **[Runtime Subscriptions](examples/basic/runtime_subscription_example.rs)** - Dynamic symbol management

### 🔑 Multi-API Examples
- **[Multi-API Demo](examples/multi_api_demo.rs)** - Manage multiple Kite Connect accounts (NEW!)

### 🚀 Advanced Examples
- **[Dynamic Demo](examples/advanced/dynamic_subscription_demo.rs)** - Complete dynamic workflow
- **[Manager Demo](examples/advanced/manager_demo.rs)** - Multi-connection setup
- **[Market Scanner](examples/advanced/market_scanner.rs)** - High-volume scanning

### ⚡ Performance Examples
- **[Performance Demo](examples/performance/performance_demo.rs)** - Benchmarking
- **[High Frequency](examples/performance/high_frequency.rs)** - Maximum throughput
 - **[Raw vs Parsed](examples/performance/raw_vs_parsed.rs)** - Micro-benchmark of raw vs parsed
 - **[Raw Full Peek](examples/performance/raw_full_peek.rs)** - Zero-copy field peeking for all packet sizes

## 🎯 Use Cases

| **Use Case** | **Configuration** | **Symbols** | **Example** |
|-------------|------------------|-------------|-------------|
| **Portfolio Monitoring** | 1 connection, Quote mode | 10-50 | Track personal investments |
| **Algorithmic Trading** | 3 connections, Quote mode | 100-1,000 | Trading strategies |
| **Market Scanner** | 3 connections, LTP mode | 1,000-9,000 | Scan entire market |
| **High-Frequency Trading** | 3 connections, Full mode | 500-3,000 | Order book analysis |

## ⚙️ Configuration Presets

### Development
```rust
let config = KiteManagerConfig {
    max_connections: 1,
    max_symbols_per_connection: 100,
    default_mode: Mode::Full,
    ..Default::default()
};
```

### Production  
```rust
let config = KiteManagerConfig {
    max_connections: 3,
    max_symbols_per_connection: 3000,
    connection_buffer_size: 20000,
    parser_buffer_size: 50000,
    enable_dedicated_parsers: true,
    default_mode: Mode::LTP,
    ..Default::default()
};
```

## 🆚 Comparison with Official Library

| **Feature** | **Official kiteconnect-rs** | **kiteticker-async-manager** |
|-------------|------------------------------|----------------------|
| **Maintenance** | ❌ Unmaintained | ✅ Actively maintained |
| **Async Support** | ❌ Callback-based | ✅ Full async/await |
| **Type Safety** | ❌ Untyped JSON | ✅ Fully typed structs |
| **Multi-Connection** | ❌ Single connection | ✅ Up to 3 connections |
| **Dynamic Subscriptions** | ❌ Manual reconnection | ✅ Runtime add/remove |
| **Performance** | ❌ Basic | ✅ High-performance optimized |
| **Error Handling** | ❌ Limited | ✅ Comprehensive |

## 🛠️ Development

### Prerequisites
```bash
# Install Rust and tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install just  # Task runner
```

### Building
```bash
# Clone and build
git clone https://github.com/SPRAGE/kiteticker-async-manager.git
cd kiteticker-async-manager
just build
```

### Running Examples
```bash
# Set API credentials
export KITE_API_KEY=your_api_key
export KITE_ACCESS_TOKEN=your_access_token

# Run examples
cargo run --example single_connection
cargo run --example dynamic_subscription_demo
cargo run --example raw_full_peek --release
cargo run --example raw_vs_parsed --release
```

### Available Tasks
```bash
just --list
```

## 📦 Features

- **Multi-Connection Management** - Utilize all 3 WebSocket connections
- **Dynamic Subscriptions** - Add/remove symbols without reconnection  
- **Load Balancing** - Automatic symbol distribution
- **High Performance** - Dedicated parsers, optimized buffers
- **Type Safety** - Fully typed market data structures
- **Error Resilience** - Comprehensive error handling and recovery
- **Health Monitoring** - Real-time connection health tracking
- **Async-First** - Built for modern Rust async ecosystems

## 🤝 Contributing

Contributions are welcome! Please see our [contribution guidelines](CONTRIBUTING.md).

### Development Setup

Use [just](https://github.com/casey/just) to run development tasks:

```bash
just --list  # Show available tasks
just build   # Build the project
just check   # Check code formatting and lints
```

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🔗 Links

- **[Kite Connect API Documentation](https://kite.trade/docs/connect/v3/websocket/)**
- **[Crates.io](https://crates.io/crates/kiteticker-async-manager)**
- **[Documentation](https://docs.rs/kiteticker-async-manager/)**
- **[GitHub Repository](https://github.com/SPRAGE/kiteticker-async-manager)**

---

**⭐ Star this repository if you find it useful!**

## 🔬 Zero-copy raw access (advanced)

For maximum throughput with minimal allocations, you can work directly with raw WebSocket frame bytes and view packet bodies using endian-safe, zero-copy structs.

Key points:

- Subscribe to raw frames via `subscribe_raw_frames()` on `KiteTickerAsync`, or
    via the manager using `get_raw_frame_channel(ChannelId)` or `get_all_raw_frame_channels()`
- Extract packet bodies using their length prefixes
- Create typed views with `as_tick_raw`, `as_index_quote_32`, or `as_inst_header_64`
- The returned `zerocopy::Ref<&[u8], T>` dereferences to `&T` and is valid while the backing bytes live (store `Bytes` to keep alive)

Example snippet:

```rust
use kiteticker_async_manager::{KiteTickerAsync, Mode, as_tick_raw};
use bytes::Bytes;

# async fn demo(mut ticker: KiteTickerAsync) -> Result<(), String> {
let mut frames = ticker.subscribe_raw_frames();
let frame: Bytes = frames.recv().await.unwrap();
let num = u16::from_be_bytes([frame[0], frame[1]]) as usize;
let mut off = 2usize;
for _ in 0..num {
    let len = u16::from_be_bytes([frame[off], frame[off+1]]) as usize;
    let body = frame.slice(off+2..off+2+len);
    if len == 184 {
        if let Some(view_ref) = as_tick_raw(&body) {
            let tick = &*view_ref;
            println!("token={} ltp_scaled={}", tick.header.instrument_token.get(), tick.header.last_price.get());
        }
    }
    off += 2 + len;
}
Ok(())
# }
```

Safety: All raw structs derive `Unaligned` and use big-endian wrappers; no `unsafe` is required.

### Manager-level raw frames

```rust
use kiteticker_async_manager::{KiteTickerManagerBuilder, Mode, ChannelId, as_tick_raw};

# #[tokio::main]
# async fn main() -> Result<(), String> {
let api_key = std::env::var("KITE_API_KEY").unwrap();
let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
let mut mgr = KiteTickerManagerBuilder::new(api_key, access_token)
    .raw_only(true)
    .build();
mgr.start().await?;
mgr.subscribe_symbols(&[256265], Some(Mode::Full)).await?;

for (id, mut rx) in mgr.get_all_raw_frame_channels() {
    tokio::spawn(async move {
        while let Ok(frame) = rx.recv().await {
            if frame.len() < 2 { continue; }
            let mut off = 2usize;
            let num = u16::from_be_bytes([frame[0], frame[1]]) as usize;
            for _ in 0..num {
                if off + 2 > frame.len() { break; }
                let len = u16::from_be_bytes([frame[off], frame[off+1]]) as usize;
                let body = frame.slice(off+2..off+2+len);
                if len == 184 {
                    if let Some(view) = as_tick_raw(&body) {
                        let token = view.header.instrument_token.get();
                        println!("conn={:?} token={}", id, token);
                    }
                }
                off += 2 + len;
            }
        }
    });
}
# Ok(()) }
```

Or, if you only want Full depth packets, use the helper:

```rust
use kiteticker_async_manager::{KiteTickerManagerBuilder, Mode, ChannelId, KiteTickerRawSubscriber184};

# #[tokio::main]
# async fn main() -> Result<(), String> {
let api_key = std::env::var("KITE_API_KEY").unwrap();
let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
let mut mgr = KiteTickerManagerBuilder::new(api_key, access_token)
    .raw_only(true)
    .build();
mgr.start().await?;
mgr.subscribe_symbols(&[256265], Some(Mode::Full)).await?;

if let Some(mut sub) = mgr.get_full_raw_subscriber(ChannelId::Connection1) {
    tokio::spawn(async move {
        while let Ok(Some(view)) = sub.recv_raw_tickraw().await {
            let t = &*view; // &TickRaw
            println!("token={} ltp={}", t.header.instrument_token.get(), t.header.last_price.get());
        }
    });
}
# Ok(()) }
```
