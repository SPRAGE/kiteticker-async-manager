use base64::{engine::general_purpose::STANDARD, Engine as _};
use kiteticker_async_manager::{Tick, TickRaw, as_tick_raw};
use std::hint::black_box;
use std::time::Instant;

// Base64 encoded mock full-quote packet from kiteconnect-mocks/ticker_full.packet
const FULL_QUOTE_B64: &str = include_str!("../../kiteconnect-mocks/ticker_full.packet");

fn main() {
  env_logger::init();

  // Clean and decode
  let b64 = FULL_QUOTE_B64.trim();
  let bytes = STANDARD.decode(b64).expect("decode mock packet");

  assert_eq!(bytes.len(), 184, "benchmark expects a 184-byte full payload");

  // Warm up
  let _ = Tick::try_from(bytes.as_slice()).expect("parse warmup");
  let _ = as_tick_raw(bytes.as_slice()).expect("raw warmup");

  // Choose iteration count roughly normalized by packet size to keep runs fast
  let iterations: usize = 10_000_000.min(200_000_000 / bytes.len());

  // Benchmark parsed Tick
  let start = Instant::now();
  let mut checksum_p: u64 = 0;
  for _ in 0..iterations {
    let t = Tick::try_from(black_box(bytes.as_slice())).unwrap();
    checksum_p = checksum_p.wrapping_add(black_box(t.instrument_token as u64));
    if let Some(lp) = t.last_price { checksum_p = checksum_p.wrapping_add(black_box(lp.to_bits())); }
  }
  let elapsed_p = start.elapsed();

  // Benchmark raw TickRaw field peeks (no allocations)
  let start = Instant::now();
  let mut checksum_r: u64 = 0;
  for _ in 0..iterations {
    let v_ref = as_tick_raw(black_box(bytes.as_slice())).unwrap();
    let v: &TickRaw = &*v_ref;
    checksum_r = checksum_r.wrapping_add(black_box(v.header.instrument_token.get() as u64));
    checksum_r = checksum_r.wrapping_add(black_box(v.header.last_price.get() as u64));
  }
  let elapsed_r = start.elapsed();

  let per_parsed = elapsed_p.as_secs_f64() / (iterations as f64);
  let per_raw = elapsed_r.as_secs_f64() / (iterations as f64);

  println!("Benchmark on 184-byte full payload:");
  println!("  bytes: {}", bytes.len());
  println!("  iterations: {}", iterations);
  println!("  parsed:   total {:?}, per op: {:.3} µs ({:.0} ns), throughput: {:.0}/s",
    elapsed_p, per_parsed * 1e6, per_parsed * 1e9, (iterations as f64)/elapsed_p.as_secs_f64());
  println!("  raw view: total {:?}, per op: {:.3} µs ({:.0} ns), throughput: {:.0}/s",
    elapsed_r, per_raw * 1e6, per_raw * 1e9, (iterations as f64)/elapsed_r.as_secs_f64());
  eprintln!("checksums parsed={}, raw={}", checksum_p, checksum_r);
}
