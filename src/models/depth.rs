use crate::Exchange;
use serde::{Deserialize, Serialize};

use crate::parser::{price, value, value_short};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
///
/// Market depth packet structure
///
pub struct Depth {
  pub buy: [DepthItem; 5],
  pub sell: [DepthItem; 5],
}

impl Depth {
  pub(crate) fn from(input: &[u8], exchange: &Exchange) -> Option<Self> {
    if let Some(bs) = input.get(0..120) {
      let parse_depth_item = |v: &[u8], start: usize| {
        v.get(start..start + 10)
          .and_then(|xs| DepthItem::from(xs, exchange))
          .unwrap_or_default()
      };
      let mut depth = Depth::default();
      for i in 0..5 {
        let start = i * 12;
        depth.buy[i] = parse_depth_item(bs, start)
      }
      for i in 0..5 {
        let start = 60 + i * 12;
        depth.sell[i] = parse_depth_item(bs, start);
      }

      Some(depth)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
///
/// Structure for each market depth entry
///
pub struct DepthItem {
  pub qty: u32,
  pub price: f64,
  pub orders: u16,
}

impl DepthItem {
  pub fn from(input: &[u8], exchange: &Exchange) -> Option<Self> {
    input.get(0..10).map(|bs| DepthItem {
      qty: value(&bs[0..=3]).unwrap(),
      price: price(&bs[4..=7], exchange).unwrap(),
      orders: value_short(&bs[8..=9]).unwrap(),
    })
  }
}
