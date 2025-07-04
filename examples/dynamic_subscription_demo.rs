use kiteticker_async_manager::{
    KiteTickerManager, KiteManagerConfig, Mode, TickerMessage
};
use std::time::{Duration, Instant};
use tokio::time::{timeout, sleep};
use env_logger;
use log::debug;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize logging
    env_logger::init();
    
    println!("🔄 KiteTicker Dynamic Subscription Demo");
    println!("═══════════════════════════════════════════");
    
    // Debug: Show current logging configuration
    debug!("Logging initialized");
    debug!("Current log level: {}", 
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));
    debug!("For detailed manager logs, set RUST_LOG=debug");
    
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        println!("⚠️  KITE_API_KEY and KITE_ACCESS_TOKEN environment variables not set");
        demonstrate_offline_dynamic_architecture().await;
        return Ok(());
    }
    
    println!("💡 Note: This demo shows enhanced tick printing with detailed market data");
    println!("📅 During market hours, you'll see live tick data with OHLC, volume, and market depth");
    println!("🕐 Outside market hours, connections work but no tick data flows");
    println!();
    
    // Create configuration optimized for dynamic operations
    let config = KiteManagerConfig {
        max_symbols_per_connection: 3000,
        max_connections: 3,
        connection_buffer_size: 5000,
        parser_buffer_size: 10000,
        connection_timeout: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(5),
        max_reconnect_attempts: 5,
        reconnect_delay: Duration::from_secs(2),
        enable_dedicated_parsers: true,
        default_mode: Mode::LTP,  // Start with LTP for efficiency
    };
    
    println!("🔧 Configuration for Dynamic Operations:");
    println!("   Max connections: {}", config.max_connections);
    println!("   Max symbols per connection: {}", config.max_symbols_per_connection);
    println!("   Default mode: {:?}", config.default_mode);
    println!();
    
    // Create and start the manager
    println!("📡 Starting multi-connection manager...");
    let start_time = Instant::now();
    
    let mut manager = KiteTickerManager::new(
        api_key,
        access_token,
        config,
    );
    
    match timeout(Duration::from_secs(30), manager.start()).await {
        Ok(Ok(())) => {
            println!("✅ Manager started in {:?}", start_time.elapsed());
        }
        Ok(Err(e)) => {
            println!("❌ Manager failed to start: {}", e);
            return Err(e);
        }
        Err(_) => {
            println!("⏱️  Manager startup timeout");
            return Err("Manager startup timeout".to_string());
        }
    }
    
    // Start permanent tick listeners IMMEDIATELY after manager starts
    println!("🎯 Starting permanent tick listeners...");
    let channels = manager.get_all_channels();
    let mut permanent_listeners = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let listener_task = tokio::spawn(async move {
            let mut total_ticks = 0;
            loop {
                match receiver.recv().await {
                    Ok(message) => {
                        if let TickerMessage::Ticks(ticks) = message {
                            total_ticks += ticks.len();
                            println!("🎯 PERMANENT LISTENER {:?}: {} ticks (total: {})", 
                                    channel_id, ticks.len(), total_ticks);
                            
                            // Debug: Print raw tick data structure
                            debug!("Raw tick count: {}", ticks.len());
                            
                            for (i, tick) in ticks.iter().enumerate() {
                                println!("    🔹 Tick {}: Symbol: {}, LTP: {:?}, Volume: {:?}, Change: {:?}, Mode: {:?}", 
                                    i + 1,
                                    tick.instrument_token, 
                                    tick.content.last_price,
                                    tick.content.volume_traded,
                                    tick.content.net_change,
                                    tick.content.mode
                                );
                                
                                // Debug: Show all available fields
                                debug!("DEBUG Full Tick Data for symbol {}:", tick.instrument_token);
                                debug!("  - Instrument Token: {}", tick.instrument_token);
                                debug!("  - Timestamp: <not available>");
                                debug!("  - Tradable: <not available>");
                                debug!("  - Exchange Timestamp: {:?}", tick.content.exchange_timestamp);
                                debug!("  - Last Traded Time: <not available>");
                                debug!("  - Total Buy Quantity: {:?}", tick.content.total_buy_qty);
                                debug!("  - Total Sell Quantity: {:?}", tick.content.total_sell_qty);
                                debug!("  - Average Price: {:?}", tick.content.avg_traded_price);
                                debug!("  - OI: {:?}", tick.content.oi);
                                debug!("  - OI Day High: {:?}", tick.content.oi_day_high);
                                debug!("  - OI Day Low: {:?}", tick.content.oi_day_low);
                                
                                // Show OHLC data if available
                                if let Some(ohlc) = &tick.content.ohlc {
                                    println!("      📊 OHLC: O:{} H:{} L:{} C:{}", ohlc.open, ohlc.high, ohlc.low, ohlc.close);
                                    debug!("OHLC data present: O:{} H:{} L:{} C:{}", ohlc.open, ohlc.high, ohlc.low, ohlc.close);
                                } else {
                                    debug!("OHLC: Not available for symbol {}", tick.instrument_token);
                                }
                                
                                // Show additional data for full mode
                                if tick.content.mode == Mode::Full {
                                    if let Some(depth) = &tick.content.depth {
                                        println!("      📈 Market Depth: {} buy orders, {} sell orders", 
                                            depth.buy.len(), depth.sell.len());
                                        
                                        // Debug: Show first few depth entries
                                        debug!("Market depth for symbol {}: {} buy, {} sell", 
                                               tick.instrument_token, depth.buy.len(), depth.sell.len());
                                        
                                        if !depth.buy.is_empty() {
                                            debug!("Top Buy Orders for symbol {}:", tick.instrument_token);
                                            for (idx, buy_order) in depth.buy.iter().take(3).enumerate() {
                                                debug!("  {}. Price: {}, Qty: {}, Orders: {}", 
                                                    idx + 1, buy_order.price, buy_order.qty, buy_order.orders);
                                            }
                                        }
                                        if !depth.sell.is_empty() {
                                            debug!("Top Sell Orders for symbol {}:", tick.instrument_token);
                                            for (idx, sell_order) in depth.sell.iter().take(3).enumerate() {
                                                debug!("  {}. Price: {}, Qty: {}, Orders: {}", 
                                                    idx + 1, sell_order.price, sell_order.qty, sell_order.orders);
                                            }
                                        }
                                    } else {
                                        debug!("Market Depth: Not available for symbol {}", tick.instrument_token);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Listener {:?} error: {}", channel_id, e);
                        break;
                    }
                }
            }
        });
        permanent_listeners.push(listener_task);
    }
    
    // Give listeners a moment to initialize
    sleep(Duration::from_millis(100)).await;
    println!("✅ Permanent listeners started and ready");
    
    // Demo: Multi-connection distribution workflow
    demo_dynamic_subscription(&mut manager).await?;
    
    // Stop the manager
    println!("\n🛑 Stopping manager...");
    manager.stop().await?;
    
    println!("🏁 Dynamic subscription demo completed successfully!");
    Ok(())
}

async fn demo_dynamic_subscription(manager: &mut KiteTickerManager) -> Result<(), String> {
    println!("\n🎯 True Dynamic Subscription Demo");
    println!("==================================");
    
    // Start with a small initial set
    let initial_symbols = vec![256265, 265, 256777]; // 3 symbols
    let additional_batch_1 = vec![274441, 260105, 273929]; // 3 more symbols  
    let additional_batch_2 = vec![260617, 257033, 257289, 257545]; // 4 more symbols
    let symbols_to_remove = vec![265, 274441]; // Remove some symbols
    let final_batch = vec![257801, 258825]; // Add final symbols
    
    // Step 1: Initial subscription with small set
    println!("\n📊 Step 1: Initial subscription to {} symbols", initial_symbols.len());
    println!("Starting with: {:?}", initial_symbols);
    
    // Start listening for ticks BEFORE subscribing
    let channels_before_sub = manager.get_all_channels();
    let mut tick_listeners = Vec::new();
    
    for (channel_id, mut receiver) in channels_before_sub {
        let task = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let mut tick_count = 0;
            
            // Listen for initial ticks for 5 seconds
            while start.elapsed() < Duration::from_secs(5) {
                match timeout(Duration::from_millis(500), receiver.recv()).await {
                    Ok(Ok(message)) => {
                        if let TickerMessage::Ticks(ticks) = message {
                            tick_count += ticks.len();
                            println!("🎯 {:?}: Received {} ticks immediately after subscription!", channel_id, ticks.len());
                            
                            // Debug: Show detailed tick information
                            debug!("Processing {} ticks on {:?}", ticks.len(), channel_id);
                            
                            for (idx, tick) in ticks.iter().enumerate() {
                                println!("  🔹 Tick {}: Symbol: {}, LTP: {:?}, Volume: {:?}, Change: {:?}", 
                                    idx + 1,
                                    tick.instrument_token, 
                                    tick.content.last_price,
                                    tick.content.volume_traded,
                                    tick.content.net_change
                                );
                                
                                // Debug: Show tick structure details
                                debug!("Tick {} Details for symbol {}:", idx + 1, tick.instrument_token);
                                debug!("  - Raw instrument_token: {}", tick.instrument_token);
                                debug!("  - Raw last_price: {:?}", tick.content.last_price);
                                debug!("  - Raw volume_traded: {:?}", tick.content.volume_traded);
                                debug!("  - Raw change: {:?}", tick.content.net_change);
                                debug!("  - Raw last_quantity: <not available>");
                                debug!("  - Raw average_price: {:?}", tick.content.avg_traded_price);
                                debug!("  - Raw buy_quantity: {:?}", tick.content.total_buy_qty);
                                debug!("  - Raw sell_quantity: {:?}", tick.content.total_sell_qty);
                                debug!("  - Raw oi: {:?}", tick.content.oi);
                                debug!("  - Raw mode: {:?}", tick.content.mode);
                                
                                if let Some(ohlc) = &tick.content.ohlc {
                                    debug!("  - OHLC available: O:{} H:{} L:{} C:{}", 
                                        ohlc.open, ohlc.high, ohlc.low, ohlc.close);
                                }
                                
                                if let Some(depth) = &tick.content.depth {
                                    debug!("  - Market depth available: {} buy, {} sell", 
                                        depth.buy.len(), depth.sell.len());
                                }
                            }
                        }
                    }
                    _ => continue,
                }
            }
            (channel_id, tick_count)
        });
        tick_listeners.push(task);
    }
    
    // Now subscribe to symbols
    manager.subscribe_symbols(&initial_symbols, Some(Mode::LTP)).await?;
    
    println!("✅ Subscription sent, waiting for initial ticks...");
    
    // Wait for the listeners to finish
    for task in tick_listeners {
        if let Ok((channel_id, count)) = task.await {
            println!("📊 {:?}: Received {} total ticks during initial subscription", channel_id, count);
        }
    }
    
    print_distribution(manager, "After initial subscription").await;
    
    // Step 2: Wait and monitor initial data
    println!("\n⏳ Step 2: Monitoring initial data flow");
    monitor_ticks_briefly(manager, 5, "Initial Subscription Data").await;
    
    if let Ok(stats) = manager.get_stats().await {
        println!("✅ Current Statistics:");
        println!("   Active connections: {}", stats.active_connections);
        println!("   Total symbols: {}", stats.total_symbols);
        println!("   Total messages: {}", stats.total_messages_received);
    }

    // Step 3: Dynamic addition - Batch 1
    println!("\n➕ Step 3: DYNAMIC ADDITION - Adding {} more symbols", additional_batch_1.len());
    println!("Adding: {:?}", additional_batch_1);
    manager.subscribe_symbols(&additional_batch_1, Some(Mode::Quote)).await?;
    
    // Give time for new tick data to arrive
    sleep(Duration::from_secs(2)).await;
    
    print_distribution(manager, "After first dynamic addition").await;
    monitor_ticks_briefly(manager, 3, "After First Addition").await;

    // Step 4: Dynamic addition - Batch 2 
    println!("\n➕ Step 4: DYNAMIC ADDITION - Adding {} more symbols", additional_batch_2.len());
    println!("Adding: {:?}", additional_batch_2);
    manager.subscribe_symbols(&additional_batch_2, Some(Mode::Full)).await?;
    
    // Give time for new tick data to arrive
    sleep(Duration::from_secs(2)).await;
    
    print_distribution(manager, "After second dynamic addition").await;
    monitor_ticks_briefly(manager, 3, "After Second Addition").await;

    // Step 5: Dynamic removal
    println!("\n➖ Step 5: DYNAMIC REMOVAL - Removing {} symbols", symbols_to_remove.len());
    println!("Removing: {:?}", symbols_to_remove);
    match manager.unsubscribe_symbols(&symbols_to_remove).await {
        Ok(()) => {
            print_distribution(manager, "After dynamic removal").await;
            println!("✅ Dynamic removal successful!");
        }
        Err(e) => {
            println!("⚠️  Dynamic removal failed: {}", e);
        }
    }
    monitor_ticks_briefly(manager, 3, "After Symbol Removal").await;

    // Step 6: Final addition and mode change demo
    println!("\n➕ Step 6: FINAL ADDITION - Adding {} symbols", final_batch.len()); 
    println!("Adding: {:?}", final_batch);
    manager.subscribe_symbols(&final_batch, Some(Mode::LTP)).await?;
    
    print_distribution(manager, "After final addition").await;
    
    // Step 7: Mode change demonstration
    println!("\n🔄 Step 7: MODE CHANGE - Changing subscription mode");
    let symbols_for_mode_change = vec![256265, 260105]; // Change some existing symbols to Full mode
    println!("Changing {:?} to Full mode", symbols_for_mode_change);
    match manager.change_mode(&symbols_for_mode_change, Mode::Full).await {
        Ok(()) => println!("✅ Mode change successful!"),
        Err(e) => println!("⚠️  Mode change failed: {}", e),
    }

    // Step 8: Final statistics and monitoring
    println!("\n📈 Step 8: Final Statistics and Performance");
    sleep(Duration::from_secs(3)).await; // Let some data flow
    
    if let Ok(stats) = manager.get_stats().await {
        println!("✅ Final Manager Statistics:");
        println!("   Active connections: {}", stats.active_connections);
        println!("   Total symbols: {}", stats.total_symbols);
        println!("   Total messages: {}", stats.total_messages_received);
        println!("   Total errors: {}", stats.total_errors);
        
        for (i, conn_stats) in stats.connection_stats.iter().enumerate() {
            println!("   Connection {}: {} symbols, {} messages", 
                    i + 1, conn_stats.symbol_count, conn_stats.messages_received);
        }
    }
    
    // Step 9: Show processor performance
    println!("\n⚡ Step 9: Final Parser Performance");
    let processor_stats = manager.get_processor_stats().await;
    for (channel_id, stats) in processor_stats {
        println!("   {:?}: {:.1} msg/sec, {:?} avg latency", 
                channel_id, stats.messages_per_second, stats.processing_latency_avg);
    }
    
    // Step 10: Monitor live data for a short period
    println!("\n📺 Step 10: Live Data Monitoring (10 seconds)");
    println!("Monitoring real-time tick data from all dynamically managed connections...");
    
    let channels = manager.get_all_channels();
    let mut tasks = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let task = tokio::spawn(async move {
            let mut count = 0;
            let start = std::time::Instant::now();
            
            while start.elapsed() < Duration::from_secs(10) {
                match timeout(Duration::from_secs(2), receiver.recv()).await {
                    Ok(Ok(message)) => {
                        count += 1;
                        if let TickerMessage::Ticks(ticks) = message {
                            println!("📋 {:?}: {} ticks received", channel_id, ticks.len());
                            
                            // Debug: Final monitoring with comprehensive details
                            debug!("Final monitoring - Message #{}, {} ticks on {:?}", 
                                count, ticks.len(), channel_id);
                            
                            for tick in &ticks {
                                println!("  🔹 Symbol: {}, LTP: {:?}, Volume: {:?}, Change: {:?}", 
                                    tick.instrument_token, 
                                    tick.content.last_price,
                                    tick.content.volume_traded,
                                    tick.content.net_change
                                );
                                
                                // Debug: Show complete tick analysis
                                debug!("Final Debug Analysis for Symbol {}:", tick.instrument_token);
                                debug!("  - Timestamp: Now = {:?}", std::time::SystemTime::now());
                                debug!("  - Data completeness check:");
                                debug!("    * Last Price: {:?} (✅)", tick.content.last_price);
                                debug!("    * Volume: {:?} ({})", 
                                    tick.content.volume_traded, 
                                    if tick.content.volume_traded.is_some() { "✅" } else { "❌" });
                                debug!("    * Change: {:?} ({})", 
                                    tick.content.net_change,
                                    if tick.content.net_change.is_some() { "✅" } else { "❌" });
                                debug!("    * Mode: {:?} (✅)", tick.content.mode);
                                
                                // Validate mode-specific data
                                match tick.content.mode {
                                    Mode::Full => {
                                        debug!("  - Full mode validation:");
                                        debug!("    * OHLC: {}", if tick.content.ohlc.is_some() { "✅ Present" } else { "❌ Missing" });
                                        debug!("    * Depth: {}", if tick.content.depth.is_some() { "✅ Present" } else { "❌ Missing" });
                                        if let Some(ohlc) = &tick.content.ohlc {
                                            debug!("    * OHLC Values: O:{} H:{} L:{} C:{}", 
                                                ohlc.open, ohlc.high, ohlc.low, ohlc.close);
                                        }
                                        if let Some(depth) = &tick.content.depth {
                                            debug!("    * Depth Levels: {} buy, {} sell", 
                                                depth.buy.len(), depth.sell.len());
                                        }
                                    }
                                    Mode::Quote => {
                                        debug!("  - Quote mode validation:");
                                        debug!("    * OHLC: {}", if tick.content.ohlc.is_some() { "✅ Present" } else { "❌ Missing" });
                                        if let Some(ohlc) = &tick.content.ohlc {
                                            debug!("    * OHLC Values: O:{} H:{} L:{} C:{}", 
                                                ohlc.open, ohlc.high, ohlc.low, ohlc.close);
                                        }
                                    }
                                    Mode::LTP => {
                                        debug!("  - LTP mode validation: ✅ Basic data only");
                                    }
                                }
                            }
                        } else {
                            println!("📋 {:?}: Non-tick message received", channel_id);
                            debug!("Message type: {:?}", message);
                        }
                    }
                    _ => continue,
                }
            }
            (channel_id, count)
        });
        tasks.push(task);
    }
    
    // Wait for monitoring to complete
    for task in tasks {
        if let Ok((channel_id, count)) = task.await {
            println!("📊 {:?}: {} total messages in 10 seconds", channel_id, count);
        }
    }

    // Final cleanup demonstration
    println!("\n🧹 Step 11: Final Cleanup Demonstration");
    let current_symbols: Vec<u32> = manager.get_symbol_distribution()
        .values()
        .flat_map(|symbols| symbols.iter().cloned())
        .collect();
    
    println!("Attempting to unsubscribe from {} remaining symbols...", current_symbols.len());
    
    match manager.unsubscribe_symbols(&current_symbols).await {
        Ok(()) => {
            print_distribution(manager, "After complete cleanup").await;
            println!("✅ Complete cleanup successful!");
        }
        Err(e) => {
            println!("⚠️  Cleanup encountered issues: {}", e);
            println!("💡 This demonstrates the current architecture capabilities");
        }
    }
    
    println!("\n✅ Dynamic Subscription Demo Completed!");
    println!("🎯 Key Dynamic Features Demonstrated:");
    println!("   ✅ Runtime symbol addition across multiple batches");
    println!("   ✅ Runtime symbol removal with proper tracking");
    println!("   ✅ Dynamic mode changes for existing symbols");
    println!("   ✅ Real-time capacity distribution and monitoring");
    println!("   ✅ Independent message processing per connection");
    println!("   ✅ High-performance parser tasks with statistics");
    println!("   ✅ Complete subscription lifecycle management");
    
    Ok(())
}

