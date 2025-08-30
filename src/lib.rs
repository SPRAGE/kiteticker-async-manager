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
//! Using the builder:
//!
//! ```rust,no_run
//! use kiteticker_async_manager::{KiteTickerManagerBuilder, Mode};
//! #[tokio::main]
//! async fn main() -> Result<(), String> {
//!   let api_key = std::env::var("KITE_API_KEY").unwrap();
//!   let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
//!   let mut manager = KiteTickerManagerBuilder::new(api_key, access_token)
//!       .max_connections(3)
//!       .max_symbols_per_connection(3000)
//!       .raw_only(true) // receive only raw frames if desired
//!       .default_mode(Mode::Quote)
//!       .enable_dedicated_parsers(true)
//!       .build();
//!   manager.start().await?;
//!   Ok(())
//! }
//! ```
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
//! ### Single Connection Usage
//!
//! ```rust,no_run
//! use kiteticker_async_manager::{KiteTickerAsync, Mode, TickerMessage};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), String> {
//!     let api_key = std::env::var("KITE_API_KEY").unwrap();
//!     let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
//!     
//!     // Connect to WebSocket
//!     let mut ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;
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
//! ### 1. `KiteTickerAsync` - Single WebSocket Connection
//! - Direct WebSocket client for simple use cases
//! - Up to 3,000 symbols per connection
//! - Manual connection management
//!
//! ### 2. `KiteTickerManager` - Multi-Connection Manager (Recommended)
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
//! - **`Mode::LTP`** - Last traded price only (minimal bandwidth)
//! - **`Mode::Quote`** - Price + volume + OHLC (standard data)
//! - **`Mode::Full`** - Complete market depth (maximum data)
//!
//! ## Zero-copy raw access (advanced)
//!
//! For maximum throughput and minimal allocations, you can work directly with the raw
//! WebSocket frame bytes and view individual packets using zero-copy, endian-safe types.
//! This is fully safe and avoids undefined behavior by using `zerocopy::Ref` and
//! big-endian field wrappers.
//!
//! Key points:
//! - Subscribe to raw frames via `KiteTickerAsync::subscribe_raw_frames()`, which yields `bytes::Bytes`.
//! - Extract packet bodies (length-prefixed) from a frame and select the size you need.
//! - Use helpers like `as_tick_raw`, `as_index_quote_32`, and `as_inst_header_64` to obtain
//!   `zerocopy::Ref<&[u8], T>` that dereferences to a typed view.
//! - The `Ref` is valid as long as the backing bytes live; examples store `Bytes` to keep it alive.
//!
//! Example (snippets):
//! ```rust,no_run
//! use kiteticker_async_manager::{KiteTickerAsync, Mode, as_tick_raw};
//! use bytes::Bytes;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), String> {
//! let api_key = std::env::var("KITE_API_KEY").unwrap();
//! let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
//! let mut ticker = KiteTickerAsync::connect_with_options(&api_key, &access_token, true).await?;
//! let _sub = ticker.subscribe(&[256265], Some(Mode::Full)).await?;
//! let mut frames = ticker.subscribe_raw_frames();
//!
//! // Receive a frame and pull out a 184-byte Full packet body
//! let frame: Bytes = frames.recv().await.unwrap();
//! let num = u16::from_be_bytes([frame[0], frame[1]]) as usize;
//! let mut off = 2usize;
//! for _ in 0..num {
//!   let len = u16::from_be_bytes([frame[off], frame[off+1]]) as usize;
//!   let body = frame.slice(off+2..off+2+len);
//!   if len == 184 {
//!     if let Some(view_ref) = as_tick_raw(&body) {
//!       let tick = &*view_ref; // &TickRaw
//!       let token = tick.header.instrument_token.get();
//!       let ltp_scaled = tick.header.last_price.get();
//!       // ... use fields ...
//!     }
//!   }
//!   off += 2 + len;
//! }
//! # Ok(()) }
//! ```
//!
//! Safety model: all raw structs derive `Unaligned` and use `big_endian` wrappers for integer fields.
//! `as_*` helpers return `Option<zerocopy::Ref<&[u8], T>>` which validates size and alignment. No `unsafe` is required.
//!
//! ## Examples
//!
//! See the [examples directory](https://github.com/SPRAGE/kiteticker-async-manager/tree/main/examples) for:
//!
//! - **Basic Examples** - Simple usage patterns
//! - **Advanced Examples** - Complex multi-connection scenarios  
//! - **Performance Examples** - Optimization and benchmarking
//!
//! ## Documentation
//!
//! - [Getting Started Guide](https://github.com/SPRAGE/kiteticker-async-manager/blob/main/docs/guides/getting-started.md)
//! - [API Reference](https://github.com/SPRAGE/kiteticker-async-manager/tree/main/docs/api)
//! - [Dynamic Subscriptions](https://github.com/SPRAGE/kiteticker-async-manager/blob/main/docs/guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)
//! - [Performance Guide](https://github.com/SPRAGE/kiteticker-async-manager/blob/main/docs/guides/PERFORMANCE_IMPROVEMENTS.md)
mod errors;
pub mod manager;
mod models;
mod parser;
pub use errors::ParseTickError;
pub use models::tick_raw::{
  as_184 as tick_as_184, as_index_quote_32, as_inst_header_64, as_tick_raw,
  DepthItemRaw, DepthRaw, IndexQuoteRaw32, InstHeaderRaw64, TickHeaderRaw,
  TickRaw, INDEX_QUOTE_SIZE, INST_HEADER_SIZE, TICK_FULL_SIZE,
};
pub use models::{
  Depth, DepthItem, Exchange, Mode, Order, OrderStatus, OrderTransactionType,
  OrderValidity, Request, TextMessage, Tick, TickMessage, TickerMessage, OHLC,
};

pub mod ticker;
pub use manager::{
  ChannelId, HealthSummary, KiteManagerConfig, KiteTickerManager,
  KiteTickerManagerBuilder, ManagerStats,
};
pub use ticker::{KiteTickerAsync, KiteTickerSubscriber};
