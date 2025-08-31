use kiteticker_async_manager::{
  as_index_quote_32, as_inst_header_64, as_tick_raw, ChannelId,
  KiteTickerManagerBuilder, Mode, INDEX_QUOTE_SIZE, INST_HEADER_SIZE,
};
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout};

#[tokio::main]
async fn main() -> Result<(), String> {
  // Optional logging for debugging
  let _ = env_logger::try_init();

  println!(
    "🚀 KiteTicker Raw Manager Demo (multi-connection + zero-copy peek)"
  );
  println!("═════════════════════════════════════════════════════════");

  let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();

  if api_key.is_empty() || access_token.is_empty() {
    println!(
      "⚠️  Set KITE_API_KEY and KITE_ACCESS_TOKEN to run this live example."
    );
    println!(
      "   This combines the multi-connection manager with raw frame parsing."
    );
    return Ok(());
  }

  // Build the manager in raw-only mode so we focus on raw frames per connection
  let mut manager = KiteTickerManagerBuilder::new(api_key, access_token)
    .raw_only(true) // skip high-level parsing; emit raw frames only
    .max_connections(3)
    .max_symbols_per_connection(3000)
    .connection_buffer_size(10_000)
    .parser_buffer_size(20_000)
    .default_mode(Mode::Full)
    .build();

  println!("📡 Starting manager in raw-only mode...");
  let start_time = Instant::now();
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

  // A modest symbol set for demo; adjust as needed
  let symbols = vec![256265, 265, 260105, 738561]; // NIFTY 50, Sensex, BankNifty, INFY
  manager
    .subscribe_symbols(&symbols, Some(Mode::Full))
    .await?;
  println!("✅ Subscribed to {} symbols (Full)", symbols.len());

  // Get raw frame receivers for every active connection
  let channels = manager.get_all_raw_frame_channels();
  println!(
    "🔀 Created {} raw frame channels (one per connection)",
    channels.len()
  );

  // Spawn a task per connection to parse length-prefixed packets and peek into Full/Index/Header
  let mut tasks = Vec::new();
  for (id, mut rx) in channels {
    let task = tokio::spawn(async move {
      let mut frames_seen: u64 = 0;
      let mut packets_seen: u64 = 0;
      let mut full_ticks_seen: u64 = 0;
      let mut index_seen: u64 = 0;
      let mut inst_seen: u64 = 0;
      let mut first_samples = 0usize;
      let started = Instant::now();

      println!("🎯 Listening for raw frames on {:?}", id);

      loop {
        match rx.recv().await {
          Ok(frame) => {
            frames_seen += 1;
            if frame.len() < 2 {
              continue;
            }

            let num_packets = u16::from_be_bytes([frame[0], frame[1]]) as usize;
            let mut off = 2usize;

            for i in 0..num_packets {
              if off + 2 > frame.len() {
                break;
              }
              let pkt_len =
                u16::from_be_bytes([frame[off], frame[off + 1]]) as usize;
              let body_start = off + 2;
              let end = body_start + pkt_len;
              if end > frame.len() {
                break;
              }
              let body = frame.slice(body_start..end);
              packets_seen += 1;

              // Print a few early samples per connection for visibility
              if first_samples < 4 {
                if pkt_len >= 4 {
                  let token =
                    u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                  println!(
                    "[{id:?}] frame#{frames_seen} pkt#{i}: token={token} size={pkt_len}"
                  );
                } else {
                  println!(
                    "[{id:?}] frame#{frames_seen} pkt#{i}: size={pkt_len}"
                  );
                }
                first_samples += 1;
              }

              // Peek into specific structures
              if pkt_len == 184 {
                if let Some(view_ref) = as_tick_raw(&body) {
                  let v = &*view_ref; // &TickRaw
                  let token = v.header.instrument_token.get();
                  let ltp = v.header.last_price.get();
                  let b0_qty = v.depth.buy[0].qty.get();
                  let b0_price = v.depth.buy[0].price.get();
                  full_ticks_seen += 1;
                  if full_ticks_seen <= 2 {
                    println!(
                      "[{id:?}] FULL: token={token} ltp_scaled={ltp} bid0_qty={b0_qty} bid0_price_scaled={b0_price}"
                    );
                  }
                }
              } else if pkt_len == INDEX_QUOTE_SIZE {
                if let Some(v) = as_index_quote_32(&body) {
                  let v = &*v;
                  index_seen += 1;
                  if index_seen <= 2 {
                    println!(
                      "[{id:?}] INDEX32: token={} ltp={} ohlc=({},{},{},{}) change={} exch_ts={}",
                      v.token.get(),
                      v.ltp.get(),
                      v.open.get(),
                      v.high.get(),
                      v.low.get(),
                      v.close.get(),
                      v.price_change.get(),
                      v.exch_ts.get()
                    );
                  }
                }
              } else if pkt_len == INST_HEADER_SIZE {
                if let Some(v) = as_inst_header_64(&body) {
                  let v = &*v;
                  inst_seen += 1;
                  if inst_seen <= 2 {
                    println!(
                      "[{id:?}] INST64: token={} ltp={} vol={} ohlc=({},{},{},{}) exch_ts={}",
                      v.instrument_token.get(),
                      v.ltp.get(),
                      v.vol.get(),
                      v.open.get(),
                      v.high.get(),
                      v.low.get(),
                      v.close.get(),
                      v.exch_ts.get()
                    );
                  }
                }
              }

              off = end;
            }

            // Lightweight periodic stats every ~10s
            if started.elapsed().as_secs() % 10 == 0 && frames_seen % 50 == 0 {
              let secs = started.elapsed().as_secs_f64();
              println!(
                "[{id:?}] 📊 frames={} ({:.1}/s) packets={} full={} index={} inst={}",
                frames_seen,
                frames_seen as f64 / secs,
                packets_seen,
                full_ticks_seen,
                index_seen,
                inst_seen
              );
            }
          }
          Err(e) => {
            println!("[{id:?}] ❌ raw channel error: {}", e);
            break;
          }
        }
      }
    });
    tasks.push((id, task));
  }

  // Run for a short demo period or until Ctrl+C
  println!(
    "\n📈 Monitoring raw frames for 45 seconds (Ctrl+C to stop early)..."
  );
  tokio::select! {
    _ = sleep(Duration::from_secs(45)) => {
      println!("\n⏰ Demo duration completed");
    }
    _ = tokio::signal::ctrl_c() => {
      println!("\n🛑 Received Ctrl+C, stopping...");
    }
  }

  // Stop consumers
  for (_, t) in &tasks {
    t.abort();
  }

  // Gracefully stop manager
  println!("\n🛑 Stopping manager...");
  manager.stop().await?;

  // Small join attempt (non-fatal if tasks already aborted)
  for (id, t) in tasks {
    if let Err(e) = t.await {
      if !e.is_cancelled() {
        println!("[{id:?}] Join error: {e:?}");
      }
    }
  }

  println!("🏁 Raw manager demo completed!");
  Ok(())
}
