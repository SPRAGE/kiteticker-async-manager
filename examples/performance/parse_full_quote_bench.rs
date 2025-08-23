use kiteticker_async_manager::Tick;
use std::time::{Duration, Instant};

// Base64 encoded mock full-quote packet from kiteconnect-mocks/ticker_full.packet
const FULL_QUOTE_B64: &str = include_str!("../../kiteconnect-mocks/ticker_full.packet");

fn main() {
    env_logger::init();

    // Clean and decode
    let b64 = FULL_QUOTE_B64.trim();
    let bytes = base64::decode(b64).expect("decode mock packet");

    // Validate size matches one of supported tick sizes (184 for Full equity)
    assert!(matches!(bytes.len(), 8 | 28 | 32 | 44 | 184), "unexpected mock size: {}", bytes.len());

    // Warm up parse
    let _ = Tick::try_from(bytes.as_slice()).expect("parse warmup");

    // Benchmark rounds
    let iterations: usize = 5_000_000.min(200_000_000 / bytes.len()); // cap to keep runtime reasonable

    // Time parsing in a tight loop
    let start = Instant::now();
    let mut checksum: u64 = 0;
    for _ in 0..iterations {
        let t = Tick::try_from(bytes.as_slice()).unwrap();
        // touch a couple fields to prevent over-optimization
        checksum = checksum.wrapping_add(t.instrument_token as u64);
        if let Some(lp) = t.last_price { checksum = checksum.wrapping_add((lp.to_bits()) as u64); }
    }
    let elapsed = start.elapsed();

    let per_parse = elapsed.as_secs_f64() / (iterations as f64);
    let parses_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!("Parse benchmark (Full quote):");
    println!("  packet bytes: {}", bytes.len());
    println!("  iterations: {}", iterations);
    println!("  total: {:?}", elapsed);
    println!("  per parse: {:.3} µs ({:.0} ns)", per_parse * 1e6, per_parse * 1e9);
    println!("  throughput: {:.0} parses/sec", parses_per_sec);
    // print checksum so optimizer can't eliminate loop
    eprintln!("checksum: {}", checksum);
}
