use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DefaultOnNull};

use crate::Exchange;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderTransactionType {
  Buy,
  Sell,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
pub enum OrderValidity {
  DAY,
  IOC,
  TTL,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
  COMPLETE,
  REJECTED,
  CANCELLED,
  UPDATE,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PartialOrd)]
#[serde(transparent)]
pub struct TimeStamp(i64);

impl From<String> for TimeStamp {
  fn from(value: String) -> Self {
    let secs = NaiveDateTime::parse_from_str(&value, "%Y-%m-%d %H:%M:%S")
      .unwrap()
      .and_utc()
      .timestamp();
    TimeStamp(secs)
  }
}

impl From<TimeStamp> for String {
  fn from(value: TimeStamp) -> Self {
    DateTime::<Utc>::from_timestamp(value.0, 0)
      .unwrap_or_default()
      .naive_utc()
      .format("%Y-%m-%d %H:%M:%S")
      .to_string()
  }
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Order {
  pub order_id: String,

  #[serde_as(as = "DefaultOnNull")]
  pub exchange_order_id: Option<String>,

  #[serde_as(as = "DefaultOnNull")]
  pub parent_order_id: Option<String>,

  pub placed_by: String,
  pub app_id: u64,

  pub status: OrderStatus,

  #[serde_as(as = "DefaultOnNull")]
  pub status_message: Option<String>,

  #[serde_as(as = "DefaultOnNull")]
  pub status_message_raw: Option<String>,

  pub tradingsymbol: String,
  pub instrument_token: u32,

  #[serde_as(as = "serde_with::FromInto<String>")]
  pub exchange: Exchange,

  pub order_type: String,
  pub transaction_type: OrderTransactionType,

  pub validity: OrderValidity,
  pub variety: String,
  pub product: Option<String>,

  #[serde(default)]
  pub average_price: f64,

  #[serde(default)]
  pub disclosed_quantity: f64,

  pub price: f64,
  pub quantity: u64,
  pub filled_quantity: u64,

  #[serde(default)]
  pub unfilled_quantity: u64,

  #[serde(default)]
  pub pending_quantity: u64,

  #[serde(default)]
  pub cancelled_quantity: u64,

  #[serde(default)]
  pub trigger_price: f64,

  pub user_id: String,

  #[serde_as(as = "serde_with::FromInto<String>")]
  pub order_timestamp: TimeStamp,
  #[serde_as(as = "serde_with::FromInto<String>")]
  pub exchange_timestamp: TimeStamp,
  #[serde_as(as = "serde_with::FromInto<String>")]
  pub exchange_update_timestamp: TimeStamp,

  pub checksum: String,
  #[serde(default)]
  pub meta: Option<serde_json::Map<String, Value>>,

  #[serde_as(as = "DefaultOnNull")]
  #[serde(default)]
  pub tag: Option<String>,
}
