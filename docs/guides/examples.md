# Examples Guide

This guide maps the examples in the repository to common usage patterns and shows how to run them.

## Prerequisites

Set your credentials and optionally logging:

```bash
export KITE_API_KEY=your_api_key
export KITE_ACCESS_TOKEN=your_access_token
export RUST_LOG=info
```

## Basic Examples

- `basic/single_connection.rs` — Minimal single WebSocket connection
- `basic/portfolio_monitor.rs` — Monitor a small portfolio and print updates
- `basic/runtime_subscription_example.rs` — Add, remove, and change modes at runtime

Run:

```bash
cargo run --example basic/single_connection
cargo run --example basic/portfolio_monitor
cargo run --example basic/runtime_subscription_example
```

## Advanced Examples

- `advanced/manager_demo.rs` — Multi-connection manager with parsed ticks and stats
- `dynamic_subscription_demo.rs` — Full dynamic subscription workflow
- `advanced/manager_raw_full_peek.rs` — Manager in raw-only mode with zero-copy packet peeking

Run:

```bash
cargo run --example advanced/manager_demo
cargo run --example dynamic_subscription_demo
cargo run --example manager_raw_full_peek
```

## Performance Examples

- `performance/performance_demo.rs` — Performance testing harness
- `performance/high_frequency.rs` — High-frequency processing setup
- `performance/raw_full_peek.rs` — Peek 184/64/32 raw packet bodies using zero-copy
- `performance/raw_vs_parsed.rs` — Compare parsed vs raw-only overhead
- `performance/market_scanner.rs` — Scan many symbols efficiently
- `performance/parse_full_quote_bench.rs` — Parse micro-benchmark

Run:

```bash
cargo run --example performance/performance_demo
cargo run --example performance/high_frequency
cargo run --example performance/raw_full_peek
cargo run --example performance/raw_vs_parsed
cargo run --example performance/market_scanner
cargo run --example performance/parse_full_quote_bench
```

## Tips

- For lowest latency and highest throughput, consider `raw_only(true)` and the raw helpers
- Increase buffer sizes when working with large symbol sets
- Use `RUST_LOG=info` (or `debug`) to observe connection and parsing behavior
