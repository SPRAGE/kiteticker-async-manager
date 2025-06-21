use byteorder::{BigEndian, ByteOrder};
use std::ops::Div;

use crate::{
  errors::ParseTickError,
  models::{Exchange, Tick},
};

pub(crate) fn value(input: &[u8]) -> Option<u32> {
  if input.len() >= 4 {
    Some(BigEndian::read_u32(input))
  } else {
    None
  }
}

pub(crate) fn value_short(input: &[u8]) -> Option<u16> {
  if input.len() >= 2 {
    Some(BigEndian::read_u16(input))
  } else {
    None
  }
}

pub(crate) fn price(input: &[u8], exchange: &Exchange) -> Option<f64> {
  if input.len() >= 4 && exchange.divisor() > 0_f64 {
    let value = BigEndian::read_i32(input) as f64;
    Some(value.div(exchange.divisor()))
  } else {
    None
  }
}

pub(crate) fn packet_length(bs: &[u8]) -> usize {
  if bs.len() >= 2 {
    BigEndian::read_u16(bs) as usize
  } else {
    0
  }
}

pub(crate) fn parse_tick(value: &[u8]) -> Result<Tick, ParseTickError> {
  match value.len() {
    8 | 28 | 32 | 44 | 184 => Ok(Tick::from_bytes(value)),
    len => Err(ParseTickError(format!("invalid tick size: {}", len))),
  }
}
