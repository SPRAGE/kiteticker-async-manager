//! Zero-copy raw views for Kite tick packets.
//!
//! This module provides endian-safe, zero-allocation views over WebSocket packet bodies
//! without copying, built on `zerocopy::Ref` and big-endian integer wrappers.
//! All structs derive `Unaligned`, ensuring that references are valid even when the
//! underlying buffer is not naturally aligned.
//!
//! Highlights:
//! - `TickRaw` — 184-byte Full quote (header + 10-depth)
//! - `IndexQuoteRaw32` — 32-byte index quote snapshot
//! - `InstHeaderRaw64` — 64-byte instrument header (no depth)
//! - `as_*` helpers return `Option<zerocopy::Ref<&[u8], T>>` after validating slice size
//!
//! Example:
//! ```rust
//! # use kiteticker_async_manager::as_tick_raw;
//! # let bytes = [0u8; 184];
//! if let Some(view_ref) = as_tick_raw(&bytes) {
//!   let v = &*view_ref; // &TickRaw
//!   let token = v.header.instrument_token.get();
//!   let ltp = v.header.last_price.get();
//!   let b0_qty = v.depth.buy[0].qty.get();
//!   let b0_price = v.depth.buy[0].price.get();
//!   let _ = (token, ltp, b0_qty, b0_price);
//! }
//! ```

use zerocopy::{Unaligned, Ref, KnownLayout, Immutable, FromBytes};
use zerocopy::big_endian::{I32 as BeI32, U16 as BeU16, U32 as BeU32};

/// Size of a full quote packet body used by parser Tick (not including the 2-byte length prefix).
/// Our raw view targets the 184-byte payload region per packet for Mode::Full on equities.
pub const TICK_FULL_SIZE: usize = 184;

/// Size of index quote packet body (common snapshot when market closed)
pub const INDEX_QUOTE_SIZE: usize = 32;
/// Size of instrument header (non-index) without depth
pub const INST_HEADER_SIZE: usize = 64;

/// First 64 bytes of Full payload contain header/meta before market depth.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Unaligned, KnownLayout, Immutable, FromBytes)]
pub struct TickHeaderRaw {
  pub instrument_token: BeU32,   // 0..4
  pub last_price: BeI32,         // 4..8 (scaled by exchange divisor)
  pub last_traded_qty: BeU32,    // 8..12
  pub avg_traded_price: BeI32,   // 12..16
  pub volume_traded: BeU32,      // 16..20
  pub total_buy_qty: BeU32,      // 20..24
  pub total_sell_qty: BeU32,     // 24..28
  pub ohlc_be: [u8; 16],         // 28..44 (open high low close - order depends on index/equity)
  pub last_traded_ts: BeU32,     // 44..48 secs
  pub oi: BeU32,                 // 48..52
  pub oi_day_high: BeU32,        // 52..56
  pub oi_day_low: BeU32,         // 56..60
  pub exchange_ts: BeU32,        // 60..64 secs
}

/// A single depth entry: qty(u32), price_be(`[u8; 4]` i32), orders(u16), pad(u16)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Unaligned, KnownLayout, Immutable, FromBytes)]
pub struct DepthItemRaw {
  pub qty: BeU32,
  pub price: BeI32,
  pub orders: BeU16,
  pub _pad: BeU16, // protocol packs 12 bytes per entry; we keep struct at 12 bytes
}

/// 5 buy + 5 sell entries = 120 bytes
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Unaligned, KnownLayout, Immutable, FromBytes)]
pub struct DepthRaw {
  pub buy: [DepthItemRaw; 5],
  pub sell: [DepthItemRaw; 5],
}

/// Complete 184-byte Full packet body
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Unaligned, KnownLayout, Immutable, FromBytes)]
pub struct TickRaw {
  pub header: TickHeaderRaw, // 64 bytes
  pub depth: DepthRaw,       // 120 bytes
}

// No inherent methods needed; prefer free functions that return zerocopy::Ref

/// Try get a fixed array reference of 184 bytes from a slice (for APIs that prefer arrays)
#[inline]
pub fn as_184(slice: &[u8]) -> Option<&[u8; TICK_FULL_SIZE]> {
  <&[u8; TICK_FULL_SIZE]>::try_from(slice).ok()
}

/// Try view as `TickRaw` from a slice (zero-copy, unaligned-safe).
///
/// Returns `None` if the slice is not exactly 184 bytes.
/// The resulting `Ref` dereferences to `&TickRaw` and is valid as long as the input slice lives.
#[inline]
pub fn as_tick_raw(slice: &[u8]) -> Option<Ref<&[u8], TickRaw>> { Ref::<_, TickRaw>::from_bytes(slice).ok() }

/// 32-byte Index Quote packet (token + LTP + HLOC + price_change + exch ts)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Unaligned, KnownLayout, Immutable, FromBytes)]
pub struct IndexQuoteRaw32 {
  pub token: BeU32,        // 0..4
  pub ltp: BeI32,          // 4..8
  pub high: BeI32,         // 8..12
  pub low: BeI32,          // 12..16
  pub open: BeI32,         // 16..20
  pub close: BeI32,        // 20..24
  pub price_change: BeI32, // 24..28
  pub exch_ts: BeU32,      // 28..32
}

#[inline]
/// Try view as `IndexQuoteRaw32` from a 32-byte slice.
/// Returns `None` if the length is not 32 bytes.
pub fn as_index_quote_32(slice: &[u8]) -> Option<Ref<&[u8], IndexQuoteRaw32>> { Ref::<_, IndexQuoteRaw32>::from_bytes(slice).ok() }

/// 64-byte instrument header (equity/derivative) without depth
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Unaligned, KnownLayout, Immutable, FromBytes)]
pub struct InstHeaderRaw64 {
  pub instrument_token: BeU32, // 0..4
  pub ltp: BeI32,              // 4..8
  pub ltq: BeU32,              // 8..12 (qty)
  pub atp: BeI32,              // 12..16
  pub vol: BeU32,              // 16..20
  pub tbq: BeU32,              // 20..24
  pub tsq: BeU32,              // 24..28
  pub open: BeI32,             // 28..32
  pub high: BeI32,             // 32..36
  pub low: BeI32,              // 36..40
  pub close: BeI32,            // 40..44
  pub last_traded_ts: BeU32,   // 44..48
  pub oi: BeU32,               // 48..52
  pub oi_day_high: BeU32,      // 52..56
  pub oi_day_low: BeU32,       // 56..60
  pub exch_ts: BeU32,          // 60..64
}

#[inline]
/// Try view as `InstHeaderRaw64` from a 64-byte slice.
/// Returns `None` if the length is not 64 bytes.
pub fn as_inst_header_64(slice: &[u8]) -> Option<Ref<&[u8], InstHeaderRaw64>> { Ref::<_, InstHeaderRaw64>::from_bytes(slice).ok() }
