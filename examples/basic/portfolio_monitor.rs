// Portfolio monitoring example
// This example demonstrates how to monitor a portfolio of stocks with organized data processing

use kiteticker_async_manager::{KiteTickerManager, KiteManagerConfig, Mode, TickerMessage};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct StockInfo {
    symbol: u32,
    name: String,
    current_price: f64,
    volume: u32,
    last_update: Instant,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize logging
    env_logger::init();
    
    println!("📊 Portfolio Monitor Example");
    println!("============================");
    
    // Get credentials
    let api_key = std::env::var("KITE_API_KEY")
        .map_err(|_| "Please set KITE_API_KEY environment variable")?;
    let access_token = std::env::var("KITE_ACCESS_TOKEN")
        .map_err(|_| "Please set KITE_ACCESS_TOKEN environment variable")?;
    
    // Define portfolio
    let portfolio = vec![
        (256265, "NIFTY 50"),
        (408065, "HDFC Bank"),
        (738561, "Reliance"),
        (5633, "TCS"),
        (884737, "Asian Paints"),
    ];
    
    println!("📈 Monitoring Portfolio:");
    for (symbol, name) in &portfolio {
        println!("   • {} ({})", name, symbol);
    }
    println!();
    
    // Create manager with optimized configuration for portfolio monitoring
    let config = KiteManagerConfig {
        max_connections: 1,  // Single connection for small portfolio
        max_symbols_per_connection: 100,
        connection_buffer_size: 2000,
        parser_buffer_size: 5000,
        enable_dedicated_parsers: true,
        default_mode: Mode::Quote,  // Quote mode for price + volume
        ..Default::default()
    };
    
    // Start manager
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    manager.start().await?;
    
    // Subscribe to portfolio symbols
    let symbols: Vec<u32> = portfolio.iter().map(|(symbol, _)| *symbol).collect();
    manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
    
    println!("✅ Subscribed to {} symbols", symbols.len());
    
    // Create portfolio tracking
    let mut portfolio_data: HashMap<u32, StockInfo> = HashMap::new();
    for (symbol, name) in portfolio {
        portfolio_data.insert(symbol, StockInfo {
            symbol,
            name: name.to_string(),
            current_price: 0.0,
            volume: 0,
            last_update: Instant::now(),
        });
    }
    
    // Get data channels
    let channels = manager.get_all_channels();
    
    // Start data processing
    for (channel_id, mut receiver) in channels {
        let mut portfolio_clone = portfolio_data.clone();
        
        tokio::spawn(async move {
            println!("📡 Started monitoring channel {:?}", channel_id);
            
            while let Ok(message) = receiver.recv().await {
                if let TickerMessage::Ticks(ticks) = message {
                    for tick in ticks {
                        if let Some(stock) = portfolio_clone.get_mut(&tick.instrument_token) {
                            // Update stock data
                            if let Some(price) = tick.content.last_price {
                                stock.current_price = price;
                            }
                            if let Some(volume) = tick.content.volume_traded {
                                stock.volume = volume;
                            }
                            stock.last_update = Instant::now();
                            
                            // Display update
                            println!("📈 {} ({}): ₹{:.2} | Vol: {} | {}",
                                stock.name,
                                stock.symbol,
                                stock.current_price,
                                stock.volume,
                                format_time_ago(stock.last_update));
                        }
                    }
                }
            }
        });
    }
    
    // Periodic portfolio summary
    let summary_portfolio = portfolio_data.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            print_portfolio_summary(&summary_portfolio);
        }
    });
    
    // Monitor for demo duration
    println!("📊 Monitoring portfolio for 2 minutes...\n");
    sleep(Duration::from_secs(120)).await;
    
    // Final summary
    println!("\n🏁 Demo completed!");
    print_portfolio_summary(&portfolio_data);
    
    // Stop manager
    manager.stop().await?;
    println!("✅ Portfolio monitor stopped");
    
    Ok(())
}

fn print_portfolio_summary(portfolio: &HashMap<u32, StockInfo>) {
    println!("\n📊 Portfolio Summary");
    println!("====================");
    
    let mut total_value = 0.0;
    let mut active_symbols = 0;
    
    for stock in portfolio.values() {
        if stock.current_price > 0.0 {
            total_value += stock.current_price;
            active_symbols += 1;
            
            let status = if stock.last_update.elapsed() < Duration::from_secs(60) {
                "🟢 Live"
            } else {
                "🔴 Stale"
            };
            
            println!("   {} {} | ₹{:.2} | Vol: {} | {}",
                status,
                stock.name,
                stock.current_price,
                stock.volume,
                format_time_ago(stock.last_update));
        }
    }
    
    println!("   💰 Portfolio metrics:");
    println!("     Active symbols: {}/{}", active_symbols, portfolio.len());
    println!("     Average price: ₹{:.2}", if active_symbols > 0 { total_value / active_symbols as f64 } else { 0.0 });
    println!();
}

fn format_time_ago(instant: Instant) -> String {
    let elapsed = instant.elapsed();
    
    if elapsed.as_secs() < 60 {
        format!("{}s ago", elapsed.as_secs())
    } else {
        format!("{}m ago", elapsed.as_secs() / 60)
    }
}
