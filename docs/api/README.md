# KiteTicker Async - API Documentation

This directory contains comprehensive API documentation for the KiteTicker Async library.

## 📚 Documentation Structure

- **[Manager API](manager.md)** - Multi-connection manager API reference
- **[Ticker API](ticker.md)** - Core ticker WebSocket client API (raw-only mode, raw subscription)
- **[Configuration](config.md)** - Configuration options and examples
- **[Models](models.md)** - Data structures and message types
- **[Error Handling](errors.md)** - Error types and handling strategies

Tip: Prefer `KiteTickerManagerBuilder` for fluent setup, including `raw_only(true)` to bypass parsing when needed.

## 🚀 Quick Start

```rust
use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode};

// Basic setup with manager
let config = KiteManagerConfig::default();
let mut manager = KiteTickerManager::new(api_key, access_token, config);
manager.start().await?;

// Subscribe to symbols
manager.subscribe_symbols(&[256265, 408065], Some(Mode::Full)).await?;

// Get data channels
let channels = manager.get_all_channels();
```

## 🔗 See Also

- **[Getting Started Guide](../guides/getting-started.md)**
- **[Dynamic Subscriptions](../guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)**
- **[Examples](../examples/)**
