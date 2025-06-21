use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
  Debug, Clone, Copy, Deserialize_repr, Serialize_repr, Default, PartialEq, PartialOrd,
)]
#[repr(u8)]
///
/// Modes in which packets are streamed
///
pub enum Mode {
  LTP = 1,
  #[default]
  Quote = 2,
  Full = 3,
}

impl TryFrom<usize> for Mode {
  type Error = String;
  fn try_from(value: usize) -> Result<Self, Self::Error> {
    match value {
      8 => Ok(Self::LTP),
      44 => Ok(Self::Quote),
      184 => Ok(Self::Full),
      _ => Err(format!("Invalid packet size: {}", value)),
    }
  }
}
