# KiteTicker Async Documentation

Welcome to the comprehensive documentation for KiteTicker Async - a high-performance WebSocket client for the Kite Connect API.

## 📚 Documentation Structure

### 🚀 [Getting Started](guides/getting-started.md)
Complete beginner's guide with installation, setup, and basic usage examples.

### 🔧 [API Reference](api/)
Detailed API documentation for all modules and functions:
- **[Manager API](api/manager.md)** - Multi-connection manager
- **[Configuration](api/config.md)** - Configuration options  
- **[Models](api/models.md)** - Data structures and types
- **[Error Handling](api/errors.md)** - Error types and handling

### 📖 [Guides](guides/)
In-depth guides for specific topics:
- **[Dynamic Subscriptions](guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)** - Runtime symbol management
- **[Performance Optimization](guides/PERFORMANCE_IMPROVEMENTS.md)** - Performance tuning
- **[Implementation Details](guides/IMPLEMENTATION_COMPLETE.md)** - Technical deep dive

### 📝 [Examples](../examples/)
Practical code examples organized by complexity:
- **[Basic Examples](../examples/basic/)** - Simple usage patterns
- **[Advanced Examples](../examples/advanced/)** - Complex scenarios
- **[Performance Examples](../examples/performance/)** - Optimization and benchmarking

## 🎯 Quick Navigation

### For Beginners
1. Start with **[Getting Started](guides/getting-started.md)**
2. Try **[Basic Examples](../examples/basic/)**
3. Read **[API Reference](api/)** for details

### For Advanced Users
1. Review **[Dynamic Subscriptions](guides/DYNAMIC_SUBSCRIPTION_GUIDE.md)**
2. Explore **[Advanced Examples](../examples/advanced/)**
3. Optimize with **[Performance Guide](guides/PERFORMANCE_IMPROVEMENTS.md)**

### For Contributors
1. Check **[Implementation Details](guides/IMPLEMENTATION_COMPLETE.md)**
2. Review **[Architecture Documentation](api/)**
3. Run **[Performance Examples](../examples/performance/)**

## 🔍 Key Concepts

### Multi-Connection Architecture
- **3 WebSocket connections** running in parallel
- **9,000 symbol capacity** (3,000 per connection)
- **Automatic load balancing** across connections
- **Independent data channels** for each connection

### Dynamic Subscriptions
- **Runtime symbol addition** without reconnection
- **Runtime symbol removal** with proper cleanup
- **Mode changes** for existing subscriptions
- **Zero-downtime operations**

### Performance Optimizations
- **Dedicated parser tasks** per connection
- **High-performance buffers** for message queuing
- **Sub-microsecond latency** for data processing
- **Memory-efficient** data structures

### Zero-copy Raw Access (Advanced)

- Subscribe to raw frames via `KiteTickerAsync::subscribe_raw_frames()`
- Extract packet bodies using length prefixes (first 2 bytes = packet count, then [len, body] pairs)
- Create typed, zero-copy views with helpers:
	- `as_tick_raw` for 184-byte Full packets
	- `as_index_quote_32` for 32-byte index packets
	- `as_inst_header_64` for 64-byte instrument headers
- Safety model: all raw structs derive `Unaligned` and use big-endian wrappers; `as_*` helpers return `Option<zerocopy::Ref<&[u8], T>>` after size validation.

See examples:
- `examples/performance/raw_full_peek.rs`
- `examples/performance/raw_vs_parsed.rs`

## 📊 Feature Comparison

| Feature | Single Connection | Multi-Connection Manager |
|---------|------------------|------------------------|
| **Max Symbols** | 3,000 | 9,000 |
| **Throughput** | Limited | 3x parallel |
| **Latency** | ~5-10µs | ~1-2µs |
| **Resilience** | Single point failure | High availability |
| **Dynamic Ops** | Manual reconnect | Runtime operations |

## 🛠️ Development Resources

### API Documentation
- Complete type definitions and method signatures
- Usage examples for all functions
- Error handling patterns
- Performance characteristics

### Code Examples
- **40+ examples** covering all use cases
- Step-by-step implementation guides
- Performance benchmarking code
- Real-world usage patterns

## 🔗 External Resources

- **[Kite Connect API Docs](https://kite.trade/docs/connect/v3/websocket/)**
- **[Crates.io Package](https://crates.io/crates/kiteticker-async-manager)**
- **[GitHub Repository](https://github.com/SPRAGE/kiteticker-async-manager)**
- **[Rust Documentation](https://docs.rs/kiteticker-async-manager/)**

## 📞 Support

### Documentation Issues
If you find issues with the documentation:
1. Check the [GitHub Issues](https://github.com/SPRAGE/kiteticker-async-manager/issues)
2. Create a new issue with the "documentation" label
3. Provide specific details about the problem

### Usage Questions
For usage questions and discussions:
1. Check existing [GitHub Discussions](https://github.com/SPRAGE/kiteticker-async-manager/discussions)
2. Review the [examples directory](../examples/)
3. Consult the [getting started guide](guides/getting-started.md)

### Bug Reports
For bug reports:
1. Use the [GitHub Issues](https://github.com/SPRAGE/kiteticker-async-manager/issues)
2. Include minimal reproduction code
3. Specify your environment details

---

**📝 Documentation Version**: Latest  
**📅 Last Updated**: August 2025  
**✨ Contributors**: See [GitHub Contributors](https://github.com/SPRAGE/kiteticker-async-manager/graphs/contributors)
