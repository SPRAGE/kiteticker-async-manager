// High-frequency trading example - Maximum throughput optimization
// This example demonstrates ultra-low latency processing for HFT scenarios

use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::init();
    
    println!("⚡ High-Frequency Trading Example");
    println!("=================================");
    
    let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
    let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
    
    if api_key.is_empty() || access_token.is_empty() {
        return Err("Please set KITE_API_KEY and KITE_ACCESS_TOKEN".to_string());
    }
    
    // Ultra-high performance configuration for HFT
    let config = KiteManagerConfig {
        max_connections: 3,
        max_symbols_per_connection: 1000,  // Focused symbol set for HFT
        connection_buffer_size: 100000,    // Maximum possible buffer
        parser_buffer_size: 200000,        // Ultra-large parser buffer
        enable_dedicated_parsers: true,
        default_mode: Mode::Full,          // Full market depth data
        ..Default::default()
    };
    
    println!("🚀 Optimized for:");
    println!("   Sub-microsecond latency");
    println!("   Maximum throughput");
    println!("   Real-time order book analysis");
    println!("   Ultra-low garbage collection");
    
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // Focus on highly liquid instruments for HFT
    let hft_symbols = vec![
        256265, // NIFTY 50 - highly liquid
        408065, // HDFC Bank - large volume
        738561, // Reliance - most traded
        884737, // ICICI Bank
        341249, // TCS
        492033, // ITC
        779521, // Kotak Bank
    ];
    
    println!("📊 Subscribing to {} liquid instruments", hft_symbols.len());
    manager.subscribe_symbols(&hft_symbols, Some(Mode::Full)).await?;
    
    // Setup high-frequency processing engines
    let channels = manager.get_all_channels();
    let mut hft_engines = Vec::new();
    
    for (channel_id, mut receiver) in channels {
        let engine = tokio::spawn(async move {
            let mut hft_processor = HFTProcessor::new(channel_id);
            
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    hft_processor.process_ultra_fast(ticks).await;
                }
            }
            
            hft_processor.print_performance_metrics();
        });
        
        hft_engines.push(engine);
    }
    
    // Run HFT simulation
    println!("⚡ Starting high-frequency processing...");
    sleep(Duration::from_secs(60)).await;
    
    // Stop processing
    manager.stop().await?;
    
    // Wait for all engines to complete
    for engine in hft_engines {
        let _ = engine.await;
    }
    
    println!("🏁 High-frequency trading simulation completed");
    Ok(())
}

struct HFTProcessor {
    channel_id: kiteticker_async_manager::ChannelId,
    tick_count: AtomicU64,
    order_signals: AtomicU64,
    latency_samples: Arc<tokio::sync::Mutex<VecDeque<Duration>>>,
    price_book: Arc<tokio::sync::RwLock<OrderBookState>>,
    start_time: Instant,
}

impl HFTProcessor {
    fn new(channel_id: kiteticker_async_manager::ChannelId) -> Self {
        Self {
            channel_id,
            tick_count: AtomicU64::new(0),
            order_signals: AtomicU64::new(0),
            latency_samples: Arc::new(tokio::sync::Mutex::new(VecDeque::with_capacity(10000))),
            price_book: Arc::new(tokio::sync::RwLock::new(OrderBookState::new())),
            start_time: Instant::now(),
        }
    }
    
    async fn process_ultra_fast(&mut self, ticks: Vec<kiteticker_async_manager::TickMessage>) {
        let process_start = Instant::now();
        
        for tick in ticks {
            self.tick_count.fetch_add(1, Ordering::Relaxed);
            
            // Ultra-fast tick processing
            if let Some(signal) = self.analyze_tick_hft(&tick).await {
                self.order_signals.fetch_add(1, Ordering::Relaxed);
                self.execute_hft_strategy(signal).await;
            }
            
            // Record processing latency (sampling every 100th tick for performance)
            if self.tick_count.load(Ordering::Relaxed) % 100 == 0 {
                let latency = process_start.elapsed();
                if let Ok(mut samples) = self.latency_samples.try_lock() {
                    samples.push_back(latency);
                    if samples.len() > 1000 {
                        samples.pop_front();
                    }
                }
            }
        }
    }
    
    async fn analyze_tick_hft(&self, tick: &kiteticker_async_manager::TickMessage) -> Option<TradingSignal> {
        // Ultra-fast market analysis
        
        // Get current market state
        let book_state = if let Ok(book) = self.price_book.try_read() {
            book.clone()
        } else {
            return None;
        };
        
        // Price movement analysis
        if let Some(current_price) = tick.content.last_price {
            if let Some(previous_price) = book_state.get_last_price(tick.instrument_token) {
                let price_change = (current_price - previous_price) / previous_price;
                
                // High-frequency signals
                if price_change > 0.001 {  // 0.1% up movement
                    return Some(TradingSignal::Buy {
                        symbol: tick.instrument_token,
                        price: current_price,
                        confidence: (price_change * 1000.0).min(1.0),
                    });
                } else if price_change < -0.001 {  // 0.1% down movement
                    return Some(TradingSignal::Sell {
                        symbol: tick.instrument_token,
                        price: current_price,
                        confidence: (price_change.abs() * 1000.0).min(1.0),
                    });
                }
            }
            
            // Update price book
            if let Ok(mut book) = self.price_book.try_write() {
                book.update_price(tick.instrument_token, current_price);
            }
        }
        
        // Order book analysis (if available)
        if let Some(depth) = &tick.content.depth {
            let spread = self.calculate_spread(depth);
            let imbalance = self.calculate_order_imbalance(depth);
            
            // Micro-structure signals
            if spread < 0.01 && imbalance.abs() > 0.7 {
                return Some(TradingSignal::Arbitrage {
                    symbol: tick.instrument_token,
                    spread,
                    imbalance,
                });
            }
        }
        
        None
    }
    
