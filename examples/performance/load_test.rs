// Load testing example - Stress testing the multi-connection manager
// This example tests the system under high load conditions

use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    
    println!("🧪 Load Testing Example");
    println!("=======================");
    
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        return Err("Please set KITE_API_KEY and KITE_ACCESS_TOKEN".to_string());
    }
    
    // Maximum performance configuration
    let config = KiteManagerConfig {
        max_connections: 3,
        max_symbols_per_connection: 3000,
        connection_buffer_size: 50000,  // Very large buffers
        parser_buffer_size: 100000,     // Maximum buffer size
        enable_dedicated_parsers: true,
        default_mode: Mode::Full,       // Full mode for maximum data
        ..Default::default()
    };
    
    println!("⚙️  Configuration:");
    println!("   Max connections: {}", config.max_connections);
    println!("   Max symbols per connection: {}", config.max_symbols_per_connection);
    println!("   Connection buffer: {}", config.connection_buffer_size);
    println!("   Parser buffer: {}", config.parser_buffer_size);
    println!("   Mode: {:?}", config.default_mode);
    
    // Run multiple load test scenarios
    run_load_test_suite(api_key, access_token, config).await?;
    
    Ok(())
}

async fn run_load_test_suite(
    api_key: String, 
    access_token: String, 
    config: KiteManagerConfig
) -> Result<(), String> {
    
    // Test 1: Maximum symbol capacity
    println!("\n🔬 Test 1: Maximum Symbol Capacity");
    println!("===================================");
    
    let symbols = generate_test_symbols(9000); // Maximum capacity
    test_symbol_load(&api_key, &access_token, &config, symbols, Duration::from_secs(60)).await?;
    
    // Test 2: High frequency mode
    println!("\n🔬 Test 2: High Frequency Processing");
    println!("====================================");
    
    let hf_symbols = generate_test_symbols(1000); // Fewer symbols but higher frequency
    test_high_frequency(&api_key, &access_token, &config, hf_symbols, Duration::from_secs(30)).await?;
    
    // Test 3: Dynamic subscription stress test
    println!("\n🔬 Test 3: Dynamic Subscription Stress");
    println!("======================================");
    
    test_dynamic_subscriptions(&api_key, &access_token, &config, Duration::from_secs(45)).await?;
    
    println!("\n🏁 Load testing completed successfully!");
    Ok(())
}

async fn test_symbol_load(
    api_key: &str,
    access_token: &str,
    config: &KiteManagerConfig,
    symbols: Vec<u32>,
    duration: Duration,
) -> Result<(), String> {
    
    let mut manager = KiteTickerManager::new(
        api_key.to_string(), 
        access_token.to_string(), 
        config.clone()
    );
    
    let start_time = Instant::now();
    manager.start().await?;
    println!("✅ Manager started in {:?}", start_time.elapsed());
    
    // Subscribe to all symbols
    let sub_start = Instant::now();
    manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;
    println!("✅ Subscribed to {} symbols in {:?}", symbols.len(), sub_start.elapsed());
    
    // Start processing
    let stats = Arc::new(LoadTestStats::new());
    let channels = manager.get_all_channels();
    let mut handles = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let stats_clone = Arc::clone(&stats);
        let handle = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                match message {
                    TickerMessage::Ticks(ticks) => {
                        stats_clone.record_ticks(ticks.len() as u64);
                    },
                    TickerMessage::Error(error) => {
                        stats_clone.record_error();
                        eprintln!("Channel {:?} error: {}", channel_id, error);
                    },
                    _ => {}
                }
            }
        });
        handles.push(handle);
    }
    
    // Run test
    let stats_clone = Arc::clone(&stats);
    let monitor_handle = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            stats_clone.print_stats();
        }
    });
    
    sleep(duration).await;
    
    // Stop and collect results
    manager.stop().await?;
    monitor_handle.abort();
    
    for handle in handles {
        handle.abort();
    }
    
    stats.print_final_stats(duration);
    Ok(())
}

async fn test_high_frequency(
    api_key: &str,
    access_token: &str,
    config: &KiteManagerConfig,
    symbols: Vec<u32>,
    duration: Duration,
) -> Result<(), String> {
    
    let mut hf_config = config.clone();
    hf_config.default_mode = Mode::Full; // Maximum data mode
    
    let mut manager = KiteTickerManager::new(
        api_key.to_string(), 
        access_token.to_string(), 
        hf_config
    );
    
    manager.start().await?;
    manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;
    
    // High-frequency processing test
    let stats = Arc::new(LoadTestStats::new());
    let channels = manager.get_all_channels();
    
    for (_channel_id, mut receiver) in channels {
        let stats_clone = Arc::clone(&stats);
        tokio::spawn(async move {
            let mut latencies = Vec::new();
            
            while let Ok(message) = receiver.recv().await {
                let receive_time = Instant::now();
                
                match message {
                    TickerMessage::Ticks(ticks) => {
                        // Simulate high-frequency processing
                        for tick in &ticks {
                            // Calculate processing latency
                            let processing_time = receive_time.elapsed();
                            latencies.push(processing_time);
                            
                            // Perform intensive calculations (simulating HFT)
                            let _ = calculate_indicators(tick);
                        }
                        
                        stats_clone.record_ticks(ticks.len() as u64);
                        
                        // Record latency stats periodically
                        if latencies.len() >= 100 {
                            let avg_latency: Duration = latencies.iter().sum::<Duration>() / latencies.len() as u32;
                            stats_clone.record_latency(avg_latency);
                            latencies.clear();
                        }
                    },
                    _ => {}
                }
            }
        });
    }
    
    sleep(duration).await;
    manager.stop().await?;
    
    stats.print_hf_stats(duration);
    Ok(())
}

