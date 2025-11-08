# Multi-API Manager Guide

The `MultiApiKiteTickerManager` enables managing multiple Kite Connect API credentials simultaneously within a single manager instance.

## Overview

### Key Features

- **Multiple API Keys**: Manage multiple Kite Connect accounts from a single manager
- **Per-API Connection Pools**: Each API key maintains up to 3 WebSocket connections
- **Flexible Distribution**: Round-robin or manual symbol assignment
- **Unified Message Stream**: All messages available through a single channel with API key identification
- **Aggregate Monitoring**: Track health and statistics across all API keys
- **Backward Compatible**: Works alongside existing single-API manager

### Use Cases

1. **Multi-Account Trading**: Monitor and trade across multiple trading accounts
2. **High-Volume Subscriptions**: Distribute 18,000+ symbols across multiple API keys
3. **Organizational Isolation**: Separate symbols by department or strategy
4. **Redundancy**: Use multiple API keys for failover scenarios

## Quick Start

### Basic Setup

```rust
use kiteticker_async_manager::{
    MultiApiKiteTickerManager,
    Mode,
    DistributionStrategy,
};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Create manager with multiple API keys
    let mut manager = MultiApiKiteTickerManager::builder()
        .add_api_key("account1", "api_key_1", "access_token_1")
        .add_api_key("account2", "api_key_2", "access_token_2")
        .max_connections_per_api(3)
        .distribution_strategy(DistributionStrategy::RoundRobin)
        .build();

    // Start all connections
    manager.start().await?;

    // Subscribe symbols (auto-distributed)
    let symbols = vec![256265, 408065, 738561];
    manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;

    // Receive messages from all API keys
    let mut unified_channel = manager.get_unified_channel();
    while let Ok((api_key_id, message)) = unified_channel.recv().await {
        println!("From {}: {:?}", api_key_id.0, message);
    }

    Ok(())
}
```

## Configuration

### Builder Pattern

The `MultiApiKiteTickerManagerBuilder` provides a fluent API for configuration:

```rust
let mut manager = MultiApiKiteTickerManager::builder()
    // Add API keys
    .add_api_key("primary", "api_key_1", "token_1")
    .add_api_key("secondary", "api_key_2", "token_2")
    
    // Connection settings
    .max_connections_per_api(3)
    .max_symbols_per_connection(3000)
    .connection_timeout(Duration::from_secs(30))
    
    // Distribution strategy
    .distribution_strategy(DistributionStrategy::RoundRobin)
    
    // Subscription defaults
    .default_mode(Mode::Quote)
    
    // Monitoring
    .enable_health_monitoring(true)
    .health_check_interval(Duration::from_secs(10))
    
    .build();
```

### Distribution Strategies

#### Round-Robin (Default)

Automatically distributes symbols evenly across all API keys:

```rust
let mut manager = MultiApiKiteTickerManager::builder()
    .add_api_key("account1", api_key_1, token_1)
    .add_api_key("account2", api_key_2, token_2)
    .distribution_strategy(DistributionStrategy::RoundRobin)
    .build();

manager.start().await?;

// Symbols automatically distributed across both API keys
manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
```

#### Manual Assignment

Explicitly assign symbols to specific API keys:

```rust
let mut manager = MultiApiKiteTickerManager::builder()
    .add_api_key("account1", api_key_1, token_1)
    .add_api_key("account2", api_key_2, token_2)
    .distribution_strategy(DistributionStrategy::Manual)
    .build();

manager.start().await?;

// Assign specific symbols to specific API keys
manager.subscribe_symbols_to_api("account1", &nifty_symbols, Some(Mode::Full)).await?;
manager.subscribe_symbols_to_api("account2", &stock_symbols, Some(Mode::LTP)).await?;
```

## Symbol Management

### Subscribe Symbols

#### Auto-Distribution (Round-Robin Only)

```rust
// Automatically distribute across all API keys
let symbols = vec![256265, 408065, 738561];
manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
```

#### Manual Assignment

```rust
// Assign to specific API key
manager.subscribe_symbols_to_api(
    "account1", 
    &symbols, 
    Some(Mode::Quote)
).await?;
```

### Unsubscribe Symbols

```rust
// Unsubscribe regardless of which API key owns them
manager.unsubscribe_symbols(&symbols).await?;
```

### Change Mode

```rust
// Change mode for symbols across all API keys
manager.change_mode(&symbols, Mode::Full).await?;
```

## Message Handling

### Unified Channel

Receive messages from all API keys through a single channel:

```rust
let mut unified_channel = manager.get_unified_channel();

while let Ok((api_key_id, message)) = unified_channel.recv().await {
    match message {
        TickerMessage::Ticks(ticks) => {
            println!("API Key {}: Received {} ticks", api_key_id.0, ticks.len());
            for tick in ticks {
                println!("  Token: {}, LTP: {:?}", 
                    tick.instrument_token, 
                    tick.content.last_price);
            }
        }
        TickerMessage::Error(err) => {
            eprintln!("API Key {}: Error - {}", api_key_id.0, err);
        }
        _ => {}
    }
}
```

### Per-API Channel

Get channel for a specific API key and connection:

