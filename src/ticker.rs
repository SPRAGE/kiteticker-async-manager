use crate::models::{
  Mode, Request, TextMessage, Tick, TickMessage, TickerMessage,
};
use crate::parser::packet_length;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Bounded capacity for reader -> parser channel to avoid unbounded memory growth
const PARSE_CHANNEL_CAP: usize = 4096;

#[derive(Debug)]
///
/// The WebSocket client for connecting to Kite Connect's streaming quotes service.
///
pub struct KiteTickerAsync {
  #[allow(dead_code)]
  api_key: String,
  #[allow(dead_code)]
  access_token: String,
  cmd_tx: Option<mpsc::UnboundedSender<Message>>,
  msg_tx: broadcast::Sender<TickerMessage>,
  raw_tx: broadcast::Sender<Bytes>, // raw binary frames
  #[allow(dead_code)]
  raw_only: bool, // if true, skip parsing and emit raw frames as TickerMessage::Raw
  writer_handle: Option<JoinHandle<()>>,
  reader_handle: Option<JoinHandle<()>>,
  parser_handle: Option<JoinHandle<()>>,
}

impl KiteTickerAsync {
  /// Establish a connection with the Kite WebSocket server
  pub async fn connect(
    api_key: &str,
    access_token: &str,
  ) -> Result<Self, String> {
    Self::connect_with_options(api_key, access_token, false).await
  }

