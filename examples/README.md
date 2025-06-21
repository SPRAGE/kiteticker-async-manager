# KiteTicker Async Examples

This directory contains practical examples demonstrating various features of the KiteTicker Async library.

## 📁 Example Categories

### 🔰 [Basic Examples](basic/)

Simple, focused examples perfect for getting started:

- **[runtime_subscription_example.rs](basic/runtime_subscription_example.rs)** - Adding/removing symbols at runtime
- **[single_connection.rs](basic/single_connection.rs)** - Basic single WebSocket connection
- **[portfolio_monitor.rs](basic/portfolio_monitor.rs)** - Monitor a portfolio of stocks

### 🎯 [Advanced Examples](advanced/)

Complex scenarios and advanced features:

- **[dynamic_subscription_demo.rs](advanced/dynamic_subscription_demo.rs)** - Complete dynamic subscription workflow
- **[manager_demo.rs](advanced/manager_demo.rs)** - Multi-connection manager demonstration
- **[market_scanner.rs](advanced/market_scanner.rs)** - High-volume market scanning
- **[algorithmic_trading.rs](advanced/algorithmic_trading.rs)** - Trading strategy implementation

### ⚡ [Performance Examples](performance/)

Performance optimization and benchmarking:

- **[performance_demo.rs](performance/performance_demo.rs)** - Performance testing and metrics
- **[message_flow_test.rs](performance/message_flow_test.rs)** - Message flow analysis
- **[high_frequency.rs](performance/high_frequency.rs)** - High-frequency data processing
- **[load_test.rs](performance/load_test.rs)** - Stress testing with maximum symbols

## 🚀 Running Examples

### Prerequisites

1. **Set environment variables:**
   ```bash
   export KITE_API_KEY=your_api_key
   export KITE_ACCESS_TOKEN=your_access_token
   export RUST_LOG=info  # Optional: Enable logging
   ```

2. **Get instrument tokens** for the symbols you want to track.

### Basic Examples

```bash
# Runtime subscription management
cargo run --example basic/runtime_subscription_example

# Simple portfolio monitoring
cargo run --example basic/portfolio_monitor

# Single connection demo
cargo run --example basic/single_connection
```

### Advanced Examples

```bash
# Complete dynamic subscription demo
cargo run --example advanced/dynamic_subscription_demo

# Multi-connection manager
cargo run --example advanced/manager_demo

# Market scanning with 9000 symbols
cargo run --example advanced/market_scanner
```

### Performance Examples

```bash
# Performance benchmarking
cargo run --example performance/performance_demo

# Message flow analysis
cargo run --example performance/message_flow_test

# High-frequency processing
cargo run --example performance/high_frequency
```

## 📖 Example Usage Patterns

### Quick Start Pattern

```rust
use kiteticker_async::{KiteTickerManager, KiteManagerConfig, Mode};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Setup
    let config = KiteManagerConfig::default();
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // Subscribe
    let symbols = vec![256265, 408065]; // NIFTY 50, HDFC Bank
    manager.subscribe_symbols(&symbols, Some(Mode::LTP)).await?;
    
    // Process data
    let channels = manager.get_all_channels();
    // ... handle data streams
    
    Ok(())
}
```

### Dynamic Management Pattern

```rust
// Add symbols at runtime
manager.subscribe_symbols(&[738561, 5633], Some(Mode::Quote)).await?;

// Change subscription mode
manager.change_mode(&[256265], Mode::Full).await?;

// Remove symbols
manager.unsubscribe_symbols(&[408065]).await?;

// Check distribution
let distribution = manager.get_symbol_distribution();
```

### High-Performance Pattern

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

## 📊 Example Comparison

| **Example** | **Connections** | **Symbols** | **Features** | **Difficulty** |
|-------------|----------------|-------------|--------------|----------------|
| `single_connection` | 1 | 1-10 | Basic WebSocket | 🟢 Beginner |
| `portfolio_monitor` | 1 | 10-50 | Portfolio tracking | 🟢 Beginner |
| `runtime_subscription` | 3 | 50-100 | Dynamic management | 🟡 Intermediate |
| `dynamic_subscription_demo` | 3 | 100+ | Complete workflow | 🟡 Intermediate |
| `manager_demo` | 3 | 200+ | Multi-connection | 🟡 Intermediate |
| `market_scanner` | 3 | 1000+ | High volume | 🔴 Advanced |
| `performance_demo` | 3 | 3000+ | Benchmarking | 🔴 Advanced |
| `high_frequency` | 3 | 9000 | Maximum capacity | 🔴 Expert |

## 🔧 Configuration Examples

### Development Configuration

```rust
let dev_config = KiteManagerConfig {
    max_connections: 1,
    max_symbols_per_connection: 100,
    connection_buffer_size: 1000,
    parser_buffer_size: 2000,
    enable_dedicated_parsers: true,
    default_mode: Mode::Full,
    ..Default::default()
};
```

### Production Configuration

```rust
let prod_config = KiteManagerConfig {
    max_connections: 3,
    max_symbols_per_connection: 3000,
    connection_buffer_size: 15000,
    parser_buffer_size: 30000,
    enable_dedicated_parsers: true,
    default_mode: Mode::LTP,
    ..Default::default()
};
```

## 📚 Learning Path

1. **Start with** `basic/single_connection.rs` - Understand WebSocket basics
2. **Progress to** `basic/portfolio_monitor.rs` - Learn data processing
3. **Try** `basic/runtime_subscription_example.rs` - Dynamic operations
4. **Advance to** `advanced/manager_demo.rs` - Multi-connection setup
5. **Master** `advanced/dynamic_subscription_demo.rs` - Complete workflow
6. **Optimize with** `performance/performance_demo.rs` - Performance tuning

## 🆘 Troubleshooting

### Common Issues

1. **Missing credentials**: Set `KITE_API_KEY` and `KITE_ACCESS_TOKEN`
2. **Invalid tokens**: Verify instrument tokens are correct
3. **Connection timeouts**: Increase `connection_timeout` in config
4. **Message drops**: Increase buffer sizes

### Debug Mode

```bash
export RUST_LOG=kiteticker_async=debug
cargo run --example basic/runtime_subscription_example
```

## 🔗 Additional Resources

- **[Getting Started Guide](../docs/guides/getting-started.md)**
- **[API Documentation](../docs/api/)**
- **[Dynamic Subscription Guide](../docs/guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)**
- **[Performance Guide](../docs/guides/PERFORMANCE_IMPROVEMENTS.md)**
