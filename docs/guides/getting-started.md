# Getting Started with KiteTicker Async

A comprehensive guide to get you up and running with the KiteTicker Async library.

## 📋 Table of Contents

- [Installation](#installation)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Basic Examples](#basic-examples)
- [Advanced Usage](#advanced-usage)
- [Common Patterns](#common-patterns)
- [Troubleshooting](#troubleshooting)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
kiteticker-async-manager = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Prerequisites

### 1. Kite Connect Account

You'll need:
- **API Key** - From your Kite Connect app
- **Access Token** - Generated via Kite Connect login flow

### 2. Environment Setup

Create a `.env` file:

```bash
KITE_API_KEY=your_api_key_here
KITE_ACCESS_TOKEN=your_access_token_here
RUST_LOG=info  # Optional: Enable logging
```

### 3. Instrument Tokens

Get instrument tokens for the symbols you want to track:
- **NIFTY 50**: `256265`
- **HDFC Bank**: `408065`
- **Reliance**: `738561`
- **TCS**: `5633`

## Quick Start

### Single Connection Example

```rust
use kiteticker_async_manager::{KiteTickerAsync, Mode, TickerMessage};

#[tokio::main]
async fn main() -> Result<(), String> {
    let api_key = std::env::var("KITE_API_KEY").unwrap();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
    
    // Connect to WebSocket
    let ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;
    
    // Subscribe to symbols
    let symbols = vec![256265, 408065]; // NIFTY 50, HDFC Bank
    let mut subscriber = ticker.subscribe(&symbols, Some(Mode::LTP)).await?;
    
    // Receive data
    while let Ok(Some(message)) = subscriber.next_message().await {
        if let TickerMessage::Ticks(ticks) = message {
            for tick in ticks {
                println!("Symbol {}: Price {}", 
                    tick.instrument_token,
                    tick.content.last_price.unwrap_or(0.0));
            }
        }
    }
    
    Ok(())
}
```

### Multi-Connection Manager (Recommended)

```rust
use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), String> {
    let api_key = std::env::var("KITE_API_KEY").unwrap();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
    
    // Configure manager for high performance
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
    
    // Subscribe to symbols (automatically distributed)
    let symbols = vec![256265, 408065, 738561, 5633]; // Multiple stocks
    manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
    
    // Get data channels
    let channels = manager.get_all_channels();
    let mut tasks = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let task = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    for tick in ticks {
                        println!("Channel {:?} - Symbol {}: {} @ {}",
                            channel_id,
                            tick.instrument_token,
                            tick.content.last_price.unwrap_or(0.0),
                            tick.content.volume.unwrap_or(0));
                    }
                }
            }
        });
        tasks.push(task);
    }
    
    // Wait for tasks (or implement your own logic)
    for task in tasks {
        let _ = task.await;
    }
    
    Ok(())
}
```

## Basic Examples

### 1. LTP Monitoring

Monitor last traded prices for a portfolio:

```rust
use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode};

async fn monitor_portfolio() -> Result<(), String> {
    let mut manager = create_manager().await?;
    
    // Portfolio symbols
    let portfolio = vec![
        256265, // NIFTY 50
        408065, // HDFC Bank  
        738561, // Reliance
        5633,   // TCS
        884737, // Asian Paints
    ];
    
    manager.subscribe_symbols(&portfolio, Some(Mode::LTP)).await?;
    
    let channels = manager.get_all_channels();
    for (_, mut receiver) in channels {
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    for tick in ticks {
                        if let Some(price) = tick.content.last_price {
                            println!("🏷️  {} @ ₹{:.2}", tick.instrument_token, price);
                        }
                    }
                }
            }
        });
    }
    
    Ok(())
}
```

### 2. Market Depth Analysis

Analyze order book for specific symbols:

```rust
async fn analyze_market_depth() -> Result<(), String> {
    let mut manager = create_manager().await?;
    
    // Subscribe with Full mode for market depth
    let symbols = vec![256265, 408065]; // NIFTY, HDFC
    manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;
    
    let channels = manager.get_all_channels();
    for (_, mut receiver) in channels {
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    for tick in ticks {
                        if let Some(depth) = &tick.content.depth {
                            analyze_order_book(tick.instrument_token, depth);
                        }
                    }
                }
            }
        });
    }
    
    Ok(())
}

fn analyze_order_book(symbol: u32, depth: &MarketDepth) {
    if let (Some(best_bid), Some(best_ask)) = (depth.buy.first(), depth.sell.first()) {
        let spread = best_ask.price - best_bid.price;
        println!("📊 {}: Spread ₹{:.2} ({}@{} | {}@{})",
            symbol, spread,
            best_bid.quantity, best_bid.price,
            best_ask.quantity, best_ask.price);
    }
}
```

### 3. Dynamic Symbol Management

Add and remove symbols at runtime:

```rust
async fn dynamic_trading_strategy() -> Result<(), String> {
    let mut manager = create_manager().await?;
    
    // Start with core holdings
    let core_symbols = vec![256265, 408065]; // NIFTY, HDFC
    manager.subscribe_symbols(&core_symbols, Some(Mode::Quote)).await?;
    
    // Add symbols based on market conditions
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Add trending stocks
        let trending = vec![738561, 5633]; // Reliance, TCS
        manager.subscribe_symbols(&trending, Some(Mode::Quote)).await.unwrap();
        println!("✅ Added trending stocks");
        
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        // Remove underperforming stocks
        manager.unsubscribe_symbols(&[408065]).await.unwrap();
        println!("❌ Removed underperforming stock");
        
        // Change mode for detailed analysis
        manager.change_mode(&[256265], Mode::Full).await.unwrap();
        println!("🔄 Enhanced NIFTY to Full mode");
    });
    
    // Process data streams...
    Ok(())
}
```

## Advanced Usage

### High-Performance Configuration

For maximum throughput applications:

```rust
use std::time::Duration;

let high_perf_config = KiteManagerConfig {
    max_connections: 3,                           // Use all connections
    max_symbols_per_connection: 3000,             // Maximum capacity
    connection_buffer_size: 25000,                // Large buffers
    parser_buffer_size: 50000,                    // Prevent drops
    connection_timeout: Duration::from_secs(30),
    health_check_interval: Duration::from_secs(5),
    max_reconnect_attempts: 10,                   // Aggressive reconnect
    reconnect_delay: Duration::from_secs(1),      // Fast recovery
    enable_dedicated_parsers: true,               // CPU optimization
    default_mode: Mode::LTP,                      // Minimal bandwidth
};
```

### Error Handling and Resilience

```rust
use tokio::time::{timeout, Duration};

async fn resilient_connection() -> Result<(), String> {
    let mut manager = create_manager().await?;
    
    // Retry logic for subscriptions
    let symbols = vec![256265, 408065];
    let mut retry_count = 0;
    
    while retry_count < 3 {
        match timeout(
            Duration::from_secs(10),
            manager.subscribe_symbols(&symbols, Some(Mode::LTP))
        ).await {
            Ok(Ok(())) => {
                println!("✅ Subscription successful");
                break;
            }
            Ok(Err(e)) => {
                println!("❌ Subscription failed: {}", e);
                retry_count += 1;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(_) => {
                println!("⏱️ Subscription timeout");
                retry_count += 1;
            }
        }
    }
    
    // Monitor connection health
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            match manager.get_health().await {
                Ok(health) => {
                    if !health.overall_health {
                        println!("⚠️ Connection health degraded: {}/{} healthy",
                            health.healthy_connections, health.total_connections);
                    }
                }
                Err(e) => {
                    println!("❌ Health check failed: {}", e);
                }
            }
        }
    });
    
    Ok(())
}
```

## Common Patterns

### 1. Symbol-Specific Processing

Process different symbols with different logic:

```rust
use std::collections::HashMap;

async fn symbol_specific_processing() {
    let channels = manager.get_all_channels();
    
    for (_, mut receiver) in channels {
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    for tick in ticks {
                        match tick.instrument_token {
                            256265 => process_nifty_tick(&tick),      // NIFTY 50
                            408065 => process_banking_tick(&tick),    // HDFC Bank
                            738561 => process_energy_tick(&tick),     // Reliance
                            _ => process_generic_tick(&tick),
                        }
                    }
                }
            }
        });
    }
}

fn process_nifty_tick(tick: &TickMessage) {
    // Index-specific logic
    if let Some(price) = tick.content.last_price {
        println!("📈 NIFTY: {}", price);
    }
}
```

### 2. Data Aggregation

Aggregate data across multiple symbols:

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MarketData {
    prices: HashMap<u32, f64>,
    volumes: HashMap<u32, u32>,
}

async fn aggregate_market_data() {
    let data = Arc::new(Mutex::new(MarketData::default()));
    let channels = manager.get_all_channels();
    
    for (_, mut receiver) in channels {
        let data_clone = Arc::clone(&data);
        
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    let mut data = data_clone.lock().unwrap();
                    
                    for tick in ticks {
                        if let Some(price) = tick.content.last_price {
                            data.prices.insert(tick.instrument_token, price);
                        }
                        if let Some(volume) = tick.content.volume {
                            data.volumes.insert(tick.instrument_token, volume);
                        }
                    }
                }
            }
        });
    }
    
    // Periodic data analysis
    let data_clone = Arc::clone(&data);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            let data = data_clone.lock().unwrap();
            println!("📊 Current tracking: {} symbols", data.prices.len());
            
            // Perform analysis on aggregated data
            analyze_market_trends(&*data);
        }
    });
}
```

### 3. Performance Monitoring

Monitor library performance:

```rust
async fn monitor_performance() {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // Get processor stats
        let processor_stats = manager.get_processor_stats().await;
        for (channel_id, stats) in processor_stats {
            println!("🔥 {:?}: {:.1} msg/sec, {:?} latency",
                channel_id, stats.messages_per_second, stats.processing_latency_avg);
        }
        
        // Get manager stats
        if let Ok(stats) = manager.get_stats().await {
            println!("📈 Total: {} symbols, {} messages, {} errors",
                stats.total_symbols, stats.total_messages_received, stats.total_errors);
        }
        
        // Check symbol distribution
        let distribution = manager.get_symbol_distribution();
        for (channel_id, symbols) in distribution {
            println!("📊 {:?}: {} symbols", channel_id, symbols.len());
        }
    }
}
```

## Troubleshooting

### Common Issues

#### 1. Authentication Errors

```
Error: Authentication failed
```

**Solution:**
- Verify API key and access token
- Ensure access token is not expired
- Check network connectivity

#### 2. Connection Timeouts

```
Error: Connection timeout
```

**Solution:**
```rust
let config = KiteManagerConfig {
    connection_timeout: Duration::from_secs(60), // Increase timeout
    max_reconnect_attempts: 10,                  // More retries
    ..Default::default()
};
```

#### 3. Message Drops

```
Warning: Parser channel full, dropping message
```

**Solution:**
```rust
let config = KiteManagerConfig {
    connection_buffer_size: 20000, // Increase buffer
    parser_buffer_size: 50000,     // Larger parser buffer
    ..Default::default()
};
```

#### 4. Symbol Not Found

```
Error: Invalid instrument token
```

**Solution:**
- Verify instrument tokens are correct
- Use Kite Connect API to get valid tokens
- Check for expired or delisted instruments

### Debug Mode

Enable detailed logging:

```bash
export RUST_LOG=kiteticker_async_manager=debug
```

### Helper Functions

```rust
// Utility function for creating managers
async fn create_manager() -> Result<KiteTickerManager, String> {
    let api_key = std::env::var("KITE_API_KEY")
        .map_err(|_| "KITE_API_KEY not set")?;
    let access_token = std::env::var("KITE_ACCESS_TOKEN")
        .map_err(|_| "KITE_ACCESS_TOKEN not set")?;
    
    let config = KiteManagerConfig::default();
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    Ok(manager)
}

// Utility for testing with demo symbols
fn get_demo_symbols() -> Vec<u32> {
    vec![
        256265, // NIFTY 50
        408065, // HDFC Bank
        738561, // Reliance
        5633,   // TCS
        884737, // Asian Paints
    ]
}
```

## Next Steps

- **[Dynamic Subscriptions Guide](DYNAMIC_SUBSCRIPTION_GUIDE.md)** - Learn runtime symbol management
- **[API Reference](../api/README.md)** - Complete API documentation
- **[Examples](../examples/)** - More advanced examples
- **[Performance Guide](PERFORMANCE_IMPROVEMENTS.md)** - Optimization techniques
