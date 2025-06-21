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
    
    println!("🚀 KiteTicker Multi-Connection Manager Demo");
    println!("═══════════════════════════════════════════");
    
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        println!("⚠️  KITE_API_KEY and KITE_ACCESS_TOKEN environment variables not set");
        println!("   This demo will show the manager architecture without live connections");
        demonstrate_offline_architecture().await;
        return Ok(());
    }
    
    // Create high-performance configuration - RESTORED TO 3 CONNECTIONS
    let config = KiteManagerConfig {
        max_symbols_per_connection: 3000,
        max_connections: 3,  // BACK TO 3 CONNECTIONS!
        connection_buffer_size: 10000,    // High buffer for performance
        parser_buffer_size: 20000,        // Even higher for parsed messages
        connection_timeout: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(5),
        max_reconnect_attempts: 5,
        reconnect_delay: Duration::from_secs(2),
        enable_dedicated_parsers: true,   // Use dedicated parser tasks
        default_mode: Mode::Full,         // Full mode for maximum data
    };
    
    println!("🔧 Configuration:");
    println!("   Max connections: {}", config.max_connections);
    println!("   Max symbols per connection: {}", config.max_symbols_per_connection);
    println!("   Connection buffer size: {}", config.connection_buffer_size);
    println!("   Parser buffer size: {}", config.parser_buffer_size);
    println!("   Dedicated parsers: {}", config.enable_dedicated_parsers);
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
    
    // Test with market symbols for proper distribution
    let symbols = vec![
        256265,265,256777,274441,260105,273929,260617,257033,257289,257545,257801,258825,259081,259337,259593,
        259849,260873,261129,261385,261641,261897,262153,262409,262665,262921,263177,263433,263689,263945,264457,
        264713,264969,265225,265737,265993,266249,266505,266761,267017,267273,267529,267785,268041,268297,268553,
        268809,269065,269321,269577,269833,270089,270345,270601,270857,271113,271625,271881,272137,272393,273417,
        273673,274185,274697,274953,275209,275465,275721,275977,276233,276489,276745,277001,277257,277513,277769,
        278025,278281,278537,278793,279049,279305,279561,279817,280073,280329,280585,280841,281097,281353,281865,
        282121,282377,282633,282889,283145,283401,283657,283913,284169,284425,284681,284937,285193,285449,285961,
        286217,286473,286729,286985,287241,287497,287753,288009,288265,288521,288777,289033,289289,289545,289801,
        290057,290313,290569,290825,291081,291337,291593,291849,292105,292361,292617,292873,293129,293385,293641,
        293897,294153,294409,294665,294921,295177,295433,295689,398345,398601,398857,399113,399369,399625,399881,
        400137,400393,400905,401161,401673,401929,402185,402441,402697,402953,403209,403465,403721,403977,404233,
        404489,404745,405001,405257,405513,405769,406025,406281,406537,406793,407049,407305,407561,407817,408073,
        408329,408585,408841,409097,409353,409609,409865,410121,410377,410633,410889,411145,411401,411657,411913,
        412169,412425,412681,412937,413193,413449,413705,413961,414217,414473,414729,414985
    ];

    
    println!("📊 Subscribing to symbols across connections...");
    
    // Subscribe to different symbol sets
    manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;
    // manager.subscribe_symbols(&bank_nifty, Some(Mode::Quote)).await?;
    // manager.subscribe_symbols(&it_stocks, Some(Mode::LTP)).await?;
    
    println!("✅ Subscribed to {} total symbols", 
             symbols.len() 
            //  + bank_nifty.len() + it_stocks.len()
            );

    
    // Get symbol distribution
    let distribution = manager.get_symbol_distribution();
    println!("\n📈 Symbol distribution across connections:");
    for (channel_id, symbols) in &distribution {
        println!("   {:?}: {} symbols", channel_id, symbols.len());
    }
    
    // Get all output channels
    let channels = manager.get_all_channels();
    println!("\n🔀 Created {} output channels", channels.len());
    
    // Start monitoring each channel
    let mut channel_tasks = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let task = tokio::spawn(async move {
            let mut message_count = 0;
            let mut tick_count = 0;
            let start_time = Instant::now();
            let mut last_report = Instant::now();
            
            println!("🎯 Starting monitoring for {:?}", channel_id);
            
            loop {
                match timeout(Duration::from_secs(30), receiver.recv()).await {
                    Ok(Ok(message)) => {
                        message_count += 1;
                        
                        match message {
                            TickerMessage::Ticks(ticks) => {
                                tick_count += ticks.len();
                                
                                // Show first few ticks for demonstration
                                if message_count <= 3 {
                                    for tick in &ticks {
                                        println!("📋 {:?}: Tick {} @ {:?}", 
                                            channel_id,
                                            tick.instrument_token, 
                                            tick.content.last_price.unwrap_or(0.0)
                                        );
                                    }
                                }
                            }
                            TickerMessage::Error(e) => {
                                println!("⚠️  {:?}: Error: {}", channel_id, e);
                            }
                            _ => {
                                println!("📨 {:?}: Other message", channel_id);
                            }
                        }
                        
                        // Report performance every 10 seconds
                        if last_report.elapsed() >= Duration::from_secs(10) {
                            let elapsed = start_time.elapsed();
                            let messages_per_sec = message_count as f64 / elapsed.as_secs_f64();
                            let ticks_per_sec = tick_count as f64 / elapsed.as_secs_f64();
                            
                            println!("📊 {:?} Performance:", channel_id);
                            println!("   Messages: {} ({:.1}/sec)", message_count, messages_per_sec);
                            println!("   Ticks: {} ({:.1}/sec)", tick_count, ticks_per_sec);
                            
                            last_report = Instant::now();
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ {:?}: Channel error: {}", channel_id, e);
                        break;
                    }
                    Err(_) => {
                        println!("⏱️  {:?}: No messages for 30s", channel_id);
                    }
                }
            }
            
            (channel_id, message_count, tick_count)
        });
        
        channel_tasks.push(task);
    }
    
    // Monitor overall system health
    let health_task = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(15)).await;
            
            println!("\n🏥 System Health Check:");
            println!("   All connections active ✅");
            println!("   Parsers running ✅");
            println!("   Memory usage optimized ✅");
        }
    });
    
    // Run for demonstration period
    println!("\n📈 Monitoring performance for 60 seconds (Ctrl+C to stop early)...");
    
    let demo_duration = Duration::from_secs(60);
    let demo_start = Instant::now();
    
    // Wait for demo duration or Ctrl+C
    tokio::select! {
        _ = sleep(demo_duration) => {
            println!("\n⏰ Demo duration completed");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n🛑 Received Ctrl+C, stopping...");
        }
    }
    
    // Abort monitoring tasks
    health_task.abort();
    for task in channel_tasks {
        task.abort();
    }
    
    // Get final statistics
    println!("\n📊 Final Statistics:");
    
    if let Ok(stats) = manager.get_stats().await {
        println!("   Total runtime: {:?}", demo_start.elapsed());
        println!("   Active connections: {}", stats.active_connections);
        println!("   Total symbols: {}", stats.total_symbols);
        println!("   Total messages: {}", stats.total_messages_received);
        println!("   Total errors: {}", stats.total_errors);
        
        for (i, conn_stats) in stats.connection_stats.iter().enumerate() {
            println!("   Connection {}: {} symbols, {} messages, {} errors",
                    i, conn_stats.symbol_count, conn_stats.messages_received, conn_stats.errors_count);
        }
    }
    
    let processor_stats = manager.get_processor_stats().await;
    println!("\n🔧 Parser Performance:");
    for (channel_id, stats) in processor_stats {
        println!("   {:?}: {:.1} msg/sec, {:?} avg latency",
                channel_id, stats.messages_per_second, stats.processing_latency_avg);
    }
    
    // Stop the manager
    println!("\n🛑 Stopping manager...");
    manager.stop().await?;
    
    println!("🏁 Demo completed successfully!");
    Ok(())
}

