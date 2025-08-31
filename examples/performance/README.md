# Performance Examples

Performance optimization and benchmarking scenarios using raw frames and zero-copy parsing.

## Examples

- `performance_demo.rs` — Performance testing and metrics
- `high_frequency.rs` — High-frequency data processing with heavy load
- `raw_full_peek.rs` — Zero-copy peek into 184/64/32-byte packet bodies from raw frames
- `raw_vs_parsed.rs` — Compare overhead of parsed vs raw-only modes
- `market_scanner.rs` — Scan a large set of symbols efficiently
- `parse_full_quote_bench.rs` — Micro-bench for full quote parsing

## How to run

```bash
# From project root
export KITE_API_KEY=your_api_key
export KITE_ACCESS_TOKEN=your_access_token
export RUST_LOG=info

cargo run --example performance/performance_demo
cargo run --example performance/high_frequency
cargo run --example performance/raw_full_peek
cargo run --example performance/raw_vs_parsed
cargo run --example performance/market_scanner
cargo run --example performance/parse_full_quote_bench
```

## Tips

- For highest throughput, consider `raw_only(true)` and use the zero-copy helpers
- Increase buffer sizes in configs when measuring with large symbol sets
- Pin CPU cores and disable debug logs for more stable benchmarks
