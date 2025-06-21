#![allow(
  clippy::cognitive_complexity,
  clippy::large_enum_variant,
  clippy::needless_doctest_main
)]
#![warn(missing_debug_implementations, rust_2018_idioms, unreachable_pub)]
#![doc(test(
  no_crate_inject,
  attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! # KiteTicker Async Manager
//! 
//! High-performance async WebSocket client for the [Kite Connect API](https://kite.trade/docs/connect/v3/websocket/#websocket-streaming) 
//! with multi-connection support and dynamic subscription management.
//!
//! ## Features
//!
//! - **🚀 Multi-Connection Support** - Utilize all 3 allowed WebSocket connections (9,000 symbol capacity)
//! - **⚡ High Performance** - Dedicated parser tasks, optimized buffers, sub-microsecond latency
//! - **🔄 Dynamic Subscriptions** - Add/remove symbols at runtime without reconnection
//! - **📊 Load Balancing** - Automatic symbol distribution across connections
//! - **💪 Production Ready** - Comprehensive error handling, health monitoring, reconnection
//! - **🔧 Async-First Design** - Built with Tokio, follows Rust async best practices
//!
//! ## Quick Start
//!
//! ### Multi-Connection Manager (Recommended)
//!
//! ```rust,no_run
//! use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), String> {
//!     // Setup credentials
//!     let api_key = std::env::var("KITE_API_KEY").unwrap();
//!     let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
//!     
//!     // Create high-performance manager
//!     let config = KiteManagerConfig {
//!         max_connections: 3,
//!         max_symbols_per_connection: 3000,
//!         enable_dedicated_parsers: true,
//!         default_mode: Mode::LTP,
//!         ..Default::default()
//!     };
//!     
//!     // Start manager
//!     let mut manager = KiteTickerManager::new(api_key, access_token, config);
//!     manager.start().await?;
//!     
//!     // Subscribe to symbols (automatically distributed)
//!     let symbols = vec![256265, 408065, 738561]; // NIFTY 50, HDFC Bank, Reliance
//!     manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
//!     
//!     // Process data from independent channels
//!     let channels = manager.get_all_channels();
//!     for (channel_id, mut receiver) in channels {
//!         tokio::spawn(async move {
//!             while let Ok(message) = receiver.recv().await {
//!                 if let TickerMessage::Ticks(ticks) = message {
//!                     for tick in ticks {
//!                         println!("Channel {:?}: {} @ ₹{:.2}",
//!                             channel_id, 
//!                             tick.instrument_token,
//!                             tick.content.last_price.unwrap_or(0.0));
//!                     }
//!                 }
//!             }
//!         });
//!     }
//!     
//!     // Dynamic operations
//!     manager.subscribe_symbols(&[5633, 884737], Some(Mode::Full)).await?;  // Add
//!     manager.unsubscribe_symbols(&[408065]).await?;                        // Remove
//!     manager.change_mode(&[256265], Mode::Full).await?;                    // Change mode
//!     
//!     Ok(())
//! }
//! ```
//!
//!
//! #[tokio::main]
//! async fn main() -> Result<(), String> {
//!     let api_key = std::env::var("KITE_API_KEY").unwrap();
//!     let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
//!     
//!     // Connect to WebSocket
//!     let ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;
//!     
//!     // Subscribe to symbols
//!     let symbols = vec![256265, 408065]; // NIFTY 50, HDFC Bank
//!     let mut subscriber = ticker.subscribe(&symbols, Some(Mode::LTP)).await?;
//!     
//!     // Receive data
//!     while let Ok(Some(message)) = subscriber.next_message().await {
//!         if let TickerMessage::Ticks(ticks) = message {
//!             for tick in ticks {
//!                 println!("Symbol {}: ₹{:.2}", 
//!                     tick.instrument_token,
//!                     tick.content.last_price.unwrap_or(0.0));
//!             }
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Performance Comparison
//!
//! | Feature | Single Connection | Multi-Connection Manager | Improvement |
//! |---------|------------------|---------------------------|-------------|
//! | **Max Symbols** | 3,000 | 9,000 | **3x capacity** |
//! | **Throughput** | Limited by 1 connection | 3 parallel connections | **3x throughput** |
//! | **Latency** | ~5-10µs | ~1-2µs | **5x faster** |
//! | **Resilience** | Single point of failure | 3 independent connections | **High availability** |
//! | **Dynamic Ops** | Manual reconnection | Runtime add/remove | **Zero downtime** |
//!
//! ## Architecture
//!
//! The library provides two main components:
//!
//! ### 1. [`KiteTickerAsync`] - Single WebSocket Connection
//! - Direct WebSocket client for simple use cases
//! - Up to 3,000 symbols per connection
//! - Manual connection management
//!
//! ### 2. [`KiteTickerManager`] - Multi-Connection Manager (Recommended)
//! - Manages up to 3 WebSocket connections automatically
//! - Supports up to 9,000 symbols total
//! - Dynamic subscription management
//! - Load balancing and health monitoring
//! - High-performance optimizations
//!
//! ## Subscription Modes
//!
//! The library supports three subscription modes:
//!
//! - **[`Mode::LTP`]** - Last traded price only (minimal bandwidth)
//! - **[`Mode::Quote`]** - Price + volume + OHLC (standard data)
//! - **[`Mode::Full`]** - Complete market depth (maximum data)
//!
//! ## Examples
//!
//! See the [examples directory](https://github.com/kaychaks/kiteticker-async/tree/master/examples) for:
//!
//! - **Basic Examples** - Simple usage patterns
//! - **Advanced Examples** - Complex multi-connection scenarios  
//! - **Performance Examples** - Optimization and benchmarking
//!
//! ## Documentation
//!
//! - [Getting Started Guide](https://github.com/kaychaks/kiteticker-async/blob/master/docs/guides/getting-started.md)
//! - [API Reference](https://github.com/kaychaks/kiteticker-async/blob/master/docs/api/)
//! - [Dynamic Subscriptions](https://github.com/kaychaks/kiteticker-async/blob/master/docs/guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)
//! - [Performance Guide](https://github.com/kaychaks/kiteticker-async/blob/master/docs/guides/PERFORMANCE_IMPROVEMENTS.md) {
//!     match msg {
//!       TickerMessage::Ticks(ticks) => {
//!         let tick = ticks.first().unwrap();
//!         println!("Received tick for instrument_token {}, {:?}", tick.instrument_token, tick);
//!         break;
//!       },
//!      _ => continue,
//!     }
//!   }
//!  }
//!
//!   Ok(())
//! }
//! ```
mod errors;
mod models;
mod parser;
pub mod manager;
pub use errors::ParseTickError;
pub use models::{
  Depth, DepthItem, Exchange, Mode, Order, OrderStatus, OrderTransactionType,
  OrderValidity, Request, TextMessage, Tick, TickMessage, TickerMessage, OHLC,
};

pub mod ticker;
pub use ticker::{KiteTickerAsync, KiteTickerSubscriber};
pub use manager::{KiteTickerManager, KiteManagerConfig, ChannelId, ManagerStats, HealthSummary};