async fn print_distribution(manager: &KiteTickerManager, context: &str) {
    let distribution = manager.get_symbol_distribution();
    println!("\n📈 Symbol Distribution ({}):", context);
    
    let mut total = 0;
    for (channel_id, symbols) in &distribution {
        println!("   {:?}: {} symbols", channel_id, symbols.len());
        total += symbols.len();
    }
    println!("   Total: {} symbols", total);
}

async fn monitor_ticks_briefly(manager: &mut KiteTickerManager, duration_secs: u64, context: &str) {
    println!("\n📺 {} - Monitoring ticks for {} seconds...", context, duration_secs);
    
    let channels = manager.get_all_channels();
    let mut tasks = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let task = tokio::spawn(async move {
            let mut count = 0;
            let start = std::time::Instant::now();
            
            while start.elapsed() < Duration::from_secs(duration_secs) {
                match timeout(Duration::from_secs(2), receiver.recv()).await {
                    Ok(Ok(message)) => {
                        count += 1;
                        if let TickerMessage::Ticks(ticks) = message {
                            println!("📊 {:?}: {} ticks in batch #{}", channel_id, ticks.len(), count);
                            
                            // Debug: Enhanced tick monitoring
                            debug!("Monitoring tick batch #{} with {} ticks on {:?}", count, ticks.len(), channel_id);
                            
                            for (idx, tick) in ticks.iter().enumerate() {
                                println!("  🔹 Symbol: {}, LTP: {:?}, Volume: {:?}, Change: {:?}, OHLC: [{}/{}/{}/{}]", 
                                    tick.instrument_token, 
                                    tick.content.last_price,
                                    tick.content.volume_traded,
                                    tick.content.net_change,
                                    tick.content.ohlc.as_ref().map(|o| o.open).unwrap_or(0.0),
                                    tick.content.ohlc.as_ref().map(|o| o.high).unwrap_or(0.0),
                                    tick.content.ohlc.as_ref().map(|o| o.low).unwrap_or(0.0),
                                    tick.content.ohlc.as_ref().map(|o| o.close).unwrap_or(0.0)
                                );
                                
                                // Debug: Show tick metadata
                                debug!("Tick {} metadata for symbol {}:", idx + 1, tick.instrument_token);
                                debug!("  - Received at: {:?}", std::time::SystemTime::now());
                                debug!("  - Mode: {:?}", tick.content.mode);
                                debug!("  - Last Qty: <not available>");
                                debug!("  - Avg Price: {:?}", tick.content.avg_traded_price);
                                debug!("  - Buy/Sell Qty: {:?}/{:?}", tick.content.total_buy_qty, tick.content.total_sell_qty);
                                
                                if tick.content.mode == Mode::Full || tick.content.mode == Mode::Quote {
                                    if tick.content.ohlc.is_some() {
                                        debug!("  - ✅ OHLC data present for symbol {}", tick.instrument_token);
                                    } else {
                                        debug!("  - ❌ OHLC data missing for symbol {} (expected for mode: {:?})", 
                                               tick.instrument_token, tick.content.mode);
                                    }
                                }
                                
                                if tick.content.mode == Mode::Full {
                                    if let Some(depth) = &tick.content.depth {
                                        debug!("  - ✅ Market depth present for symbol {}: {} buy, {} sell levels", 
                                            tick.instrument_token, depth.buy.len(), depth.sell.len());
                                    } else {
                                        debug!("  - ❌ Market depth missing for symbol {} (expected for full mode)", 
                                               tick.instrument_token);
                                    }
                                }
                            }
                        } else {
                            debug!("Non-tick message received on {:?}: {:?}", channel_id, message);
                        }
                    }
                    _ => continue,
                }
            }
            (channel_id, count)
        });
        tasks.push(task);
    }
    
    // Wait for monitoring to complete
    for task in tasks {
        if let Ok((channel_id, count)) = task.await {
            println!("📊 {:?}: {} total messages in {} seconds", channel_id, count, duration_secs);
        }
    }
}

