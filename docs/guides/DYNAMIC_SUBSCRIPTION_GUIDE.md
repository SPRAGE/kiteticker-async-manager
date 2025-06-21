# Dynamic Symbol Subscription/Unsubscription Guide

## Overview

**YES, dynamic symbol subscription and unsubscription is fully supported!** Your KiteTickerManager can handle runtime symbol management efficiently across all 3 WebSocket connections, with each supporting up to 3000 symbols (9000 total capacity).

## Key Features

### ✅ **Runtime Symbol Management**
- Add new symbols without restarting connections
- Remove symbols dynamically 
- Change subscription modes for existing symbols
- Automatic load balancing across connections

### ✅ **Intelligent Distribution**
- Round-robin allocation across 3 connections
- Automatic capacity management (3000 symbols per connection)
- Real-time symbol tracking and mapping

### ✅ **Efficient Operations**
- No connection restarts required
- Minimal network overhead
- Batch operations support
- Real-time capacity monitoring

## API Methods

### 1. **Subscribe to New Symbols**
```rust
// Add symbols with specific mode
manager.subscribe_symbols(&[408065, 884737, 738561], Some(Mode::Full)).await?;

// Add symbols with default mode
manager.subscribe_symbols(&symbols, None).await?;
```

### 2. **Unsubscribe from Symbols**
```rust
// Remove specific symbols
manager.unsubscribe_symbols(&[408065, 884737]).await?;

// Remove all symbols (pass empty slice to each connection's unsubscribe)
let all_symbols: Vec<u32> = manager.get_symbol_distribution()
    .values()
    .flatten()
    .cloned()
    .collect();
manager.unsubscribe_symbols(&all_symbols).await?;
```

### 3. **Change Subscription Mode**
```rust
// Upgrade existing symbols to Full mode
manager.change_mode(&[408065, 884737], Mode::Full).await?;

// Downgrade to LTP mode for efficiency
manager.change_mode(&symbols, Mode::LTP).await?;
```

### 4. **Monitor Distribution**
```rust
let distribution = manager.get_symbol_distribution();
for (channel_id, symbols) in &distribution {
    println!("Connection {:?}: {} symbols", channel_id, symbols.len());
}
```

## Complete Example

```rust
use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize manager
    let config = KiteManagerConfig {
        max_symbols_per_connection: 3000,
        max_connections: 3,
        // ... other config
        default_mode: Mode::LTP,
    };
    
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // 1. Initial subscription
    let nifty_stocks = vec![408065, 884737, 738561];
    manager.subscribe_symbols(&nifty_stocks, Some(Mode::LTP)).await?;
    
    // 2. Add more symbols dynamically
    let bank_stocks = vec![341249, 492033, 779521];
    manager.subscribe_symbols(&bank_stocks, Some(Mode::Quote)).await?;
    
    // 3. Change mode for existing symbols
    manager.change_mode(&nifty_stocks, Mode::Full).await?;
    
    // 4. Remove some symbols
    manager.unsubscribe_symbols(&bank_stocks).await?;
    
    // 5. Monitor distribution
    let distribution = manager.get_symbol_distribution();
    println!("Current distribution: {:?}", distribution);
    
    Ok(())
}
```

## Capacity Management

### **Connection Limits**
- **Per Connection**: 3000 symbols maximum
- **Total Capacity**: 9000 symbols across 3 connections
- **Automatic Distribution**: Round-robin allocation

### **Efficient Strategies**

#### 1. **Mode Optimization**
```rust
// Start with LTP for maximum capacity
manager.subscribe_symbols(&large_symbol_list, Some(Mode::LTP)).await?;

// Upgrade only critical symbols to Full mode
manager.change_mode(&critical_symbols, Mode::Full).await?;
```

#### 2. **Batch Operations**
```rust
// Add symbols in batches
for chunk in symbols.chunks(100) {
    manager.subscribe_symbols(chunk, Some(Mode::LTP)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await; // Rate limiting
}
```

#### 3. **Dynamic Watchlist Management**
```rust
async fn rotate_watchlist(
    manager: &mut KiteTickerManager,
    old_symbols: &[u32],
    new_symbols: &[u32],
) -> Result<(), String> {
    // Remove old symbols
    manager.unsubscribe_symbols(old_symbols).await?;
    
    // Add new symbols
    manager.subscribe_symbols(new_symbols, Some(Mode::LTP)).await?;
    
    Ok(())
}
```

## Real-World Use Cases