  /// Connect with options
  pub async fn connect_with_options(
    api_key: &str,
    access_token: &str,
    raw_only: bool,
  ) -> Result<Self, String> {
    // Build URL with proper percent-encoding of query params
    let mut url = url::Url::parse("wss://ws.kite.trade")
      .map_err(|e| format!("Invalid base URL: {}", e))?;
    {
      let mut qp = url.query_pairs_mut();
      qp.append_pair("api_key", api_key);
      qp.append_pair("access_token", access_token);
    }
    // tokio-tungstenite >=0.27 accepts types implementing IntoClientRequest (Url is fine)
    let (ws_stream, _resp) =
      connect_async(url.as_str()).await.map_err(|e| match e {
        tokio_tungstenite::tungstenite::Error::Http(response) => {
          // Provide clearer context for HTTP handshake failures
          let status = response.status();
          let reason = status.canonical_reason().unwrap_or("");
          format!(
            "HTTP error during WebSocket handshake: {} {}",
            status, reason
          )
        }
        other => other.to_string(),
      })?;

    let (write_half, mut read_half) = ws_stream.split();

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Message>();
    // Increase buffer size for high-frequency tick data
    let (msg_tx, _) = broadcast::channel(1000);
    let (raw_tx, _) = broadcast::channel(1000);
    let mut write = write_half;
    let writer_handle = tokio::spawn(async move {
      while let Some(msg) = cmd_rx.recv().await {
        if write.send(msg).await.is_err() {
          break;
        }
      }
    });

    // Channel to decouple read and parse so the websocket stream isn't blocked by parsing.
    // Use a bounded channel with try_send to provide lightweight backpressure under bursts.
    let (parse_tx, mut parse_rx) = mpsc::channel::<Message>(PARSE_CHANNEL_CAP);

    // Reader: only forward messages into parse channel, avoid heavy work here
    let msg_sender_for_reader = msg_tx.clone();
    let reader_handle = tokio::spawn(async move {
      while let Some(message) = read_half.next().await {
        match message {
          Ok(msg) => {
            // Forward to parser using non-blocking try_send; if channel is full, drop frame
            match parse_tx.try_send(msg) {
              Ok(_) => {}
              Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                log::warn!(
                  "Reader: parse channel full, dropping incoming frame"
                );
                // Drop and continue to keep read loop unblocked
              }
              Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                // Parser task gone; exit reader
                break;
              }
            }
          }
          Err(e) => {
            // Send error and continue trying to read
            let error_msg =
              TickerMessage::Error(format!("WebSocket error: {}", e));
            let _ = msg_sender_for_reader.send(error_msg);
            if matches!(
              e,
              tokio_tungstenite::tungstenite::Error::ConnectionClosed
                | tokio_tungstenite::tungstenite::Error::AlreadyClosed
            ) {
              break;
            }
          }
        }
      }
    });

    // Parser: processes messages from the channel and publishes results
    let msg_sender = msg_tx.clone();
    let raw_sender = raw_tx.clone();
    let parser_handle = tokio::spawn(async move {
      let raw_only_mode = raw_only; // capture
      while let Some(msg) = parse_rx.recv().await {
        if let Some(processed) =
          process_message(msg, &raw_sender, raw_only_mode)
        {
          let _ = msg_sender.send(processed);
        }
      }
    });

    Ok(KiteTickerAsync {
      api_key: api_key.to_string(),
      access_token: access_token.to_string(),
      cmd_tx: Some(cmd_tx),
      msg_tx,
      raw_tx,
      raw_only,
      writer_handle: Some(writer_handle),
      reader_handle: Some(reader_handle),
      parser_handle: Some(parser_handle),
    })
  }

  /// Subscribes the client to a list of instruments
  pub async fn subscribe(
    &mut self,
    instrument_tokens: &[u32],
    mode: Option<Mode>,
  ) -> Result<KiteTickerSubscriber, String> {
    self.subscribe_cmd(instrument_tokens, mode.as_ref()).await?;
    let default_mode = mode.unwrap_or_default();
    let st = instrument_tokens
      .iter()
      .map(|&t| (t, default_mode))
      .collect();

    let rx = self.msg_tx.subscribe();
    Ok(KiteTickerSubscriber {
      subscribed_tokens: st,
      rx,
      cmd_tx: self.cmd_tx.clone().map(Arc::new),
    })
  }

  /// Close the websocket connection
  pub async fn close(&mut self) -> Result<(), String> {
    if let Some(tx) = self.cmd_tx.take() {
      let _ = tx.send(Message::Close(None));
    }
    if let Some(handle) = self.writer_handle.take() {
      handle.await.map_err(|e| e.to_string())?;
    }
    if let Some(handle) = self.reader_handle.take() {
      handle.await.map_err(|e| e.to_string())?;
    }
    if let Some(handle) = self.parser_handle.take() {
      handle.await.map_err(|e| e.to_string())?;
    }
    Ok(())
  }

  async fn subscribe_cmd(
    &mut self,
    instrument_tokens: &[u32],
    mode: Option<&Mode>,
  ) -> Result<(), String> {
    let mode_value = mode.cloned().unwrap_or_default();
    let msgs = vec![
      Message::Text(Request::subscribe(instrument_tokens).to_string().into()),
      Message::Text(
        Request::mode(mode_value, instrument_tokens)
          .to_string()
          .into(),
      ),
    ];

    for msg in msgs {
      if let Some(tx) = &self.cmd_tx {
        tx.send(msg).map_err(|e| e.to_string())?;
      }
    }

    Ok(())
  }

  // internal helpers removed after refactor; operations now issued via subscriber command handle

  /// Check if the connection is still alive
  pub fn is_connected(&self) -> bool {
    self.cmd_tx.is_some()
      && self
        .writer_handle
        .as_ref()
        .is_some_and(|h| !h.is_finished())
      && self
        .reader_handle
        .as_ref()
        .is_some_and(|h| !h.is_finished())
  }

  /// Send a ping to keep the connection alive
  pub async fn ping(&mut self) -> Result<(), String> {
    if let Some(tx) = &self.cmd_tx {
      tx.send(Message::Ping(bytes::Bytes::new()))
        .map_err(|e| e.to_string())?;
      Ok(())
    } else {
      Err("Connection is closed".to_string())
    }
  }

  /// Get the current broadcast channel receiver count
  pub fn receiver_count(&self) -> usize {
    self.msg_tx.receiver_count()
  }

  /// Get the current broadcast channel capacity
  pub fn channel_capacity(&self) -> usize {
    // The broadcast channel doesn't expose capacity directly,
    // but we can estimate based on our configuration
    1000 // This matches our increased buffer size
  }

  /// Subscribe to raw binary frames (zero-copy). Each item is the full websocket frame bytes.
  ///
  /// Use this to implement custom parsing or zero-copy peeking on packet bodies.
  /// Each emitted item is a `bytes::Bytes` that shares the underlying frame buffer (clone is cheap).
  ///
  /// See the crate-level docs for an end-to-end example of slicing packet bodies from a frame.
  pub fn subscribe_raw_frames(&self) -> broadcast::Receiver<Bytes> {
    self.raw_tx.subscribe()
  }

  /// Backward-compatible alias for subscribe_raw_frames.
  #[deprecated(
    note = "use subscribe_raw_frames() instead; now returns bytes::Bytes"
  )]
  pub fn subscribe_raw(&self) -> broadcast::Receiver<Bytes> {
    self.subscribe_raw_frames()
  }

  /// Create a subscriber that yields only 184-byte Full tick payloads sliced from frames.
  ///
  /// The returned subscriber exposes convenience methods to receive raw `Bytes`, a fixed `[u8;184]`
  /// reference, or a `zerocopy::Ref<&[u8], TickRaw>` view via `recv_raw_tickraw`.
  ///
  /// Note: the typed `Ref` returned by `recv_raw_tickraw` is valid until the next method call that
  /// overwrites the internal buffer. If you need to hold onto the data longer, clone the `Bytes` and
  /// re-create the view as needed using `as_tick_raw`.
  pub fn subscribe_full_raw(&self) -> KiteTickerRawSubscriber184 {
    KiteTickerRawSubscriber184 {
      rx: self.raw_tx.subscribe(),
      last_payload: None,
    }
  }

  /// Get a clone of the internal command sender for incremental ops
  pub fn command_sender(&self) -> Option<mpsc::UnboundedSender<Message>> {
    self.cmd_tx.clone()
  }
}