```rust
use kiteticker_async_manager::ChannelId;

// Get channel for specific API key's first connection
if let Some(mut channel) = manager.get_channel("account1", ChannelId::Connection1) {
    while let Ok(message) = channel.recv().await {
        println!("Account1/Connection1: {:?}", message);
    }
}
```

## Monitoring

### Aggregate Statistics

```rust
let stats = manager.get_stats().await;

println!("Total API Keys: {}", stats.total_api_keys);
println!("Total Connections: {}", stats.total_connections);
println!("Total Symbols: {}", stats.total_symbols);
println!("Total Messages: {}", stats.total_messages_received);
println!("Uptime: {:?}", stats.uptime);

// Per-API statistics
for api_stat in stats.per_api_stats {
    println!("\nAPI Key: {}", api_stat.api_key_id);
    println!("  Active Connections: {}", api_stat.active_connections);
    println!("  Symbols: {}", api_stat.total_symbols);
    println!("  Messages: {}", api_stat.total_messages_received);
}
```

### Per-API Statistics

```rust
let api_stats = manager.get_api_stats("account1").await?;

println!("API Key: {}", api_stats.api_key_id);
println!("Active Connections: {}", api_stats.active_connections);
println!("Total Symbols: {}", api_stats.total_symbols);
```

### Symbol Distribution

View how symbols are distributed across API keys and connections:

```rust
let distribution = manager.get_symbol_distribution();

for (api_key_id, conn_map) in distribution {
    println!("API Key: {}", api_key_id.0);
    for (conn_idx, symbols) in conn_map {
        println!("  Connection {}: {:?}", conn_idx, symbols);
    }
}
```

## Advanced Usage

### Capacity Planning

Each API key can handle:
- **3 connections** (Kite limit)
- **3,000 symbols per connection**
- **9,000 total symbols per API key**

With multiple API keys:
```rust
// 2 API keys = 18,000 symbols capacity
// 3 API keys = 27,000 symbols capacity
// etc.
```

### Error Handling

```rust
match manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await {
    Ok(_) => println!("Subscribed successfully"),
    Err(e) if e.contains("capacity") => {
        println!("All API keys at capacity, need to add more");
    }
    Err(e) => eprintln!("Subscription error: {}", e),
}
```

### Graceful Shutdown

```rust
// Stop all connections and clean up
manager.stop().await?;
```

## Best Practices

### 1. API Key Identification

Use meaningful identifiers for API keys:

```rust
.add_api_key("production_primary", api_key, token)
.add_api_key("production_backup", api_key_2, token_2)
.add_api_key("development", api_key_3, token_3)
```

### 2. Distribution Strategy Selection

- Use **RoundRobin** for balanced load across API keys
- Use **Manual** for organizational isolation or specific routing needs

### 3. Monitoring

Regularly check statistics to ensure balanced distribution:

```rust
// Periodic stats check
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let stats = manager.get_stats().await;
        log::info!("Stats: {:?}", stats);
    }
});
```

### 4. Resource Management

- Start with minimum required API keys
- Add more API keys only when approaching capacity
- Monitor connection health per API key

## Examples

See the complete working example at:
- `examples/multi_api_demo.rs`

Run with:
```bash
export KITE_API_KEY_1="your_api_key_1"
export KITE_ACCESS_TOKEN_1="your_token_1"
export KITE_API_KEY_2="your_api_key_2"
export KITE_ACCESS_TOKEN_2="your_token_2"

cargo run --example multi_api_demo
```

## Comparison: Single vs Multi-API

| Feature | Single-API Manager | Multi-API Manager |
|---------|-------------------|-------------------|
| **Max Symbols** | 9,000 | Unlimited (9,000 × N API keys) |
| **API Keys** | 1 | Multiple |
| **Message Channels** | Per-connection | Unified + Per-connection |
| **Symbol Assignment** | Automatic | Automatic or Manual |
| **Use Case** | Single account | Multiple accounts, high volume |

## Troubleshooting

### Issue: "All API keys are at capacity"

**Solution**: Add more API keys or reduce symbol count

```rust
builder.add_api_key("additional_account", api_key, token)
```

### Issue: "Cannot use auto-subscribe with Manual distribution"

**Solution**: Use `subscribe_symbols_to_api` instead:

```rust
manager.subscribe_symbols_to_api("account1", &symbols, mode).await?;
```

### Issue: Messages from one API key are missing

**Check**:
1. Connection health per API key
2. Symbol distribution
3. Per-API statistics

```rust
let stats = manager.get_api_stats("account1").await?;
println!("Health: {:?}", stats);
```

## Migration from Single-API Manager

Existing code using `KiteTickerManager` works unchanged. To migrate to multi-API:

```rust
// Before: Single-API
let mut manager = KiteTickerManager::new(api_key, token, config);

// After: Multi-API
let mut manager = MultiApiKiteTickerManager::builder()
    .add_api_key("main", api_key, token)
    .base_config(config)
    .build();

// API remains similar
manager.start().await?;
manager.subscribe_symbols(&symbols, mode).await?;

// Use unified channel instead of per-connection channels
let mut unified = manager.get_unified_channel();
```

## API Reference

See [multi_api_manager.rs](../../src/manager/multi_api_manager.rs) for complete API documentation.
