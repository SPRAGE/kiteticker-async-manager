use kiteticker_async_manager::{
  KiteManagerConfig, KiteTickerManager, Mode, TickerMessage,
};
use log::info;
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};

#[tokio::main]
async fn main() -> Result<(), String> {
  // Initialize logging
  env_logger::init();

  println!("🔄 KiteTicker Mode Change Test");
  println!("═══════════════════════════════");

  let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();

  if api_key.is_empty() || access_token.is_empty() {
    println!(
      "⚠️  KITE_API_KEY and KITE_ACCESS_TOKEN environment variables not set"
    );
    demonstrate_mode_change_issue().await;
    return Ok(());
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

  println!("🔧 Starting manager...");
  let mut manager = KiteTickerManager::new(api_key, access_token, config);

  manager.start().await?;
  println!("✅ Manager started successfully");

  // Test the mode change issue
  test_mode_change_issue(&mut manager).await?;

  // Stop the manager
  println!("\n🛑 Stopping manager...");
  manager.stop().await?;

  println!("🏁 Mode change test completed!");
  Ok(())
}

async fn test_mode_change_issue(
  manager: &mut KiteTickerManager,
) -> Result<(), String> {
  println!("\n🧪 Testing Mode Change Issue");
  println!("=============================");

  // Step 1: Subscribe to a symbol with LTP mode
  let test_symbol = 738561; // Test symbol provided by user
  println!("\n1️⃣ Subscribing to symbol {} with LTP mode", test_symbol);
  manager
    .subscribe_symbols(&[test_symbol], Some(Mode::LTP))
    .await?;

  // Start monitoring ticks to see the mode
  let channels = manager.get_all_channels();
  let mut tick_listeners = Vec::new();

  for (channel_id, mut receiver) in channels {
    let task = tokio::spawn(async move {
      let mut ticks_received = 0;
      let start = Instant::now();

      while start.elapsed() < Duration::from_secs(5) && ticks_received < 3 {
        match timeout(Duration::from_millis(500), receiver.recv()).await {
          Ok(Ok(message)) => {
            if let TickerMessage::Ticks(ticks) = message {
              for tick in &ticks {
                if tick.instrument_token == test_symbol {
                  ticks_received += 1;
                  println!(
                    "   📊 Received tick for {}: Mode={:?}, LTP={:?}",
                    tick.instrument_token,
                    tick.content.mode,
                    tick.content.last_price
                  );

                  // Check if OHLC data is present (should not be for LTP mode)
                  if tick.content.ohlc.is_some() {
                    println!(
                      "   ⚠️  OHLC data present in LTP mode - unexpected!"
                    );
                  } else {
                    println!("   ✅ No OHLC data in LTP mode - correct");
                  }
                }
              }
            }
          }
          _ => continue,
        }
      }
      (channel_id, ticks_received)
    });
    tick_listeners.push(task);
  }

  // Wait for initial ticks
  for task in tick_listeners {
    if let Ok((channel_id, count)) = task.await {
      println!(
        "   📈 {:?}: Received {} ticks for initial LTP subscription",
        channel_id, count
      );
    }
  }

  sleep(Duration::from_secs(2)).await;

  // Step 2: Attempt to change mode to Full (this is where the issue should manifest)
  println!(
    "\n2️⃣ Attempting to change mode from LTP to Full for symbol {}",
    test_symbol
  );
  match manager.change_mode(&[test_symbol], Mode::Full).await {
    Ok(()) => {
      println!("   ✅ Mode change command sent successfully");
    }
    Err(e) => {
      println!("   ❌ Mode change command failed: {}", e);
      return Err(e);
    }
  }

  // Step 3: Monitor ticks after mode change to see if it actually worked
  println!("\n3️⃣ Monitoring ticks after mode change...");
  let channels = manager.get_all_channels();
  let mut post_change_listeners = Vec::new();

  for (channel_id, mut receiver) in channels {
    let task = tokio::spawn(async move {
      let mut ticks_received = 0;
      let mut ohlc_present = 0;
      let mut depth_present = 0;
      let start = Instant::now();

      while start.elapsed() < Duration::from_secs(10) && ticks_received < 5 {
        match timeout(Duration::from_millis(500), receiver.recv()).await {
          Ok(Ok(message)) => {
            if let TickerMessage::Ticks(ticks) = message {
              for tick in &ticks {
                if tick.instrument_token == test_symbol {
                  ticks_received += 1;
                  println!(
                    "   📊 Post-change tick for {}: Mode={:?}, LTP={:?}",
                    tick.instrument_token,
                    tick.content.mode,
                    tick.content.last_price
                  );

                  // Check if Full mode data is present
                  if let Some(ohlc) = &tick.content.ohlc {
                    ohlc_present += 1;
                    println!(
                      "   ✅ OHLC data present: O:{} H:{} L:{} C:{}",
                      ohlc.open, ohlc.high, ohlc.low, ohlc.close
                    );
                  } else {
                    println!("   ❌ OHLC data missing - mode change may not have worked!");
                  }

                  if let Some(depth) = &tick.content.depth {
                    depth_present += 1;
                    println!(
                      "   ✅ Market depth present: {} buy, {} sell orders",
                      depth.buy.len(),
                      depth.sell.len()
                    );
                  } else {
                    println!("   ❌ Market depth missing - mode change may not have worked!");
                  }

                  // Log the actual mode reported in the tick
                  info!("Tick mode reported: {:?}", tick.content.mode);
                }
              }
            }
          }
          _ => continue,
        }
      }
      (channel_id, ticks_received, ohlc_present, depth_present)
    });
    post_change_listeners.push(task);
  }

  // Wait for post-change ticks
  let mut total_ticks = 0;
  let mut total_ohlc = 0;
  let mut total_depth = 0;

  for task in post_change_listeners {
    if let Ok((channel_id, ticks, ohlc, depth)) = task.await {
      println!(
        "   📈 {:?}: {} ticks, {} with OHLC, {} with depth",
        channel_id, ticks, ohlc, depth
      );
      total_ticks += ticks;
      total_ohlc += ohlc;
      total_depth += depth;
    }
  }

  // Step 4: Analyze results
  println!("\n4️⃣ Test Results Analysis");
  println!("========================");

  if total_ticks == 0 {
    println!("   ⚠️  No ticks received after mode change - connection issue?");
  } else if total_ohlc == 0 && total_depth == 0 {
    println!("   ❌ ISSUE CONFIRMED: Mode change command sent but Full mode data not received");
    println!("   💡 This confirms that set_mode() alone doesn't work - need unsubscribe+resubscribe");
  } else if total_ohlc > 0 && total_depth > 0 {
    println!("   ✅ Mode change worked - Full mode data received");
  } else {
    println!(
      "   ⚠️  Partial mode change - some Full mode data received but not all"
    );
  }

  println!("\n📊 Summary:");
  println!("   Total ticks received: {}", total_ticks);
  println!("   Ticks with OHLC data: {}", total_ohlc);
  println!("   Ticks with market depth: {}", total_depth);

  Ok(())
}

async fn demonstrate_mode_change_issue() {
  println!("\n🔄 Mode Change Issue Demonstration");
  println!("==================================");

  println!("\n🐛 The Issue:");
  println!(
    "   When a WebSocket token is set to a specific mode (LTP, Quote, Full),"
  );
  println!(
    "   the Kite Connect WebSocket API doesn't allow direct mode changes."
  );
  println!(
    "   Simply sending a 'mode' command doesn't upgrade the subscription."
  );

  println!("\n❌ Current Implementation (Broken):");
  println!("   ```rust");
  println!("   // This only sends a mode command but doesn't work");
  println!("   manager.change_mode(&[symbol], Mode::Full).await?;");
  println!("   ```");

  println!("\n✅ Required Solution:");
  println!("   ```rust");
  println!("   // Must unsubscribe first, then resubscribe with new mode");
  println!("   manager.unsubscribe_symbols(&[symbol]).await?;");
  println!("   manager.subscribe_symbols(&[symbol], Some(Mode::Full)).await?;");
  println!("   ```");

  println!("\n🔧 Fix Needed:");
  println!("   The change_mode() method should internally:");
  println!("   1. Unsubscribe from the symbols");
  println!("   2. Resubscribe with the new mode");
  println!("   3. Maintain symbol tracking across the operation");

  println!("\n🚀 To test with real data:");
  println!("   export KITE_API_KEY=your_api_key");
  println!("   export KITE_ACCESS_TOKEN=your_access_token");
  println!("   export RUST_LOG=debug");
  println!("   cargo run --example mode_change_test");
}
