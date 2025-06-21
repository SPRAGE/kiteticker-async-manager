# Contributing to KiteTicker Async

Thank you for your interest in contributing to KiteTicker Async! This guide will help you get started with contributing to this high-performance WebSocket library for the Kite Connect API.

## 🎯 Ways to Contribute

### 🐛 Bug Reports
- Report bugs through [GitHub Issues](https://github.com/shaunpai/kiteticker-async-manager/issues)
- Include minimal reproduction code
- Specify environment details (OS, Rust version, etc.)

### 💡 Feature Requests  
- Suggest new features via [GitHub Discussions](https://github.com/shaunpai/kiteticker-async-manager/discussions)
- Explain the use case and expected behavior
- Consider the impact on performance and API design

### 📝 Documentation
- Improve API documentation and examples
- Fix typos and clarify explanations
- Add missing usage patterns

### 🔧 Code Contributions
- Fix bugs and implement features
- Improve performance and reliability
- Add comprehensive tests

## 🚀 Getting Started

### Prerequisites

1. **Rust toolchain** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Just task runner**
   ```bash
   cargo install just
   ```

3. **Git** for version control

### Development Setup

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/your-username/kiteticker-async-manager.git
   cd kiteticker-async-manager
   ```

2. **Set up environment**
   ```bash
   # Copy example environment file
   cp .env.example .env
   
   # Add your API credentials (for testing)
   export KITE_API_KEY=your_api_key
   export KITE_ACCESS_TOKEN=your_access_token
   ```

3. **Install dependencies and build**
   ```bash
   just build
   ```

4. **Run tests to verify setup**
   ```bash
   just test
   ```

### Development Tasks

Use `just` to run common development tasks:

```bash
just --list                    # Show all available tasks
just build                     # Build the project
just test                      # Run all tests
just check                     # Check formatting and lints
just fmt                       # Format code
just lint                      # Run clippy lints
just doc                       # Generate documentation
just bench                     # Run benchmarks
```

## 📋 Development Guidelines

### Code Style

- **Follow Rust conventions** - Use `rustfmt` and `clippy`
- **Write clear documentation** - All public APIs must be documented
- **Add comprehensive tests** - Unit tests and integration tests
- **Handle errors properly** - Use `Result` types appropriately

### Performance Considerations

This is a high-performance library, so consider:
- **Memory allocations** - Minimize unnecessary allocations
- **CPU efficiency** - Optimize hot paths
- **Network efficiency** - Minimize WebSocket overhead
- **Async best practices** - Use appropriate async patterns

### Testing Requirements

All contributions must include:

1. **Unit tests** for new functionality
2. **Integration tests** for complex features  
3. **Documentation tests** for examples
4. **Benchmark tests** for performance-critical code

### API Design Principles

- **Async-first** - All APIs should be async-compatible
- **Type safety** - Use strong typing for reliability
- **Ergonomic** - APIs should be easy to use correctly
- **Backward compatible** - Avoid breaking changes when possible

## 🔄 Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

### 2. Make Changes

- Write your code following the guidelines above
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Test Your Changes

```bash
# Run the full test suite
just test

# Run specific tests
cargo test test_name

# Test with real API (if applicable)
cargo run --example basic/single_connection

# Run benchmarks (for performance changes)
just bench
```

### 4. Check Code Quality

```bash
# Format code
just fmt

# Run lints
just lint

# Check documentation
just doc
```

### 5. Commit Changes

```bash
git add .
git commit -m "feat: add dynamic subscription batching

- Implement batch operations for better performance
- Add tests for batch subscription/unsubscription
- Update documentation with batch examples"
```

**Commit Message Format:**
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `perf:` - Performance improvements
- `test:` - Test additions/changes
- `refactor:` - Code refactoring

### 6. Submit Pull Request

1. Push your branch to your fork
2. Create a pull request on GitHub
3. Provide a clear description of changes
4. Link to relevant issues
5. Wait for review and address feedback

## 🧪 Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_symbol_subscription() {
        // Test implementation
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory:

```rust
// tests/integration_test.rs
use kiteticker_async::{KiteTickerManager, KiteManagerConfig};

#[tokio::test]
async fn test_manager_full_workflow() {
    // Full workflow test
}
```

### Benchmark Tests

Add benchmarks for performance-critical code:

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parser(c: &mut Criterion) {
    c.bench_function("parse_message", |b| {
        b.iter(|| {
            // Benchmark implementation
        })
    });
}

criterion_group!(benches, benchmark_parser);
criterion_main!(benches);
```

## 📖 Documentation Guidelines

### API Documentation

All public APIs must have comprehensive documentation:

```rust
/// Subscribes to symbols with automatic load balancing across connections
/// 
/// This method distributes symbols across available connections using round-robin
/// allocation. Symbols are automatically deduplicated, and existing subscriptions
/// are skipped.
/// 
/// # Arguments
/// 
/// * `symbols` - Array of instrument tokens to subscribe to
/// * `mode` - Subscription mode (LTP, Quote, Full) - uses default if None
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or `Err(String)` with error details
/// 
/// # Example
/// 
/// ```rust
/// use kiteticker_async::{KiteTickerManager, Mode};
/// 
/// # async fn example() -> Result<(), String> {
/// let mut manager = KiteTickerManager::new(api_key, token, config);
/// manager.start().await?;
/// 
/// // Subscribe to NIFTY 50 and HDFC Bank
/// let symbols = vec![256265, 408065];
/// manager.subscribe_symbols(&symbols, Some(Mode::Quote)).await?;
/// # Ok(())
/// # }
/// ```
/// 
/// # Performance
/// 
/// This operation has O(n) complexity where n is the number of symbols.
/// Symbols are distributed in batches for optimal network efficiency.
pub async fn subscribe_symbols(&mut self, symbols: &[u32], mode: Option<Mode>) -> Result<(), String> {
    // Implementation
}
```

### Example Documentation

Add comprehensive examples to the `examples/` directory:

```rust
//! # Dynamic Subscription Example
//! 
//! This example demonstrates how to add and remove symbols at runtime
//! without reconnecting the WebSocket connections.
//! 
//! ## Features Demonstrated
//! 
//! - Runtime symbol addition
//! - Runtime symbol removal  
//! - Mode changes for existing symbols
//! - Real-time statistics monitoring

use kiteticker_async::{KiteTickerManager, KiteManagerConfig, Mode};

#[tokio::main]
async fn main() -> Result<(), String> {
    // Example implementation
}
```

## 🚨 Common Issues and Solutions

### Build Issues

**Problem**: Compilation errors with async code
**Solution**: Ensure you're using the latest stable Rust and check async/await syntax

**Problem**: Missing dependencies
**Solution**: Run `cargo update` and check `Cargo.toml`

### Test Issues

**Problem**: Tests fail with authentication errors
**Solution**: Verify API credentials are set correctly in environment

**Problem**: Flaky integration tests
**Solution**: Add proper timeouts and retry logic

### Performance Issues

**Problem**: High latency in message processing
**Solution**: Check buffer sizes and parser configuration

**Problem**: Memory usage grows over time
**Solution**: Verify proper cleanup and resource management

## 📊 Performance Benchmarking

When making performance-related changes:

1. **Baseline measurement**
   ```bash
   just bench > baseline.txt
   ```

2. **Make your changes**

3. **Compare results**
   ```bash
   just bench > improved.txt
   ```

4. **Include results in PR description**

## 🔍 Code Review Process

### What We Look For

- **Correctness** - Does the code work as intended?
- **Performance** - Does it maintain or improve performance?
- **Safety** - Is it memory-safe and thread-safe?
- **Documentation** - Is it well-documented?
- **Tests** - Are there adequate tests?

### Review Timeline

- **Initial review** - Within 2-3 days
- **Follow-up reviews** - Within 1-2 days
- **Merge** - After approval from maintainers

## 🏆 Recognition

Contributors are recognized in:
- GitHub contributors list
- Release notes for significant contributions  
- Documentation credits
- Crates.io package metadata

## 📞 Getting Help

### Questions During Development

- **GitHub Discussions** - For design questions
- **GitHub Issues** - For specific problems
- **Code comments** - Tag maintainers for review

### Maintainer Contact

- **GitHub**: @kaychaks
- **Email**: See GitHub profile

## 📄 License

By contributing to KiteTicker Async, you agree that your contributions will be licensed under the Apache 2.0 License.

---

**Thank you for contributing to KiteTicker Async! 🚀**
