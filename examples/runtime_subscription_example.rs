use kiteticker_async_manager::{
    KiteTickerManager, KiteManagerConfig, Mode, TickerMessage
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize with your API credentials
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        println!("⚠️ Please set KITE_API_KEY and KITE_ACCESS_TOKEN environment variables");
        return Err("Missing API credentials".to_string());
    }
    
    // Create configuration
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
        default_mode: Mode::LTP,
    };
    
    // Start the manager
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    println!("🚀 KiteTicker Manager started!");
    
    // Example of runtime subscription management
    runtime_subscription_demo(&mut manager).await?;
    
    // Stop the manager
    manager.stop().await?;
    Ok(())
}

async fn runtime_subscription_demo(manager: &mut KiteTickerManager) -> Result<(), String> {
    println!("\n📡 Runtime Subscription Management Demo");
    println!("=====================================");
    
    // 1. Start with initial symbols
    println!("\n1️⃣ Initial subscription to base symbols");
    let initial_symbols = vec![256265, 265, 256777];
    manager.subscribe_symbols(&initial_symbols, Some(Mode::LTP)).await?;
    print_current_state(manager, "Initial subscription").await;
    
    sleep(Duration::from_secs(2)).await;
    
    // 2. Add more symbols dynamically
    println!("\n2️⃣ Adding symbols dynamically");
    let additional_symbols = vec![274441, 260105, 273929];
    manager.subscribe_symbols(&additional_symbols, Some(Mode::Quote)).await?;
    print_current_state(manager, "After adding symbols").await;
    
    sleep(Duration::from_secs(2)).await;
    
    // 3. Change mode for existing symbols
    println!("\n3️⃣ Changing subscription mode");
    let symbols_for_mode_change = vec![256265, 265];
    manager.change_mode(&symbols_for_mode_change, Mode::Full).await?;
    print_current_state(manager, "After mode change").await;
    
    sleep(Duration::from_secs(2)).await;
    
    // 4. Remove some symbols
    println!("\n4️⃣ Removing symbols dynamically");
    let symbols_to_remove = vec![265, 274441];
    manager.unsubscribe_symbols(&symbols_to_remove).await?;
    print_current_state(manager, "After removing symbols").await;
    
    sleep(Duration::from_secs(2)).await;
    
    // 5. Add different symbols with different modes
    println!("\n5️⃣ Adding new symbols with Full mode");
    let full_mode_symbols = vec![257801, 258825];
    manager.subscribe_symbols(&full_mode_symbols, Some(Mode::Full)).await?;
    print_current_state(manager, "After adding Full mode symbols").await;
    
    // 6. Monitor live data for a short period
    println!("\n6️⃣ Monitoring live data (5 seconds)");
    monitor_live_data(manager, 5).await?;
    
    // 7. Complete cleanup
    println!("\n7️⃣ Complete cleanup");
    let all_symbols: Vec<u32> = manager.get_symbol_distribution()
        .values()
        .flat_map(|symbols| symbols.iter().cloned())
        .collect();
    
    if !all_symbols.is_empty() {
        manager.unsubscribe_symbols(&all_symbols).await?;
        print_current_state(manager, "After complete cleanup").await;
    }
    
    println!("\n✅ Runtime subscription demo completed!");
    Ok(())
}

async fn print_current_state(manager: &KiteTickerManager, context: &str) {
    println!("\n📊 Current State ({})", context);
    
    // Show distribution
    let distribution = manager.get_symbol_distribution();
    let mut total = 0;
    for (channel_id, symbols) in &distribution {
        println!("   {:?}: {} symbols", channel_id, symbols.len());
        total += symbols.len();
    }
    println!("   📈 Total: {} symbols", total);
    
    // Show stats
    if let Ok(stats) = manager.get_stats().await {
        println!("   📊 Stats: {} connections, {} messages", 
                stats.active_connections, stats.total_messages_received);
    }
}

async fn monitor_live_data(manager: &mut KiteTickerManager, seconds: u64) -> Result<(), String> {
    let channels = manager.get_all_channels();
    let mut tasks = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let task = tokio::spawn(async move {
            let mut count = 0;
            let start = std::time::Instant::now();
            
            while start.elapsed() < Duration::from_secs(seconds) {
                match tokio::time::timeout(Duration::from_millis(100), receiver.recv()).await {
                    Ok(Ok(message)) => {
                        count += 1;
                        if let TickerMessage::Ticks(ticks) = message {
                            if count <= 2 { // Show first few ticks
                                println!("   📋 {:?}: Received {} ticks", channel_id, ticks.len());
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
            println!("   📊 {:?}: {} total messages", channel_id, count);
        }
    }
    
    Ok(())
}
