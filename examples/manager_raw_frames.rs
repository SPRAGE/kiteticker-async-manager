use bytes::Bytes;
use kiteticker_async_manager::{
    as_tick_raw,
    ChannelId,
    KiteTickerManagerBuilder,
    Mode,
};

/// Example: Using the manager to consume raw WebSocket frames (zero-copy) per-connection
/// and printing fields from 184-byte Full packets using a typed view.
#[tokio::main]
async fn main() -> Result<(), String> {
    // Credentials from environment
    let api_key = std::env::var("KITE_API_KEY")
        .map_err(|_| "Missing KITE_API_KEY env var".to_string())?;
    let access_token = std::env::var("KITE_ACCESS_TOKEN")
        .map_err(|_| "Missing KITE_ACCESS_TOKEN env var".to_string())?;

    // Build manager in raw-only mode (no parsed messages, raw frames only)
    let mut manager = KiteTickerManagerBuilder::new(api_key, access_token)
        .max_connections(3)
        .raw_only(true)
        .build();

    manager.start().await?;

    // Subscribe a few tokens in Full mode so 184-byte payloads arrive
    // Example tokens: NIFTY 50 (256265), HDFC Bank (341249), Reliance (738561)
    let symbols = vec![256265u32, 341249u32, 738561u32];
    manager.subscribe_symbols(&symbols, Some(Mode::Full)).await?;

    // Get raw frame receivers for all initialized connections
    let channels = manager.get_all_raw_frame_channels();
    println!("spawned {} raw channels", channels.len());

    for (id, mut rx) in channels {
      tokio::spawn(async move {
        println!("[{:?}] raw consumer started", id);
        loop {
          match rx.recv().await {
            Ok(frame) => handle_frame(id, &frame),
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
              eprintln!("[{:?}] lagged by {} frames; skipping", id, n);
            }
          }
        }
      });
    }

    // Run for a short while then stop
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    manager.stop().await?;

    Ok(())
}

fn handle_frame(id: ChannelId, frame: &Bytes) {
    if frame.len() < 2 { return; }
    let mut off = 2usize;
    let num = u16::from_be_bytes([frame[0], frame[1]]) as usize;
    for _ in 0..num {
        if off + 2 > frame.len() { break; }
        let len = u16::from_be_bytes([frame[off], frame[off + 1]]) as usize;
        let body_start = off + 2;
        let body_end = body_start + len;
        if body_end > frame.len() { break; }
        let body = frame.slice(body_start..body_end);

        // Print some details for Full payloads (184 bytes)
        if len == kiteticker_async_manager::TICK_FULL_SIZE {
            if let Some(view) = as_tick_raw(&body) {
                let t = &*view;
                let token = t.header.instrument_token.get();
                let ltp_scaled = t.header.last_price.get();
                println!("[{:?}] token={} ltp_scaled={}", id, token, ltp_scaled);
            }
        }

        off = body_end;
    }
}
