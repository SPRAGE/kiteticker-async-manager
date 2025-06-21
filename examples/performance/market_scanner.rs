// Market scanner example - High-volume symbol scanning
// This example demonstrates scanning large numbers of symbols efficiently

use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    
    println!("🔍 Market Scanner Example");
    println!("=========================");
    
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        return Err("Please set KITE_API_KEY and KITE_ACCESS_TOKEN".to_string());
    }
    
    // High-performance configuration for scanning
    let config = KiteManagerConfig {
        max_connections: 3,
        max_symbols_per_connection: 3000,
        connection_buffer_size: 20000,  // Large buffer for high volume
        parser_buffer_size: 50000,      // Even larger for parsed data
        enable_dedicated_parsers: true,
        default_mode: Mode::LTP,        // LTP mode for scanning (minimal bandwidth)
        ..Default::default()
    };
    
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // Large symbol set for market scanning
    let large_symbol_set = generate_symbol_list(8000); // 8000 symbols across 3 connections
    
    println!("📊 Scanning {} symbols across {} connections", 
             large_symbol_set.len(), 3);
    
    let start_time = Instant::now();
    manager.subscribe_symbols(&large_symbol_set, Some(Mode::LTP)).await?;
    println!("✅ Subscribed to {} symbols in {:?}", 
             large_symbol_set.len(), start_time.elapsed());
    
    // Get all channels and start parallel processing
    let channels = manager.get_all_channels();
    let mut handles = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let handle = tokio::spawn(async move {
            let mut scanner = MarketScanner::new(channel_id);
            
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    scanner.process_ticks(ticks).await;
                }
            }
            
            scanner.print_summary();
        });
        
        handles.push(handle);
    }
    
    // Run scanner for specified duration
    println!("🔄 Scanning market for 30 seconds...");
    sleep(Duration::from_secs(30)).await;
    
    // Stop manager
    manager.stop().await?;
    
    // Wait for all scanners to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    println!("🏁 Market scanning completed");
    Ok(())
}

struct MarketScanner {
    channel_id: kiteticker_async_manager::ChannelId,
    tick_count: u64,
    symbol_prices: HashMap<u32, f64>,
    price_changes: HashMap<u32, f64>,
    start_time: Instant,
    last_update: Instant,
}

impl MarketScanner {
    fn new(channel_id: kiteticker_async_manager::ChannelId) -> Self {
        let now = Instant::now();
        Self {
            channel_id,
            tick_count: 0,
            symbol_prices: HashMap::new(),
            price_changes: HashMap::new(),
            start_time: now,
            last_update: now,
        }
    }
    
    async fn process_ticks(&mut self, ticks: Vec<kiteticker_async_manager::TickMessage>) {
        for tick in ticks {
            self.tick_count += 1;
            
            if let Some(current_price) = tick.content.last_price {
                // Track price changes
                if let Some(&previous_price) = self.symbol_prices.get(&tick.instrument_token) {
                    let change_percent = ((current_price - previous_price) / previous_price) * 100.0;
                    self.price_changes.insert(tick.instrument_token, change_percent);
                    
                    // Alert on significant price movements
                    if change_percent.abs() > 2.0 {
                        println!("🚨 Alert: Symbol {} moved {:.2}% to ₹{:.2}", 
                                tick.instrument_token, change_percent, current_price);
                    }
                }
                
                self.symbol_prices.insert(tick.instrument_token, current_price);
            }
        }
        
        self.last_update = Instant::now();
        
        // Print progress every 1000 ticks
        if self.tick_count % 1000 == 0 {
            self.print_progress();
        }
    }
    
    fn print_progress(&self) {
        let elapsed = self.start_time.elapsed();
        let rate = self.tick_count as f64 / elapsed.as_secs_f64();
        
        println!("📈 Channel {:?}: {} ticks, {} symbols, {:.0} ticks/sec", 
                 self.channel_id, self.tick_count, self.symbol_prices.len(), rate);
    }
    
    fn print_summary(&self) {
        let elapsed = self.start_time.elapsed();
        let avg_rate = self.tick_count as f64 / elapsed.as_secs_f64();
        
        // Find top movers
        let mut movers: Vec<_> = self.price_changes.iter().collect();
        movers.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        
        println!("\n📊 Channel {:?} Summary:", self.channel_id);
        println!("   Total ticks: {}", self.tick_count);
        println!("   Unique symbols: {}", self.symbol_prices.len());
        println!("   Average rate: {:.1} ticks/sec", avg_rate);
        println!("   Duration: {:?}", elapsed);
        
        if !movers.is_empty() {
            println!("   Top 5 movers:");
            for (symbol, change) in movers.iter().take(5) {
                if let Some(&price) = self.symbol_prices.get(symbol) {
                    println!("     {} -> {:.2}% (₹{:.2})", symbol, change, price);
                }
            }
        }
    }
}

fn generate_symbol_list(count: usize) -> Vec<u32> {
    // Generate a realistic symbol list for testing
    // In production, you would load this from a file or database
    let base_symbols = vec![
        256265, 408065, 738561, 884737, 341249, 492033, // NIFTY 50 stocks
        779521, 2953217, 1850625, 2815745, 140481,      // Additional large caps
        1076033, 3876609, 5900545, 2672641, 177665,     // Mid caps
    ];
    
    let mut symbols = Vec::new();
    let mut symbol_id = 100000u32;
    
    // Add base symbols
    symbols.extend_from_slice(&base_symbols);
    
    // Generate additional symbols to reach desired count
    while symbols.len() < count {
        symbols.push(symbol_id);
        symbol_id += 1;
    }
    
    symbols.truncate(count);
    symbols
}