#[derive(Debug)]
///
/// The Websocket client that entered in a pub/sub mode once the client subscribed to a list of instruments
///
pub struct KiteTickerSubscriber {
  // Now independent of owning the ticker; commands go through channel retained in KiteTickerAsync
  subscribed_tokens: HashMap<u32, Mode>,
  rx: broadcast::Receiver<TickerMessage>,
  cmd_tx: Option<Arc<mpsc::UnboundedSender<Message>>>,
}

impl KiteTickerSubscriber {
  /// Get the list of subscribed instruments
  pub fn get_subscribed(&self) -> Vec<u32> {
    self
      .subscribed_tokens
      .clone()
      .into_keys()
      .collect::<Vec<_>>()
  }

  /// get all tokens common between subscribed tokens and input tokens
  /// and if the input is empty then all subscribed tokens will be unsubscribed
  fn get_subscribed_or(&self, tokens: &[u32]) -> Vec<u32> {
    if tokens.is_empty() {
      self.get_subscribed()
    } else {
      tokens
        .iter()
        .filter(|t| self.subscribed_tokens.contains_key(t))
        .copied()
        .collect::<Vec<_>>()
    }
  }

  /// Subscribe to new tokens
  pub async fn subscribe(
    &mut self,
    tokens: &[u32],
    mode: Option<Mode>,
  ) -> Result<(), String> {
    // Only send incremental subscribe for new tokens
    let default_mode = mode.unwrap_or_default();
    let mut new_tokens: Vec<u32> = Vec::new();
    for &t in tokens {
      if let std::collections::hash_map::Entry::Vacant(e) =
        self.subscribed_tokens.entry(t)
      {
        e.insert(default_mode);
        new_tokens.push(t);
      }
    }
    if new_tokens.is_empty() {
      return Ok(());
    }
    if let Some(tx) = &self.cmd_tx {
      // send subscribe
      let _ = tx.send(Message::Text(
        Request::subscribe(&new_tokens).to_string().into(),
      ));
      if mode.is_some() {
        let _ = tx.send(Message::Text(
          Request::mode(default_mode, &new_tokens).to_string().into(),
        ));
      }
    }
    Ok(())
  }

  /// Change the mode of the subscribed instrument tokens
  pub async fn set_mode(
    &mut self,
    instrument_tokens: &[u32],
    mode: Mode,
  ) -> Result<(), String> {
    let tokens = self.get_subscribed_or(instrument_tokens);
    if tokens.is_empty() {
      return Ok(());
    }
    if let Some(tx) = &self.cmd_tx {
      let _ = tx.send(Message::Text(
        Request::mode(mode, &tokens).to_string().into(),
      ));
    }
    Ok(())
  }

