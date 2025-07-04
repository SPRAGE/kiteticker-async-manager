use serde::{Deserialize, Serialize};
use std::fmt;

use crate::Mode;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
///
/// Websocket request actions
///
enum RequestActions {
  Subscribe,
  Unsubscribe,
  Mode,
}

#[derive(Clone, Debug)]
///
/// Websocket request data
///
enum RequestData {
  InstrumentTokens(Vec<u32>),
  InstrumentTokensWithMode(Mode, Vec<u32>),
}

// Custom serialization for RequestData
impl serde::Serialize for RequestData {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    match self {
      RequestData::InstrumentTokens(tokens) => tokens.serialize(serializer),
      RequestData::InstrumentTokensWithMode(mode, tokens) => {
        // Serialize as [mode_string, tokens_array] according to Kite docs
        let mode_str = mode.to_websocket_string();
        let tuple = (mode_str, tokens);
        tuple.serialize(serializer)
      }
    }
  }
}

// We don't need custom deserialization for RequestData since it's only used for sending
impl<'de> serde::Deserialize<'de> for RequestData {
  fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    // This is only used for outgoing requests, not parsing responses
    unimplemented!("RequestData deserialization not needed")
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
///
/// Websocket request structure
///
pub struct Request {
  a: RequestActions,
  v: RequestData,
}

impl Request {
  fn new(action: RequestActions, value: RequestData) -> Request {
    Request {
      a: action,
      v: value,
    }
  }

  ///
  /// Subscribe to a list of instrument tokens
  ///
  pub fn subscribe(instrument_tokens: Vec<u32>) -> Request {
    Request::new(
      RequestActions::Subscribe,
      RequestData::InstrumentTokens(instrument_tokens),
    )
  }

  ///
  /// Subscribe to a list of instrument tokens with mode
  ///
  pub fn mode(mode: Mode, instrument_tokens: Vec<u32>) -> Request {
    Request::new(
      RequestActions::Mode,
      RequestData::InstrumentTokensWithMode(mode, instrument_tokens),
    )
  }

  ///
  /// Unsubscribe from a list of instrument tokens
  ///
  pub fn unsubscribe(instrument_tokens: Vec<u32>) -> Request {
    Request::new(
      RequestActions::Unsubscribe,
      RequestData::InstrumentTokens(instrument_tokens),
    )
  }
}

impl fmt::Display for Request {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let json =
      serde_json::to_string(self).expect("failed to serialize Request to JSON");
    write!(f, "{}", json)
  }
}
