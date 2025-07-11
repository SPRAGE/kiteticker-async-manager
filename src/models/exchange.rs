use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
  Debug, Clone, Copy, Default, PartialEq, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
///
/// Exchange options
///
pub enum Exchange {
  #[default]
  NSE = 1,
  NFO = 2,
  CDS = 3,
  BSE = 4,
  BFO = 5,
  BCD = 6,
  MCX = 7,
  MCXSX = 8,
  INDICES = 9,
}

impl Exchange {
  pub(crate) fn divisor(&self) -> f64 {
    match self {
      Self::CDS => 1_000_000.0,
      Self::BCD => 1_000.0,
      _ => 100.0,
    }
  }

  pub(crate) fn is_tradable(&self) -> bool {
    !matches!(self, Self::INDICES)
  }
}

impl From<usize> for Exchange {
  fn from(value: usize) -> Self {
    match value {
      9 => Self::INDICES,
      8 => Self::MCXSX,
      7 => Self::MCX,
      6 => Self::BCD,
      5 => Self::BFO,
      4 => Self::BSE,
      3 => Self::CDS,
      2 => Self::NFO,
      1 => Self::NSE,
      _ => Self::NSE,
    }
  }
}

impl From<String> for Exchange {
  fn from(value: String) -> Self {
    match value.as_str() {
      "NSE" => Self::NSE,
      "NFO" => Self::NFO,
      "CDS" => Self::CDS,
      "BSE" => Self::BSE,
      "BFO" => Self::BFO,
      "BCD" => Self::BCD,
      "MCX" => Self::MCX,
      "MCXSX" => Self::MCXSX,
      "INDICES" => Self::INDICES,
      _ => Self::NSE,
    }
  }
}

impl From<Exchange> for String {
  fn from(value: Exchange) -> Self {
    match value {
      Exchange::NSE => "NSE".to_string(),
      Exchange::NFO => "NFO".to_string(),
      Exchange::CDS => "CDS".to_string(),
      Exchange::BSE => "BSE".to_string(),
      Exchange::BFO => "BFO".to_string(),
      Exchange::BCD => "BCD".to_string(),
      Exchange::MCX => "MCX".to_string(),
      Exchange::MCXSX => "MCXSX".to_string(),
      Exchange::INDICES => "INDICES".to_string(),
    }
  }
}