async fn test_dynamic_subscriptions(
    api_key: &str,
    access_token: &str,
    config: &KiteManagerConfig,
    duration: Duration,
) -> Result<(), String> {
    
    let mut manager = KiteTickerManager::new(
        api_key.to_string(), 
        access_token.to_string(), 
        config.clone()
    );
    
    manager.start().await?;
    
    let start_time = Instant::now();
    let mut operation_count = 0;
    
    while start_time.elapsed() < duration {
        // Add symbols
        let new_symbols = generate_test_symbols(100);
        if let Ok(Ok(())) = timeout(Duration::from_secs(5), 
                              manager.subscribe_symbols(&new_symbols, Some(Mode::LTP))).await {
            operation_count += 1;
        }
        
        sleep(Duration::from_millis(500)).await;
        
        // Remove some symbols
        let remove_symbols: Vec<u32> = new_symbols.into_iter().take(50).collect();
        if let Ok(Ok(())) = timeout(Duration::from_secs(5), 
                              manager.unsubscribe_symbols(&remove_symbols)).await {
            operation_count += 1;
        }
        
        sleep(Duration::from_millis(500)).await;
        
        // Change mode
        let mode_symbols = generate_test_symbols(25);
        if let Ok(Ok(())) = timeout(Duration::from_secs(5), 
                              manager.change_mode(&mode_symbols, Mode::Quote)).await {
            operation_count += 1;
        }
        
        sleep(Duration::from_millis(1000)).await;
    }
    
    manager.stop().await?;
    
    println!("✅ Completed {} dynamic operations in {:?}", operation_count, duration);
    println!("   Average rate: {:.1} operations/sec", 
             operation_count as f64 / duration.as_secs_f64());
    
    Ok(())
}

struct LoadTestStats {
    total_ticks: AtomicU64,
    total_errors: AtomicU64,
    start_time: Instant,
    latencies: Arc<tokio::sync::Mutex<Vec<Duration>>>,
}

impl LoadTestStats {
    fn new() -> Self {
        Self {
            total_ticks: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            start_time: Instant::now(),
            latencies: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    
    fn record_ticks(&self, count: u64) {
        self.total_ticks.fetch_add(count, Ordering::Relaxed);
    }
    
    fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    fn record_latency(&self, latency: Duration) {
        if let Ok(mut latencies) = self.latencies.try_lock() {
            latencies.push(latency);
        }
    }
    
    fn print_stats(&self) {
        let elapsed = self.start_time.elapsed();
        let ticks = self.total_ticks.load(Ordering::Relaxed);
        let errors = self.total_errors.load(Ordering::Relaxed);
        let rate = ticks as f64 / elapsed.as_secs_f64();
        
        println!("📊 Current: {} ticks, {} errors, {:.1} ticks/sec", 
                 ticks, errors, rate);
    }
    
    fn print_final_stats(&self, duration: Duration) {
        let ticks = self.total_ticks.load(Ordering::Relaxed);
        let errors = self.total_errors.load(Ordering::Relaxed);
        let avg_rate = ticks as f64 / duration.as_secs_f64();
        
        println!("\n📈 Final Results:");
        println!("   Total ticks processed: {}", ticks);
        println!("   Total errors: {}", errors);
        println!("   Average rate: {:.1} ticks/sec", avg_rate);
        println!("   Error rate: {:.4}%", (errors as f64 / ticks as f64) * 100.0);
        println!("   Duration: {:?}", duration);
    }
    
    fn print_hf_stats(&self, duration: Duration) {
        self.print_final_stats(duration);
        
        if let Ok(latencies) = self.latencies.try_lock() {
            if !latencies.is_empty() {
                let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
                let max_latency = latencies.iter().max().unwrap();
                
                println!("   Average latency: {:?}", avg_latency);
                println!("   Maximum latency: {:?}", max_latency);
            }
        }
    }
}

fn generate_test_symbols(count: usize) -> Vec<u32> {
    (0..count).map(|i| 100000 + i as u32).collect()
}

fn calculate_indicators(tick: &kiteticker_async_manager::TickMessage) -> f64 {
    // Simulate complex HFT calculations
    if let Some(price) = tick.content.last_price {
        // Mock technical indicators
        let ema = price * 0.9;
        let rsi = (price % 100.0) / 100.0;
        let momentum = price - (price * 0.95);
        
        ema + rsi + momentum
    } else {
        0.0
    }
}
