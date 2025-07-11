use crate::{Order, TextMessage, TickMessage};
use serde::{Deserialize, Serialize};

use super::text_message::TextMessageType;

#[derive(Debug, Clone, Serialize, Deserialize)]
///
/// Parsed message from websocket
///
pub enum TickerMessage {
  /// Quote packets for subscribed tokens
  Ticks(Vec<TickMessage>),
  /// Error response
  Error(String),
  /// Order postback
  OrderPostback(Result<Order, String>),
  /// Messages and alerts from broker
  Message(serde_json::Value),
  /// Websocket closing frame
  ClosingMessage(serde_json::Value),
}

impl From<TextMessage> for TickerMessage {
  fn from(value: TextMessage) -> Self {
    let message_type: TextMessageType = value.message_type.into();
    match message_type {
      TextMessageType::Order => Self::OrderPostback(
        serde_json::from_value(value.data).map_err(|e| e.to_string()),
      ),
      TextMessageType::Error => Self::Error(value.data.to_string()),
      TextMessageType::Message => Self::Message(value.data),
    }
  }
}