### 1. **Algorithmic Trading**
```rust
// Dynamic watchlist based on market conditions
async fn update_trading_watchlist(
    manager: &mut KiteTickerManager,
    market_scanner_results: Vec<u32>,
) -> Result<(), String> {
    // Get current symbols
    let current_symbols: Vec<u32> = manager.get_symbol_distribution()
        .values()
        .flatten()
        .cloned()
        .collect();
    
    // Calculate changes
    let to_remove: Vec<u32> = current_symbols.iter()
        .filter(|&symbol| !market_scanner_results.contains(symbol))
        .cloned()
        .collect();
    
    let to_add: Vec<u32> = market_scanner_results.iter()
        .filter(|&symbol| !current_symbols.contains(symbol))
        .cloned()
        .collect();
    
    // Apply changes
    if !to_remove.is_empty() {
        manager.unsubscribe_symbols(&to_remove).await?;
    }
    
    if !to_add.is_empty() {
        manager.subscribe_symbols(&to_add, Some(Mode::LTP)).await?;
    }
    
    Ok(())
}
```

### 2. **Event-Driven Subscriptions**
```rust
// Subscribe to stocks based on news events
async fn subscribe_to_news_stocks(
    manager: &mut KiteTickerManager,
    news_symbols: Vec<u32>,
) -> Result<(), String> {
    // Upgrade news-related stocks to Full mode for detailed data
    manager.subscribe_symbols(&news_symbols, Some(Mode::Full)).await?;
    
    // Schedule removal after news impact period
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_hours(2)).await;
        // Remove or downgrade after 2 hours
        manager.change_mode(&news_symbols, Mode::LTP).await.ok();
    });
    
    Ok(())
}
```

### 3. **Time-Based Rotation**
```rust
// Rotate symbols based on trading sessions
async fn session_based_subscription(
    manager: &mut KiteTickerManager,
) -> Result<(), String> {
    loop {
        let now = chrono::Utc::now().time();
        
        if now.hour() >= 9 && now.hour() < 12 {
            // Morning session: Focus on large caps
            let large_caps = vec![408065, 884737, 738561];
            manager.subscribe_symbols(&large_caps, Some(Mode::Full)).await?;
        } else if now.hour() >= 12 && now.hour() < 15 {
            // Afternoon session: Add mid caps
            let mid_caps = vec![341249, 492033, 779521];
            manager.subscribe_symbols(&mid_caps, Some(Mode::Quote)).await?;
        }
        
        tokio::time::sleep(Duration::from_secs(3600)).await; // Check hourly
    }
}
```

## Performance Tips

### 1. **Minimize Mode Changes**
- Start with `Mode::LTP` for maximum capacity
- Upgrade to `Mode::Full` only when needed
- Use `Mode::Quote` as middle ground

### 2. **Batch Operations**
- Group symbol additions/removals
- Use reasonable delays between operations
- Monitor connection health

### 3. **Monitor Capacity**
```rust
fn check_capacity(manager: &KiteTickerManager) -> (usize, usize) {
    let distribution = manager.get_symbol_distribution();
    let used: usize = distribution.values().map(|v| v.len()).sum();
    let total_capacity = 3000 * 3;
    (used, total_capacity - used)
}
```

### 4. **Graceful Error Handling**
```rust
async fn safe_subscribe(
    manager: &mut KiteTickerManager,
    symbols: &[u32],
    mode: Mode,
) -> Result<(), String> {
    let (used, remaining) = check_capacity(manager);
    
    if symbols.len() > remaining {
        return Err(format!(
            "Cannot add {} symbols, only {} slots available", 
            symbols.len(), 
            remaining
        ));
    }
    
    manager.subscribe_symbols(symbols, Some(mode)).await
}
```

## Testing Dynamic Operations

Run the demo:
```bash
export KITE_API_KEY=your_api_key
export KITE_ACCESS_TOKEN=your_access_token
export RUST_LOG=info
cargo run --example dynamic_subscription_demo
```

The demo will show:
1. Initial symbol subscription
2. Dynamic symbol addition
3. Mode changes
4. Symbol removal
5. Capacity management
6. Load balancing across connections

## Summary

Dynamic subscription/unsubscription is **fully implemented and highly efficient**. The system handles:

- ✅ **Runtime symbol management** without connection restarts
- ✅ **Intelligent load balancing** across 3 connections  
- ✅ **9000 symbol capacity** (3000 per connection)
- ✅ **Mode changes** for existing symbols
- ✅ **Real-time monitoring** and capacity management
- ✅ **Batch operations** for efficiency
- ✅ **Graceful error handling** and recovery

This makes it perfect for algorithmic trading, market scanning, and any application requiring dynamic market data subscriptions.