  /// Unsubscribe provided subscribed tokens, if input is empty then all subscribed tokens will unsubscribed
  ///
  /// Tokens in the input which are not part of the subscribed tokens will be ignored.
  pub async fn unsubscribe(
    &mut self,
    instrument_tokens: &[u32],
  ) -> Result<(), String> {
    let tokens = self.get_subscribed_or(instrument_tokens);
    if tokens.is_empty() {
      return Ok(());
    }
    if let Some(tx) = &self.cmd_tx {
      let _ = tx.send(Message::Text(
        Request::unsubscribe(&tokens).to_string().into(),
      ));
    }
    self.subscribed_tokens.retain(|k, _| !tokens.contains(k));
    Ok(())
  }

  /// Get the next message from the server, waiting if necessary.
  /// If the result is None then server is terminated
  pub async fn next_message(
    &mut self,
  ) -> Result<Option<TickerMessage>, String> {
    match self.rx.recv().await {
      Ok(msg) => Ok(Some(msg)),
      Err(broadcast::error::RecvError::Closed) => Ok(None),
      Err(e) => Err(e.to_string()),
    }
  }

  pub async fn close(&mut self) -> Result<(), String> {
    Ok(())
  }
}

fn process_message(
  message: Message,
  raw_sender: &broadcast::Sender<Bytes>,
  raw_only: bool,
) -> Option<TickerMessage> {
  match message {
    Message::Text(text_message) => {
      process_text_message(text_message.to_string())
    }
    Message::Binary(binary_message) => {
      // Convert once to Bytes to avoid cloning the Vec for raw subscribers
      let bytes = binary_message;
      let slice: &[u8] = &bytes;
      // publish raw first (cheap clone)
      let _ = raw_sender.send(bytes.clone());
      if raw_only {
        // In raw-only mode, rely solely on raw_tx broadcast to deliver zero-copy frames.
        // Do not emit a TickerMessage to avoid extra allocations or duplicates.
        return None;
      }
      // Drop 1-byte heartbeat frames per protocol (no downstream churn)
      if slice.len() < 2 {
        None
      } else {
        process_binary(slice)
      }
    }
    Message::Close(closing_message) => closing_message.map(|c| {
      TickerMessage::ClosingMessage(json!({
        "code": c.code.to_string(),
        "reason": c.reason.to_string()
      }))
    }),
    Message::Ping(_) => None,
    Message::Pong(_) => None,
    Message::Frame(_) => None,
  }
}

#[derive(Debug)]
/// Subscriber that yields raw 184-byte payloads (Mode::Full) extracted from incoming frames.
pub struct KiteTickerRawSubscriber184 {
  rx: broadcast::Receiver<Bytes>,
  // Keep last payload alive for reference-returning APIs
  last_payload: Option<Bytes>,
}

impl KiteTickerRawSubscriber184 {
  /// Receive the next 184-byte payload, if any frame contains it. Skips non-Full packets.
  /// Returns Bytes that points to the underlying frame memory (zero-copy); slice is cloned out.
  pub async fn recv_raw(&mut self) -> Result<Option<Bytes>, String> {
    loop {
      match self.rx.recv().await {
        Ok(frame) => {
          if let Some(bytes) = extract_first_full_payload(&frame) {
            self.last_payload = Some(bytes.clone());
            return Ok(Some(bytes));
          }
          // else keep looping for next frame
        }
        Err(broadcast::error::RecvError::Closed) => return Ok(None),
        Err(e) => return Err(e.to_string()),
      }
    }
  }

  /// Receive next payload and return a reference to a fixed 184-byte array.
  /// The reference remains valid until the next call that overwrites internal buffer.
  pub async fn recv_raw_ref(&mut self) -> Result<Option<&[u8; 184]>, String> {
    use crate::tick_as_184 as as_184;
    match self.recv_raw().await? {
      Some(bytes) => {
        // Store to keep alive, then take a ref from stored bytes
        self.last_payload = Some(bytes);
        if let Some(ref b) = self.last_payload {
          Ok(as_184(b))
        } else {
          Ok(None)
        }
      }
      None => Ok(None),
    }
  }

