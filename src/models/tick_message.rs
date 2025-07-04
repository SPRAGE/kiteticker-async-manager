use crate::Tick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
///
/// Parsed quote packet
///
pub struct TickMessage {
  pub instrument_token: u32,
  pub content: Tick,
}

impl TickMessage {
  pub(crate) fn new(instrument_token: u32, content: Tick) -> Self {
    Self {
      instrument_token,
      content,
    }
  }
}