    async fn execute_hft_strategy(&self, signal: TradingSignal) {
        // Ultra-fast strategy execution (simulation)
        match signal {
            TradingSignal::Buy { symbol, price, confidence } => {
                if confidence > 0.8 {
                    println!("🟢 BUY Signal: {} @ ₹{:.2} (confidence: {:.2})", 
                             symbol, price, confidence);
                    // In real HFT: submit_buy_order(symbol, price, quantity).await;
                }
            },
            TradingSignal::Sell { symbol, price, confidence } => {
                if confidence > 0.8 {
                    println!("🔴 SELL Signal: {} @ ₹{:.2} (confidence: {:.2})", 
                             symbol, price, confidence);
                    // In real HFT: submit_sell_order(symbol, price, quantity).await;
                }
            },
            TradingSignal::Arbitrage { symbol, spread, imbalance } => {
                println!("⚡ ARB Signal: {} (spread: {:.4}, imbalance: {:.2})", 
                         symbol, spread, imbalance);
                // In real HFT: execute_arbitrage_strategy(symbol, spread).await;
            }
        }
    }
    
    fn calculate_spread(&self, depth: &kiteticker_async_manager::Depth) -> f64 {
        if let (Some(best_bid), Some(best_ask)) = (
            depth.buy.first().map(|d| d.price),
            depth.sell.first().map(|d| d.price)
        ) {
            (best_ask - best_bid) / best_ask
        } else {
            1.0 // Large spread if no data
        }
    }
    
    fn calculate_order_imbalance(&self, depth: &kiteticker_async_manager::Depth) -> f64 {
        let bid_volume: u32 = depth.buy.iter().map(|d| d.qty).sum();
        let ask_volume: u32 = depth.sell.iter().map(|d| d.qty).sum();
        
        if bid_volume + ask_volume > 0 {
            (bid_volume as f64 - ask_volume as f64) / (bid_volume + ask_volume) as f64
        } else {
            0.0
        }
    }
    
    fn print_performance_metrics(&self) {
        let total_ticks = self.tick_count.load(Ordering::Relaxed);
        let total_signals = self.order_signals.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        let tick_rate = total_ticks as f64 / elapsed.as_secs_f64();
        let signal_rate = (total_signals as f64 / total_ticks as f64) * 100.0;
        
        println!("\n⚡ Channel {:?} HFT Performance:", self.channel_id);
        println!("   Total ticks processed: {}", total_ticks);
        println!("   Trading signals generated: {}", total_signals);
        println!("   Tick processing rate: {:.0} ticks/sec", tick_rate);
        println!("   Signal generation rate: {:.2}%", signal_rate);
        
        // Latency statistics
        if let Ok(samples) = self.latency_samples.try_lock() {
            if !samples.is_empty() {
                let avg_latency = samples.iter().sum::<Duration>() / samples.len() as u32;
                let min_latency = samples.iter().min().unwrap();
                let max_latency = samples.iter().max().unwrap();
                
                // Calculate percentiles
                let mut sorted_samples: Vec<_> = samples.iter().collect();
                sorted_samples.sort();
                let p95_idx = (sorted_samples.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted_samples.len() as f64 * 0.99) as usize;
                
                println!("   Average latency: {:?}", avg_latency);
                println!("   Minimum latency: {:?}", min_latency);
                println!("   Maximum latency: {:?}", max_latency);
                println!("   95th percentile: {:?}", sorted_samples[p95_idx]);
                println!("   99th percentile: {:?}", sorted_samples[p99_idx]);
            }
        }
        
        println!("   Runtime: {:?}", elapsed);
    }
}

#[derive(Debug, Clone)]
enum TradingSignal {
    Buy { symbol: u32, price: f64, confidence: f64 },
    Sell { symbol: u32, price: f64, confidence: f64 },
    Arbitrage { symbol: u32, spread: f64, imbalance: f64 },
}

#[derive(Debug, Clone)]
struct OrderBookState {
    last_prices: std::collections::HashMap<u32, f64>,
    last_update: Instant,
}

impl OrderBookState {
    fn new() -> Self {
        Self {
            last_prices: std::collections::HashMap::new(),
            last_update: Instant::now(),
        }
    }
    
    fn get_last_price(&self, symbol: u32) -> Option<f64> {
        self.last_prices.get(&symbol).copied()
    }
    
    fn update_price(&mut self, symbol: u32, price: f64) {
        self.last_prices.insert(symbol, price);
        self.last_update = Instant::now();
    }
}
