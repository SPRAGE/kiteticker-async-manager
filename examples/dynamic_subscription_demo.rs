use kiteticker_async_manager::{
    KiteTickerManager, KiteManagerConfig, Mode, TickerMessage
};
use std::time::{Duration, Instant};
use tokio::time::{timeout, sleep};
use env_logger;

#[tokio::main]
pub async fn main() -> Result<(), String> {
    // Initialize logging
    env_logger::init();
    
    println!("🔄 KiteTicker Dynamic Subscription Demo");
    println!("═══════════════════════════════════════════");
    
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        println!("⚠️  KITE_API_KEY and KITE_ACCESS_TOKEN environment variables not set");
        demonstrate_offline_dynamic_architecture().await;
        return Ok(());
    }
    
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
    manager.subscribe_symbols(&initial_symbols, Some(Mode::LTP)).await?;
    
    print_distribution(manager, "After initial subscription").await;
    
    // Step 2: Wait and monitor initial data
    println!("\n⏳ Step 2: Monitoring initial data flow (5 seconds)");
    sleep(Duration::from_secs(5)).await;
    
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
    
    print_distribution(manager, "After first dynamic addition").await;
    sleep(Duration::from_secs(3)).await;

    // Step 4: Dynamic addition - Batch 2 
    println!("\n➕ Step 4: DYNAMIC ADDITION - Adding {} more symbols", additional_batch_2.len());
    println!("Adding: {:?}", additional_batch_2);
    manager.subscribe_symbols(&additional_batch_2, Some(Mode::Full)).await?;
    
    print_distribution(manager, "After second dynamic addition").await;
    sleep(Duration::from_secs(3)).await;

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
    sleep(Duration::from_secs(3)).await;

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
                match timeout(Duration::from_millis(100), receiver.recv()).await {
                    Ok(Ok(message)) => {
                        count += 1;
                        if let TickerMessage::Ticks(ticks) = message {
                            if count <= 3 { // Show first few ticks
                                println!("📋 {:?}: {} ticks received", channel_id, ticks.len());
                            }
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
    println!("   export RUST_LOG=info");
    println!("   cargo run --example dynamic_subscription_demo");
}
