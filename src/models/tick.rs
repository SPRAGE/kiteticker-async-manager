use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{
  errors::ParseTickError,
  parser::{price, value},
  Depth, Exchange, Mode, OHLC,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
///
/// Quote packet structure
///
pub struct Tick {
  pub mode: Mode,
  pub instrument_token: u32,
  pub exchange: Exchange,
  pub is_tradable: bool,
  pub is_index: bool,

  pub last_traded_qty: Option<u32>,
  pub avg_traded_price: Option<f64>,
  pub last_price: Option<f64>,
  pub volume_traded: Option<u32>,
  pub total_buy_qty: Option<u32>,
  pub total_sell_qty: Option<u32>,
  pub ohlc: Option<OHLC>,

  pub last_traded_timestamp: Option<Duration>,
  pub oi: Option<u32>,
  pub oi_day_high: Option<u32>,
  pub oi_day_low: Option<u32>,
  pub exchange_timestamp: Option<Duration>,

  pub net_change: Option<f64>,
  pub depth: Option<Depth>,
}

impl Tick {
  fn set_instrument_token(&mut self, input: &[u8]) -> &mut Self {
    self.instrument_token = value(&input[0..=3]).unwrap();
    self.exchange = ((self.instrument_token & 0xFF) as usize).into();
    self
  }

  fn set_change(&mut self) -> &mut Self {
    self.net_change = self
      .ohlc
      .as_ref()
      .map(|o| o.close)
      .map(|close_price| {
        if let Some(last_price) = self.last_price {
          if close_price == 0_f64 {
            None
          } else {
            // Some(((last_price - close_price) * 100.0).div(close_price))
            Some(last_price - close_price)
          }
        } else {
          None
        }
      })
      .unwrap_or_default();
    self
  }
}

impl Tick {
  pub(crate) fn from_bytes(input: &[u8]) -> Self {
    let mut tick = Tick::default();
    // Parse LTP fields (first 8 bytes)
    tick.set_instrument_token(input);
    if let Some(bs) = input.get(4..8) {
      tick.mode = Mode::LTP;
      tick.last_price = price(bs, &tick.exchange);
    }

    let is_index = !tick.exchange.is_tradable();
    tick.is_index = is_index;
    tick.is_tradable = !is_index;

    // Parse Quote section
    if is_index {
      if let Some(bs) = input.get(8..28) {
        tick.mode = Mode::Quote;
        // 8 - 24 bytes : ohlc (indices use HLOC order)
        tick.ohlc = OHLC::from_index(&bs[0..16], &tick.exchange);
        // 24 - 28 bytes : price change (provided only for indices)
        tick.net_change = price(&bs[16..20], &tick.exchange);
      }
    } else if let Some(bs) = input.get(8..44) {
      tick.mode = Mode::Quote;
      // 8 - 12 bytes : last traded quantity
      tick.last_traded_qty = value(&bs[0..4]);
      // 12 - 16 bytes : avg traded price
      tick.avg_traded_price = price(&bs[4..8], &tick.exchange);
      // 16 - 20 bytes : volume traded today
      tick.volume_traded = value(&bs[8..12]);
      // 20 - 24 bytes : total buy quantity
      tick.total_buy_qty = value(&bs[12..16]);
      // 24 - 28 bytes : total sell quantity
      tick.total_sell_qty = value(&bs[16..20]);
      // 28 - 44 bytes : ohlc
      tick.ohlc = OHLC::from(&bs[20..36], &tick.exchange);
    }

    // Parse Full section
    if is_index {
      if let Some(bs) = input.get(28..32) {
        tick.mode = Mode::Full;
        // 28 - 32 bytes : exchange time
        tick.exchange_timestamp =
          value(bs).map(|x| Duration::from_secs(x.into()));
      }
    } else if let Some(bs) = input.get(44..184) {
      tick.mode = Mode::Full;
      tick.set_change();

      // 44 - 48 bytes : last traded timestamp
      tick.last_traded_timestamp =
        value(&bs[0..4]).map(|x| Duration::from_secs(x.into()));

      // 48 - 52 bytes : oi
      tick.oi = value(&bs[4..8]);
      // 52 - 56 bytes : oi day high
      tick.oi_day_high = value(&bs[8..12]);
      // 56 - 60 bytes : oi day low
      tick.oi_day_low = value(&bs[12..16]);
      // 60 - 64 bytes : exchange time
      tick.exchange_timestamp =
        value(&bs[16..20]).map(|x| Duration::from_secs(x.into()));
      // 64 - 184 bytes : market depth
      tick.depth = Depth::from(&bs[20..140], &tick.exchange);
    }

    tick
  }
}

impl TryFrom<&[u8]> for Tick {
  type Error = ParseTickError;
  fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
    match value.len() {
      8 | 28 | 32 | 44 | 184 => Ok(Tick::from_bytes(value)),
      len => Err(ParseTickError(format!("invalid tick size: {}", len))),
    }
  }
}
