//! # Multi-API Manager Demo
//!
//! This example demonstrates how to use the `MultiApiKiteTickerManager` to handle
//! multiple Kite Connect API credentials simultaneously.
//!
//! ## Features Demonstrated
//!
//! - Adding multiple API keys to a single manager
//! - Automatic symbol distribution across API keys (round-robin)
//! - Manual symbol assignment to specific API keys
//! - Receiving unified message stream with API key identification
//! - Monitoring per-API and aggregate statistics
//! - Dynamic subscription management
//!
//! ## Usage
//!
//! Set environment variables for multiple accounts:
//!
//! ```bash
//! export KITE_API_KEY_1="your_first_api_key"
//! export KITE_ACCESS_TOKEN_1="your_first_access_token"
//! export KITE_API_KEY_2="your_second_api_key"
//! export KITE_ACCESS_TOKEN_2="your_second_access_token"
//! # Optional third account
//! export KITE_API_KEY_3="your_third_api_key"
//! export KITE_ACCESS_TOKEN_3="your_third_access_token"
//!
//! cargo run --example multi_api_demo
//! ```

use kiteticker_async_manager::{
  DistributionStrategy, Mode, MultiApiKiteTickerManager, TickerMessage,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), String> {
  // Initialize logger
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
    .init();

  println!("🚀 Multi-API KiteTicker Manager Demo\n");

  // Load credentials from environment
  let credentials = load_credentials()?;

  if credentials.is_empty() {
    return Err("No API credentials found. Please set KITE_API_KEY_N and KITE_ACCESS_TOKEN_N environment variables.".to_string());
  }

  println!("📋 Loaded {} API key(s):\n", credentials.len());
  for (id, _, _) in &credentials {
    println!("  ✓ {}", id);
  }
  println!();

  // Build multi-API manager
  let mut builder = MultiApiKiteTickerManager::builder()
    .max_connections_per_api(3)
    .max_symbols_per_connection(3000)
    .distribution_strategy(DistributionStrategy::RoundRobin)
    .default_mode(Mode::Quote)
    .enable_health_monitoring(true);

  // Add all API keys
  for (id, api_key, access_token) in credentials {
    builder = builder.add_api_key(id, api_key, access_token);
  }

  let mut manager = builder.build();

  // Start the manager
  println!("🔌 Starting multi-API manager...");
  manager.start().await?;
  println!("✅ Manager started successfully\n");

  // Demo symbols (Indian stock market)
  let nifty_symbols = vec![
    256265, // NIFTY 50
    260105, // NIFTY BANK
  ];

  let stock_symbols = vec![
    408065, // HDFC Bank
    738561, // Reliance Industries
    779521, // Infosys
    492033, // TCS
  ];

  // Example 1: Auto-distribute symbols across all API keys
  println!("📊 Example 1: Auto-distributing symbols across API keys");
  manager
    .subscribe_symbols(&nifty_symbols, Some(Mode::Quote))
    .await?;
  println!("  ✓ Subscribed to {} index symbols\n", nifty_symbols.len());

  // Show symbol distribution
  let distribution = manager.get_symbol_distribution();
  println!("📍 Symbol Distribution:");
  for (api_key_id, conn_map) in &distribution {
    println!("  API Key: {}", api_key_id.0);
    for (conn_idx, symbols) in conn_map {
      println!("    Connection {}: {:?}", conn_idx, symbols);
    }
  }
  println!();

  // Example 2: Manual assignment to specific API key
  let api_keys = manager.get_api_keys();
  if !api_keys.is_empty() {
    let first_api = &api_keys[0];
    println!("📊 Example 2: Manually assigning symbols to {}", first_api.0);
    manager
      .subscribe_symbols_to_api(first_api.clone(), &stock_symbols, Some(Mode::LTP))
      .await?;
    println!("  ✓ Subscribed to {} stock symbols\n", stock_symbols.len());
  }

  // Example 3: Receive unified message stream
  println!("📨 Example 3: Receiving unified message stream");
  println!("  (Press Ctrl+C to stop)\n");

  let mut unified_channel = manager.get_unified_channel();
  let stats_handle = tokio::spawn({
    let manager_clone = manager.get_api_keys();
    async move {
      loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("\n📊 Periodic Stats Check");
        println!("  API Keys: {:?}", manager_clone);
      }
    }
  });

  // Message processing loop
  let mut message_count = 0;
  let mut tick_count_by_api = std::collections::HashMap::new();

  let timeout_duration = Duration::from_secs(60); // Run for 60 seconds
  let start_time = std::time::Instant::now();

  while start_time.elapsed() < timeout_duration {
    match tokio::time::timeout(Duration::from_secs(5), unified_channel.recv()).await
    {
      Ok(Ok((api_key_id, message))) => {
        message_count += 1;

        match message {
          TickerMessage::Ticks(ticks) => {
            let count = tick_count_by_api.entry(api_key_id.0.clone()).or_insert(0);
            *count += ticks.len();

            if message_count % 10 == 0 {
              // Print every 10th message to avoid spam
              println!(
                "  [{:>15}] Received {} ticks (Total messages: {})",
                api_key_id.0,
                ticks.len(),
                message_count
              );

              // Show first tick details
              if let Some(tick) = ticks.first() {
                println!(
                  "    └─ Token: {}, Mode: {:?}, LTP: {:?}",
                  tick.instrument_token,
                  tick.content.mode,
                  tick.content.last_price
                );
              }
            }
          }
          TickerMessage::Error(err) => {
            println!("  [{:>15}] Error: {}", api_key_id.0, err);
          }
          other => {
            println!("  [{:>15}] Other message: {:?}", api_key_id.0, other);
          }
        }
      }
      Ok(Err(e)) => {
        println!("  ⚠️  Channel error: {}", e);
      }
      Err(_) => {
        println!("  ⏱️  No messages for 5 seconds, checking stats...");
        
        // Get and display statistics
        let stats = manager.get_stats().await;
        println!("\n📈 Current Statistics:");
        println!("  Total API Keys: {}", stats.total_api_keys);
        println!("  Total Connections: {}", stats.total_connections);
        println!("  Total Symbols: {}", stats.total_symbols);
        println!("  Total Messages: {}", stats.total_messages_received);
        println!("  Uptime: {:?}", stats.uptime);

        println!("\n  Per-API Stats:");
        for api_stat in &stats.per_api_stats {
          println!("    {}:", api_stat.api_key_id);
          println!("      Active Connections: {}", api_stat.active_connections);
          println!("      Symbols: {}", api_stat.total_symbols);
          println!("      Messages: {}", api_stat.total_messages_received);
        }
        println!();
      }
    }
  }

  println!("\n⏹️  Demo completed after 60 seconds\n");

  // Final statistics
  println!("📊 Final Tick Count by API Key:");
  for (api_key, count) in tick_count_by_api {
    println!("  {}: {} ticks", api_key, count);
  }
  println!();

  let final_stats = manager.get_stats().await;
  println!("📈 Final Statistics:");
  println!("  Total Messages Received: {}", final_stats.total_messages_received);
  println!("  Total Messages Parsed: {}", final_stats.total_messages_parsed);
  println!("  Total Errors: {}", final_stats.total_errors);
  println!("  Uptime: {:?}", final_stats.uptime);

  // Example 4: Dynamic operations
  println!("\n📊 Example 4: Dynamic subscription management");

  // Change mode
  if !nifty_symbols.is_empty() {
    println!("  Changing mode for index symbols to Full...");
    manager
      .change_mode(&nifty_symbols, Mode::Full)
      .await?;
    println!("  ✓ Mode changed");
  }

  // Wait a bit for mode change to take effect
  tokio::time::sleep(Duration::from_secs(2)).await;

  // Unsubscribe from some symbols
  if !stock_symbols.is_empty() {
    println!("  Unsubscribing from stock symbols...");
    manager.unsubscribe_symbols(&stock_symbols).await?;
    println!("  ✓ Unsubscribed from {} symbols", stock_symbols.len());
  }

  // Show updated distribution
  println!("\n📍 Updated Symbol Distribution:");
  let final_distribution = manager.get_symbol_distribution();
  for (api_key_id, conn_map) in &final_distribution {
    println!("  API Key: {}", api_key_id.0);
    for (conn_idx, symbols) in conn_map {
      println!("    Connection {}: {:?}", conn_idx, symbols);
    }
  }
  println!();

  // Cleanup
  stats_handle.abort();
  let _ = stats_handle.await;

  println!("🛑 Stopping manager...");
  manager.stop().await?;
  println!("✅ Manager stopped successfully\n");

  println!("🎉 Multi-API demo completed!");

  Ok(())
}

/// Load API credentials from environment variables
fn load_credentials() -> Result<Vec<(String, String, String)>, String> {
  let mut credentials = Vec::new();

  // Try to load up to 5 API keys
  for i in 1..=5 {
    let api_key_var = format!("KITE_API_KEY_{}", i);
    let access_token_var = format!("KITE_ACCESS_TOKEN_{}", i);

    if let (Ok(api_key), Ok(access_token)) = (
      std::env::var(&api_key_var),
      std::env::var(&access_token_var),
    ) {
      let id = format!("account_{}", i);
      credentials.push((id, api_key, access_token));
    }
  }

  Ok(credentials)
}
