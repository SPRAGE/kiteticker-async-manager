use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
///
/// Postbacks and non-binary message types
///
pub(crate) enum TextMessageType {
  /// Order postback
  Order = 1,
  /// Error response
  Error = 2,
  /// Messages and alerts from the broker
  Message = 3,
}

impl From<String> for TextMessageType {
  fn from(value: String) -> Self {
    match value.as_str() {
      "order" => Self::Order,
      "error" => Self::Error,
      _ => Self::Message,
    }
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
///
/// Postback and non-binary message structure
///
pub struct TextMessage {
  #[serde(rename = "type")]
  pub(crate) message_type: String,
  pub(crate) data: serde_json::Value,
}