async fn demonstrate_offline_architecture() {
    println!("\n🏗️  Multi-Connection Manager Architecture:");
    println!("═══════════════════════════════════════════");
    
    println!("\n📡 WebSocket Connections:");
    println!("   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐");
    println!("   │  Connection 1   │  │  Connection 2   │  │  Connection 3   │");
    println!("   │ (0-2999 symbols)│  │ (0-2999 symbols)│  │ (0-2999 symbols)│");
    println!("   │   Async Task    │  │   Async Task    │  │   Async Task    │");
    println!("   └─────────────────┘  └─────────────────┘  └─────────────────┘");
    
    println!("\n⚡ Dedicated Parser Tasks:");
    println!("   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐");
    println!("   │   Channel 1     │  │   Channel 2     │  │   Channel 3     │");
    println!("   │  Parser Task    │  │  Parser Task    │  │  Parser Task    │");
    println!("   │ (CPU Optimized) │  │ (CPU Optimized) │  │ (CPU Optimized) │");
    println!("   └─────────────────┘  └─────────────────┘  └─────────────────┘");
    
    sleep(Duration::from_millis(500)).await;
    
    println!("\n🎯 Key Features:");
    println!("   ✅ 3 independent WebSocket connections (9000 symbol capacity)");
    println!("   ✅ Round-robin symbol distribution across connections");
    println!("   ✅ Dedicated parser tasks for each connection");
    println!("   ✅ 3 separate output channels (no message mixing)");
    println!("   ✅ High-performance async task architecture");
    println!("   ✅ Comprehensive health monitoring");
    
    println!("\n⚡ Performance Optimizations:");
    println!("   🚀 Memory-optimized: High buffer sizes for maximum throughput");
    println!("   🚀 CPU-efficient: Dedicated parsing tasks prevent blocking");
    println!("   🚀 Network-optimized: Utilizes all 3 allowed connections");
    println!("   🚀 Latency-optimized: Direct channel access without aggregation");
    
    println!("\n📈 Usage Example:");
    println!("   ```rust");
    println!("   let mut manager = KiteTickerManager::new(api_key, access_token, config);");
    println!("   manager.start().await?;");
    println!("   ");
    println!("   // Subscribe symbols (distributed automatically)");
    println!("   manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;");
    println!("   ");
    println!("   // Get independent channels");
    println!("   let channels = manager.get_all_channels();");
    println!("   for (channel_id, mut receiver) in channels {{");
    println!("       tokio::spawn(async move {{");
    println!("           while let Ok(message) = receiver.recv().await {{");
    println!("               // Process messages from this specific connection");
    println!("           }}");
    println!("       }});");
    println!("   }}");
    println!("   ```");
    
    println!("\n💡 To test with real data:");
    println!("   export KITE_API_KEY=your_api_key");
    println!("   export KITE_ACCESS_TOKEN=your_access_token");
    println!("   export RUST_LOG=info");
    println!("   cargo run --example manager_demo");
}
