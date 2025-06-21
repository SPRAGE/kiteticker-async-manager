# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-06-21

### Added
- **Multi-Connection Support**: Utilize all 3 allowed WebSocket connections (9,000 symbol capacity)
- **High Performance**: Dedicated parser tasks, optimized buffers, sub-microsecond latency  
- **Dynamic Subscriptions**: Add/remove symbols at runtime without reconnection
- **Load Balancing**: Automatic symbol distribution across connections
- **Production Ready**: Comprehensive error handling, health monitoring, reconnection
- **Async-First Design**: Built with Tokio, follows Rust async best practices
- **Type Safety**: Fully typed market data structures with serde serialization
- **Comprehensive Documentation**: API docs, guides, and examples
- **Performance Examples**: Market scanner, load testing, high-frequency trading examples
- **Health Monitoring**: Real-time connection health tracking
- **Error Resilience**: Comprehensive error handling and recovery mechanisms

### Features
- `KiteTickerManager` for managing multiple WebSocket connections
- `KiteManagerConfig` for flexible configuration
- Dynamic subscription management with runtime operations
- Independent output channels for each connection
- Optimized binary message parsing with dedicated tasks
- Comprehensive error types and handling
- Connection health monitoring and automatic recovery
- Support for all Kite Connect modes (LTP, Quote, Full)

### Examples
- Basic examples: Single connection, portfolio monitoring, runtime subscriptions
- Advanced examples: Dynamic subscription workflows, multi-connection setups
- Performance examples: Market scanning, load testing, high-frequency trading

### Documentation
- Complete API reference documentation
- Getting started guide with practical examples
- Dynamic subscription guide
- Performance optimization guide
- Error handling documentation
- Contributing guidelines

### Development
- Comprehensive test suite with unit and integration tests
- Benchmark suite for performance testing
- Development tasks with `just` task runner
- CI-ready project structure
- Nix flake for reproducible development environment

---

### Contributors
- Kaushik Chakraborty <git@kaushikc.org> - Original implementation
- Shaun Pai <shauna.pai@gmail.com> - Enhanced features and documentation

### License
Licensed under the Apache License, Version 2.0
