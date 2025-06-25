use serde::{Deserialize, Serialize};
use crate::Exchange;

use crate::parser::price;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
///
/// OHLC packet structure
///
pub struct OHLC {
  pub open: f64,
  pub high: f64,
  pub low: f64,
  pub close: f64,
}

impl OHLC {
  pub(crate) fn from(value: &[u8], exchange: &Exchange) -> Option<Self> {
    if let Some(bs) = value.get(0..16) {
      Some(OHLC {
        open: price(&bs[0..=3], exchange).unwrap(),
        high: price(&bs[4..=7], exchange).unwrap(),
        low: price(&bs[8..=11], exchange).unwrap(),
        close: price(&bs[12..=15], exchange).unwrap(),
      })
    } else {
      None
    }
  }

  /// Parse OHLC bytes for index instruments.
  ///
  /// The order of fields for indices is `high`, `low`, `open`, `close`.
  pub(crate) fn from_index(value: &[u8], exchange: &Exchange) -> Option<Self> {
    if let Some(bs) = value.get(0..16) {
      Some(OHLC {
        open: price(&bs[8..=11], exchange).unwrap(),
        high: price(&bs[0..=3], exchange).unwrap(),
        low: price(&bs[4..=7], exchange).unwrap(),
        close: price(&bs[12..=15], exchange).unwrap(),
      })
    } else {
      None
    }
  }
}
