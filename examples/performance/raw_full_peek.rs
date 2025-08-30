use kiteticker_async_manager::{
  as_index_quote_32, as_inst_header_64, as_tick_raw, KiteTickerAsync, Mode,
  INDEX_QUOTE_SIZE, INST_HEADER_SIZE,
};

#[tokio::main]
async fn main() -> Result<(), String> {
  let api_key = std::env::var("KITE_API_KEY").unwrap_or_default();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap_or_default();
  if api_key.is_empty() || access_token.is_empty() {
    eprintln!("Set KITE_API_KEY and KITE_ACCESS_TOKEN to run this example.");
    return Ok(());
  }

  let mut ticker =
    KiteTickerAsync::connect_with_options(&api_key, &access_token, true)
      .await?;
  let mut sub = ticker
    .subscribe(&[256265, 265, 260105, 738561], Some(Mode::Full))
    .await?; // NIFTY 50, Sensex, Bank Nifty

  // Attach raw full-frame subscriber (we'll extract packets of any size)
  let mut frames = ticker.subscribe_raw_frames();

  // Receive a few frames and print packet summaries; show TickRaw when 184-byte payload arrives
  for _ in 0..3 {
    let frame = match frames.recv().await {
      Ok(f) => f,
      Err(e) => {
        eprintln!("raw frame channel error: {}", e);
        break;
      }
    };
    if frame.len() < 2 {
      continue;
    }
    let num_packets = u16::from_be_bytes([frame[0], frame[1]]) as usize;
    let mut start = 2usize;
    for i in 0..num_packets {
      if start + 2 > frame.len() {
        break;
      }
      let pkt_len =
        u16::from_be_bytes([frame[start], frame[start + 1]]) as usize;
      let body_start = start + 2;
      let end = body_start + pkt_len;
      if end > frame.len() {
        break;
      }
      let body = frame.slice(body_start..end);
      // Print minimal info
      if pkt_len >= 4 {
        let token = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
        println!("frame packet#{i}: token={token} size={pkt_len}");
      } else {
        println!("frame packet#{i}: size={pkt_len}");
      }
      // If it's a Full 184-byte payload, show a few TickRaw fields too
      if pkt_len == 184 {
        if let Some(view_ref) = as_tick_raw(&body) {
          let view = &*view_ref;
          let token = view.header.instrument_token.get();
          let ltp_scaled = view.header.last_price.get();
          let b0_qty = view.depth.buy[0].qty.get();
          let b0_price = view.depth.buy[0].price.get();
          println!(
            "FULL: token={} ltp_scaled={} bid0_qty={} bid0_price_scaled={}",
            token, ltp_scaled, b0_qty, b0_price
          );
        }
      }
      if pkt_len == INDEX_QUOTE_SIZE {
        if let Some(v_ref) = as_index_quote_32(&body) {
          let v = &*v_ref;
          println!(
            "INDEX32: token={} ltp={} ohlc=({},{},{},{}) price_change={} exch_ts={}",
            v.token.get(), v.ltp.get(), v.open.get(), v.high.get(), v.low.get(), v.close.get(),
            v.price_change.get(), v.exch_ts.get()
          );
        }
      }
      if pkt_len == INST_HEADER_SIZE {
        if let Some(v_ref) = as_inst_header_64(&body) {
          let v = &*v_ref;
          println!(
            "INST64: token={} ltp={} vol={} ohlc=({},{},{},{}) exch_ts={}",
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
      start = end;
    }
  }

  sub.close().await?;
  ticker.close().await?;
  Ok(())
}
