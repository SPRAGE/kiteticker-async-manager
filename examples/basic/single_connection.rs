// Basic single WebSocket connection example
// This example demonstrates the simplest way to connect and receive market data

use kiteticker_async_manager::{KiteTickerAsync, Mode, TickerMessage};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Initialize logging
    env_logger::init();
    
    println!("🔌 Single Connection Example");
    println!("============================");
    
    // Get credentials from environment
    let api_key = std::env::var("KITE_API_KEY")
        .map_err(|_| "Please set KITE_API_KEY environment variable")?;
    let access_token = std::env::var("KITE_ACCESS_TOKEN")
        .map_err(|_| "Please set KITE_ACCESS_TOKEN environment variable")?;
    
    println!("📡 Connecting to Kite WebSocket...");
    
    // Establish WebSocket connection
    let ticker = KiteTickerAsync::connect(&api_key, &access_token).await
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    println!("✅ Connected successfully!");
    
    // Subscribe to a few symbols
    let symbols = vec![
        256265, // NIFTY 50
        408065, // HDFC Bank
        738561, // Reliance
    ];
    
    println!("📊 Subscribing to {} symbols with LTP mode", symbols.len());
    println!("Symbols: {:?}", symbols);
    
    let mut subscriber = ticker.subscribe(&symbols, Some(Mode::LTP)).await
        .map_err(|e| format!("Failed to subscribe: {}", e))?;
    
    println!("✅ Subscription successful!");
    println!("📈 Receiving live market data...\n");
    
    // Receive and process messages
    let mut message_count = 0;
    let start_time = std::time::Instant::now();
    
    loop {
        match subscriber.next_message().await {
            Ok(Some(message)) => {
                match message {
                    TickerMessage::Ticks(ticks) => {
                        message_count += 1;
                        
                        for tick in ticks {
                            if let Some(price) = tick.content.last_price {
                                println!("📈 Symbol {}: ₹{:.2}", tick.instrument_token, price);
                            }
                        }
                        
                        // Show statistics every 10 messages
                        if message_count % 10 == 0 {
                            let elapsed = start_time.elapsed();
                            let rate = message_count as f64 / elapsed.as_secs_f64();
                            println!("📊 Stats: {} messages in {:?} ({:.1} msg/sec)\n", 
                                    message_count, elapsed, rate);
                        }
                        
                        // Exit after 50 messages for demo
                        if message_count >= 50 {
                            println!("🏁 Demo completed! Received {} messages", message_count);
                            break;
                        }
                    },
                    TickerMessage::Message(message) => {
                        println!("📜 Broker message: {}", message);
                    },
                    TickerMessage::Error(error) => {
                        println!("❌ Error: {}", error);
                    },
                    TickerMessage::OrderPostback(order_result) => {
                        match order_result {
                            Ok(order) => println!("📋 Order update: {:?}", order),
                            Err(err) => println!("❌ Order error: {}", err),
                        }
                    },
                    TickerMessage::ClosingMessage(close_msg) => {
                        println!("🔌 Connection closing: {}", close_msg);
                    }
                }
            }
            Ok(None) => {
                println!("🔌 Connection closed by server");
                break;
            }
            Err(e) => {
                println!("❌ Error receiving message: {}", e);
                break;
            }
        }
    }
    
    // Close the connection
    println!("🛑 Closing connection...");
    subscriber.close().await.map_err(|e| format!("Failed to close: {}", e))?;
    println!("✅ Connection closed successfully");
    
    Ok(())
}
