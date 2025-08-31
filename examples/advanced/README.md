# Advanced Examples

- `manager_demo.rs`: Multi-connection manager with high-level parsed ticks.
- `manager_raw_full_peek.rs`: Multi-connection manager in raw-only mode that peeks into length-prefixed packets (184-byte Full, 32-byte Index, 64-byte InstHeader) using zero-copy helpers.

Run with live credentials:

```bash
export KITE_API_KEY=your_api_key
export KITE_ACCESS_TOKEN=your_access_token
export RUST_LOG=info

# Parsed manager demo
cargo run --example manager_demo

# Raw manager demo combining manager + raw peek
cargo run --example advanced/manager_raw_full_peek
```
