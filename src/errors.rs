use std::fmt;

#[derive(Debug, Clone)]
/// Errors that can occur while parsing tick data
pub struct ParseTickError(pub String);

impl fmt::Display for ParseTickError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::error::Error for ParseTickError {}