async fn demonstrate_offline_dynamic_architecture() {
    println!("\n🔄 Dynamic Subscription Architecture:");
    println!("═══════════════════════════════════════");
    
    println!("\n🎯 Key Dynamic Features:");
    println!("   ✅ Runtime symbol addition/removal per connection");
    println!("   ✅ Mode changes for existing symbols");
    println!("   ✅ Intelligent load balancing across 3 connections");
    println!("   ✅ Real-time capacity monitoring");
    println!("   ✅ Efficient WebSocket command batching");
    
    println!("\n📊 Capacity Management:");
    println!("   🔹 Connection 1: 0-3000 symbols");
    println!("   🔹 Connection 2: 0-3000 symbols");
    println!("   🔹 Connection 3: 0-3000 symbols");
    println!("   🔹 Total Capacity: 9000 symbols");
    
    println!("\n⚡ Dynamic Operations:");
    println!("   ```rust");
    println!("   // Add symbols at runtime");
    println!("   manager.subscribe_symbols(&[408065, 884737], Some(Mode::Full)).await?;");
    println!("   ");
    println!("   // Remove symbols");
    println!("   manager.unsubscribe_symbols(&[408065]).await?;");
    println!("   ");
    println!("   // Change subscription mode");
    println!("   manager.change_mode(&[884737], Mode::Quote).await?;");
    println!("   ");
    println!("   // Check distribution");
    println!("   let distribution = manager.get_symbol_distribution();");
    println!("   ```");
    
    println!("\n🚀 Performance Benefits:");
    println!("   ⚡ No connection restarts needed");
    println!("   ⚡ Minimal network overhead");
    println!("   ⚡ Automatic load balancing");
    println!("   ⚡ Real-time capacity monitoring");
    
    println!("\n💡 Use Cases:");
    println!("   📈 Algorithmic trading with changing watchlists");
    println!("   📊 Market scanners with dynamic filters");
    println!("   🔍 Event-driven subscription management");
    println!("   ⏰ Time-based symbol rotation");
    
    println!("\n🔧 To test with real data:");
    println!("   export KITE_API_KEY=your_api_key");
    println!("   export KITE_ACCESS_TOKEN=your_access_token");
    println!("   export RUST_LOG=debug");
    println!("   cargo run --example dynamic_subscription_demo");
}
