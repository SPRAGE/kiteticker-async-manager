use kiteticker_async_manager::{
  KiteManagerConfig, KiteTickerManager, Mode, TickerMessage,
};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), String> {
  env_logger::init();

  println!("🔄 Simple Tick Test - Capturing Initial Ticks");
  println!("═══════════════════════════════════════════");

  let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();

  if api_key.is_empty() || access_token.is_empty() {
    println!(
      "⚠️  KITE_API_KEY and KITE_ACCESS_TOKEN environment variables not set"
    );
    return Ok(());
  }

  let config = KiteManagerConfig {
    max_symbols_per_connection: 3000,
    max_connections: 1, // Use only 1 connection for simplicity
    connection_buffer_size: 5000,
    parser_buffer_size: 10000,
    connection_timeout: Duration::from_secs(30),
    health_check_interval: Duration::from_secs(5),
    max_reconnect_attempts: 5,
    reconnect_delay: Duration::from_secs(2),
    enable_dedicated_parsers: true,
    default_mode: Mode::LTP,
  };

  let mut manager = KiteTickerManager::new(api_key, access_token, config);

  // Start manager
  println!("📡 Starting manager...");
  manager.start().await?;
  println!("✅ Manager started");

  // Get channel BEFORE subscribing
  println!("🎯 Getting channels...");
  let channels = manager.get_all_channels();
  let (channel_id, mut receiver) = channels.into_iter().next().unwrap();

  // Start listener in background
  let listener_handle = tokio::spawn(async move {
    println!("👂 Listener started for {:?}", channel_id);

    let mut tick_count = 0;
    loop {
      match timeout(Duration::from_secs(30), receiver.recv()).await {
        Ok(Ok(message)) => match message {
          TickerMessage::Ticks(ticks) => {
            tick_count += ticks.len();
            println!(
              "🎯 CAPTURED TICKS! {:?}: {} ticks (total: {})",
              channel_id,
              ticks.len(),
              tick_count
            );

            for tick in &ticks {
              println!("🔹 FULL TICK DEBUG:");
              println!("{:#?}", tick);
              println!("─────────────────────────────────────");
            }
          }
          TickerMessage::Error(err) => {
            println!("❌ Error: {}", err);
          }
          _ => {
            println!("📨 Other message: {:?}", message);
          }
        },
        Ok(Err(e)) => {
          println!("❌ Receive error: {}", e);
          break;
        }
        Err(_) => {
          println!("⏱️  Listener timeout");
          break;
        }
      }
    }
    println!("👂 Listener stopped");
  });

  // Give listener time to start
  tokio::time::sleep(Duration::from_millis(200)).await;

  // Now subscribe to a symbol
  println!("📊 Subscribing to symbol 256265...");
  manager
    .subscribe_symbols(&[128083204], Some(Mode::Full))
    .await?;
  println!("✅ Subscription sent");

  // Wait for ticks
  println!("⏳ Waiting 10 seconds for ticks...");
  tokio::time::sleep(Duration::from_secs(10)).await;

  // Stop
  manager.stop().await?;
  listener_handle.abort();

  println!("🏁 Test completed");
  Ok(())
}
