use std::ops::Div;

use crate::models::Exchange;

#[inline(always)]
pub(crate) fn value(input: &[u8]) -> Option<u32> {
  if input.len() >= 4 {
    // Safety: we guarded length; copy into array then convert
    Some(u32::from_be_bytes([input[0], input[1], input[2], input[3]]))
  } else {
    None
  }
}

#[inline(always)]
pub(crate) fn value_short(input: &[u8]) -> Option<u16> {
  if input.len() >= 2 {
    Some(u16::from_be_bytes([input[0], input[1]]))
  } else {
    None
  }
}

#[inline(always)]
pub(crate) fn price(input: &[u8], exchange: &Exchange) -> Option<f64> {
  if input.len() >= 4 && exchange.divisor() > 0_f64 {
    let value =
      i32::from_be_bytes([input[0], input[1], input[2], input[3]]) as f64;
    Some(value.div(exchange.divisor()))
  } else {
    None
  }
}

#[inline(always)]
pub(crate) fn packet_length(bs: &[u8]) -> usize {
  if bs.len() >= 2 {
    u16::from_be_bytes([bs[0], bs[1]]) as usize
  } else {
    0
  }
}

// parse_tick inlined into Tick::try_from to remove an extra function call per packet
