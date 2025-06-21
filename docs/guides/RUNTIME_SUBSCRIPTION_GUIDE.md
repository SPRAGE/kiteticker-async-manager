# Runtime Subscription Management Guide

## 🔄 Dynamic Operations

### Adding Symbols
```rust
// Add symbols with LTP mode (lightweight)
manager.subscribe_symbols(&[408065, 884737], Some(Mode::LTP)).await?;

// Add symbols with Quote mode (price + volume)
manager.subscribe_symbols(&[775937, 492033], Some(Mode::Quote)).await?;

// Add symbols with Full mode (complete market depth)
manager.subscribe_symbols(&[256265, 265], Some(Mode::Full)).await?;
```

### Removing Symbols
```rust
// Remove specific symbols
let symbols_to_remove = vec![408065, 884737];
manager.unsubscribe_symbols(&symbols_to_remove).await?;

// Remove all symbols (complete cleanup)
let all_symbols: Vec<u32> = manager.get_symbol_distribution()
    .values()
    .flat_map(|symbols| symbols.iter().cloned())
    .collect();
manager.unsubscribe_symbols(&all_symbols).await?;
```

### Changing Modes
```rust
// Upgrade existing symbols to Full mode for deeper data
let symbols_for_upgrade = vec![775937, 492033];
manager.change_mode(&symbols_for_upgrade, Mode::Full).await?;

// Downgrade to LTP for performance
let symbols_for_downgrade = vec![408065, 884737];
manager.change_mode(&symbols_for_downgrade, Mode::LTP).await?;
```

### Monitoring State
```rust
// Check current distribution across connections
let distribution = manager.get_symbol_distribution();
for (channel_id, symbols) in distribution {
    println!("{:?}: {} symbols", channel_id, symbols.len());
}

// Get performance statistics
let stats = manager.get_stats().await?;
println!("Active connections: {}", stats.active_connections);
println!("Total symbols: {}", stats.total_symbols);
println!("Messages received: {}", stats.total_messages_received);
```

## 🚀 Advanced Patterns

### Event-Driven Subscriptions
```rust
// Subscribe based on market events
async fn handle_market_event(event: MarketEvent, manager: &mut KiteTickerManager) -> Result<(), String> {
    match event {
        MarketEvent::HighVolume(symbols) => {
            // Upgrade to Full mode for high-volume stocks
            manager.change_mode(&symbols, Mode::Full).await?;
        }
        MarketEvent::NewWatchlist(symbols) => {
            // Add new symbols to monitoring
            manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
        }
        MarketEvent::RemoveStocks(symbols) => {
            // Remove symbols from monitoring
            manager.unsubscribe_symbols(&symbols).await?;
        }
    }
    Ok(())
}
```

### Time-Based Management
```rust
// Rotate symbols based on time
async fn rotate_symbols(manager: &mut KiteTickerManager) -> Result<(), String> {
    // Remove old symbols
    let old_symbols = vec![256265, 265];
    manager.unsubscribe_symbols(&old_symbols).await?;
    
    // Add new symbols
    let new_symbols = vec![408065, 884737];
    manager.subscribe_symbols(&new_symbols, Some(Mode::Quote)).await?;
    
    Ok(())
}
```

### Capacity Management
```rust
// Smart capacity management
async fn manage_capacity(manager: &mut KiteTickerManager, new_symbols: Vec<u32>) -> Result<(), String> {
    let distribution = manager.get_symbol_distribution();
    let total_symbols: usize = distribution.values().map(|v| v.len()).sum();
    
    // Check if we need to make room
    if total_symbols + new_symbols.len() > 9000 { // Total capacity
        // Remove some lower priority symbols first
        let symbols_to_remove = vec![256265, 265, 256777, 274441, 260105]; // Example symbols
        manager.unsubscribe_symbols(&symbols_to_remove).await?;
    }
    
    // Add new symbols
    manager.subscribe_symbols(&new_symbols, Some(Mode::LTP)).await?;
    Ok(())
}
```

## 💡 Best Practices

1. **Batch Operations**: Group multiple symbol operations together for efficiency
2. **Mode Strategy**: Start with LTP, upgrade to Quote/Full only when needed
3. **Monitoring**: Regularly check distribution and performance stats
4. **Error Handling**: Always handle potential errors from dynamic operations
5. **Capacity Planning**: Monitor total symbol count across all connections

## ⚡ Performance Tips

- Use `Mode::LTP` for basic price tracking (lowest overhead)
- Use `Mode::Quote` for price + volume data
- Use `Mode::Full` only for symbols requiring market depth
- Monitor connection health with `get_stats()`
- Batch symbol operations when possible
