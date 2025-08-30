````markdown
# Ticker API Reference

Low-level WebSocket client for the Kite Connect ticker: `KiteTickerAsync`.

Use this when you want to manage a single connection yourself or when you need raw frames.

## Constructing a client

```rust
use kiteticker_async_manager::KiteTickerAsync;

#[tokio::main]
async fn main() -> Result<(), String> {
  let api_key = std::env::var("KITE_API_KEY").unwrap();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();

  // Normal parsed mode
  let mut ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;

  // Or: raw-only (frames are forwarded without parsing)
  let mut raw_ticker = KiteTickerAsync::connect_with_options(&api_key, &access_token, true).await?;
  Ok(())
}
```

## Subscribing and receiving ticks

```rust
use kiteticker_async_manager::{KiteTickerAsync, Mode, TickerMessage};

#[tokio::main]
async fn main() -> Result<(), String> {
  let api_key = std::env::var("KITE_API_KEY").unwrap();
  let access_token = std::env::var("KITE_ACCESS_TOKEN").unwrap();
  let mut ticker = KiteTickerAsync::connect(&api_key, &access_token).await?;

  let symbols = vec![256265, 408065];
  let mut sub = ticker.subscribe(&symbols, Some(Mode::Quote)).await?;

  while let Ok(Some(msg)) = sub.next_message().await {
    match msg {
      TickerMessage::Ticks(ticks) => {
        for t in ticks { /* handle t.content */ }
      }
      TickerMessage::Error(e) => eprintln!("error: {}", e),
      TickerMessage::ClosingMessage(info) => eprintln!("closing: {}", info),
      TickerMessage::Raw(_) => { /* only in raw-only mode */ }
      TickerMessage::Message(v) => println!("message: {}", v),
      TickerMessage::OrderPostback(_) => { /* optional */ }
    }
  }
  Ok(())
}
```

## Receiving raw frames

Subscribe to a zero-copy raw feed regardless of `raw_only` using:

```rust
let mut raw_rx = ticker.subscribe_raw_frames(); // broadcast::Receiver<bytes::Bytes>
while let Ok(frame) = raw_rx.recv().await {
  // frame is the full WebSocket frame bytes
}
```

## Pinging

Keep the connection alive manually if needed:

```rust
ticker.ping().await?;
```

For multi-connection, prefer the manager and its builder (`KiteTickerManagerBuilder`) to configure `raw_only` globally.
````