  /// Receive next payload and return a zero-copy typed view `TickRaw`.
  ///
  /// Returns `Some(zerocopy::Ref<&[u8], TickRaw>)` for a Full packet body (184 bytes), otherwise waits.
  /// The `Ref` dereferences to `&TickRaw` and stays valid until another method call that replaces
  /// the internal `Bytes` buffer.
  pub async fn recv_raw_tickraw(
    &mut self,
  ) -> Result<Option<zerocopy::Ref<&[u8], crate::TickRaw>>, String> {
    use crate::as_tick_raw;
    match self.recv_raw().await? {
      Some(bytes) => {
        self.last_payload = Some(bytes.clone());
        if let Some(ref b) = self.last_payload {
          Ok(as_tick_raw(b))
        } else {
          Ok(None)
        }
      }
      None => Ok(None),
    }
  }

  /// Receive up to `max` 184-byte payloads from the next frame(s). This avoids per-packet awaits.
  pub async fn recv_batch_raw(
    &mut self,
    max: usize,
  ) -> Result<Vec<Bytes>, String> {
    let mut out = Vec::with_capacity(max.max(1));
    while out.len() < max {
      match self.rx.recv().await {
        Ok(frame) => {
          extract_all_full_payloads(&frame, max - out.len(), &mut out);
          if out.len() >= max {
            break;
          }
          // continue to next frame if more needed
        }
        Err(broadcast::error::RecvError::Closed) => break,
        Err(e) => return Err(e.to_string()),
      }
    }
    Ok(out)
  }
}

#[inline]
fn extract_first_full_payload(frame: &Bytes) -> Option<Bytes> {
  if frame.len() < 2 {
    return None;
  }
  let mut start = 2usize;
  let num_packets = u16::from_be_bytes([frame[0], frame[1]]) as usize;
  for _ in 0..num_packets {
    if start + 2 > frame.len() {
      return None;
    }
    let packet_len = packet_length(&frame[start..start + 2]);
    let body_start = start + 2;
    let next_start = body_start + packet_len;
    if next_start > frame.len() {
      return None;
    }
    if packet_len == 184 {
      // slice reference into Bytes
      return Some(frame.slice(body_start..next_start));
    }
    start = next_start;
  }
  None
}

#[inline]
fn extract_all_full_payloads(
  frame: &Bytes,
  limit: usize,
  out: &mut Vec<Bytes>,
) {
  if frame.len() < 2 || limit == 0 {
    return;
  }
  let mut start = 2usize;
  let num_packets = u16::from_be_bytes([frame[0], frame[1]]) as usize;
  let mut cnt = 0usize;
  for _ in 0..num_packets {
    if cnt >= limit {
      break;
    }
    if start + 2 > frame.len() {
      break;
    }
    let packet_len = packet_length(&frame[start..start + 2]);
    let body_start = start + 2;
    let next_start = body_start + packet_len;
    if next_start > frame.len() {
      break;
    }
    if packet_len == 184 {
      out.push(frame.slice(body_start..next_start));
      cnt += 1;
      if cnt >= limit {
        break;
      }
    }
    start = next_start;
  }
}

fn process_binary(binary_message: &[u8]) -> Option<TickerMessage> {
  if binary_message.len() < 2 {
    return None;
  }
  let num_packets =
    u16::from_be_bytes([binary_message[0], binary_message[1]]) as usize;
  if num_packets > 0 {
    let mut start = 2;
    // Inline small optimization: most frames contain modest number of ticks
    let mut ticks: SmallVec<[TickMessage; 32]> =
      SmallVec::with_capacity(num_packets.min(32));
    let mut had_error = false;
    for _ in 0..num_packets {
      if start + 2 > binary_message.len() {
        had_error = true;
        break;
      }
      let packet_len = packet_length(&binary_message[start..start + 2]);
      let next_start = start + 2 + packet_len;
      if next_start > binary_message.len() {
        had_error = true;
        break;
      }
      match Tick::try_from(&binary_message[start + 2..next_start]) {
        Ok(tick) => ticks.push(TickMessage::new(tick.instrument_token, tick)),
        Err(_e) => {
          // Skip this packet, continue with others
          had_error = true;
        }
      }
      start = next_start;
    }
    if !ticks.is_empty() {
      Some(TickerMessage::Ticks(ticks.into_vec()))
    } else if had_error {
      Some(TickerMessage::Error(
        "Failed to parse tick(s) in frame".to_string(),
      ))
    } else {
      None
    }
  } else {
    None
  }
}

fn process_text_message(text_message: String) -> Option<TickerMessage> {
  serde_json::from_str::<TextMessage>(&text_message)
    .map(|x| x.into())
    .ok()
}
