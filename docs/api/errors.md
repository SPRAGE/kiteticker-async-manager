# Error Handling Documentation

This document provides comprehensive information about error types, handling patterns, and best practices when using kiteticker-async-manager.

## Error Types

### Connection Errors

Connection errors occur during WebSocket establishment or maintenance:

```rust
use kiteticker_async_manager::{KiteTickerAsync, KiteTickerManager};

// Handle connection errors
match KiteTickerAsync::connect(&api_key, &access_token).await {
    Ok(ticker) => {
        // Connection successful
    },
    Err(e) if e.contains("Invalid credentials") => {
        eprintln!("Authentication failed: {}", e);
    },
    Err(e) if e.contains("Network") => {
        eprintln!("Network error: {}", e);
    },
    Err(e) => {
        eprintln!("Connection error: {}", e);
    }
}
```

### Subscription Errors

Errors that occur during symbol subscription or unsubscription:

```rust
// Handle subscription errors
match manager.subscribe_symbols(&symbols, Some(Mode::LTP)).await {
    Ok(()) => {
        println!("Successfully subscribed to {} symbols", symbols.len());
    },
    Err(e) if e.contains("rate limit") => {
        eprintln!("Rate limited: {}", e);
        // Implement exponential backoff
    },
    Err(e) if e.contains("invalid symbol") => {
        eprintln!("Invalid symbols provided: {}", e);
    },
    Err(e) => {
        eprintln!("Subscription error: {}", e);
    }
}
```

### Message Processing Errors

Errors during message parsing and processing:

```rust
use kiteticker_async_manager::TickerMessage;

// Handle message processing errors
while let Ok(message) = receiver.recv().await {
    match message {
        TickerMessage::Ticks(ticks) => {
            // Process valid tick data
            for tick in ticks {
                println!("Tick: {}", tick.instrument_token);
            }
        },
        TickerMessage::Error(error) => {
            eprintln!("Server error: {}", error);
            // Implement error-specific handling
            if error.contains("invalid session") {
                // Reconnect logic
            }
        },
        _ => {} // Handle other message types
    }
}
```

## Error Recovery Patterns

### Automatic Reconnection

```rust
use tokio::time::{sleep, Duration};

async fn robust_connection(api_key: &str, access_token: &str) -> Result<(), String> {
    let mut retry_count = 0;
    const MAX_RETRIES: usize = 5;
    
    loop {
        match KiteTickerAsync::connect(api_key, access_token).await {
            Ok(ticker) => {
                println!("Connected successfully");
                return process_data(ticker).await;
            },
            Err(e) => {
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    return Err(format!("Failed after {} retries: {}", MAX_RETRIES, e));
                }
                
                let delay = Duration::from_secs(2_u64.pow(retry_count as u32));
                eprintln!("Connection failed, retrying in {:?}: {}", delay, e);
                sleep(delay).await;
            }
        }
    }
}
```

### Circuit Breaker Pattern

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct CircuitBreaker {
    failure_count: Arc<AtomicUsize>,
    threshold: usize,
}

impl CircuitBreaker {
    fn new(threshold: usize) -> Self {
        Self {
            failure_count: Arc::new(AtomicUsize::new(0)),
            threshold,
        }
    }
    
    fn should_attempt(&self) -> bool {
        self.failure_count.load(Ordering::Relaxed) < self.threshold
    }
    
    fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
    }
    
    fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }
}
```

## Best Practices

### 1. Graceful Degradation

```rust
// Implement fallback mechanisms
async fn process_with_fallback(mut receiver: broadcast::Receiver<TickerMessage>) {
    let mut consecutive_errors = 0;
    const ERROR_THRESHOLD: usize = 10;
    
    while let Ok(message) = receiver.recv().await {
        match message {
            TickerMessage::Ticks(ticks) => {
                consecutive_errors = 0; // Reset on success
                process_ticks(ticks);
            },
            TickerMessage::Error(error) => {
                consecutive_errors += 1;
                
                if consecutive_errors >= ERROR_THRESHOLD {
                    eprintln!("Too many consecutive errors, switching to fallback mode");
                    // Switch to cached data or reduced functionality
                    break;
                }
            },
            _ => {}
        }
    }
}
```

### 2. Resource Cleanup

```rust
// Ensure proper cleanup
async fn managed_processing() -> Result<(), String> {
    let mut manager = KiteTickerManager::new(api_key, access_token, config);
    
    // Setup cleanup on exit
    let cleanup_manager = manager.clone();
    ctrlc::set_handler(move || {
        println!("Received interrupt signal, cleaning up...");
        tokio::spawn(async move {
            let _ = cleanup_manager.stop().await;
        });
    }).expect("Error setting Ctrl-C handler");
    
    // Start processing
    manager.start().await?;
    
    // Process data...
    
    // Cleanup on normal exit
    manager.stop().await?;
    Ok(())
}
```

### 3. Monitoring and Alerting

```rust
use log::{error, warn, info};

// Implement comprehensive monitoring
async fn monitored_processing(mut receiver: broadcast::Receiver<TickerMessage>) {
    let mut metrics = ProcessingMetrics::new();
    
    loop {
        match receiver.recv().await {
            Ok(TickerMessage::Ticks(ticks)) => {
                metrics.record_success(ticks.len());
                info!("Processed {} ticks", ticks.len());
            },
            Ok(TickerMessage::Error(error)) => {
                metrics.record_error();
                error!("Processing error: {}", error);
                
                // Send alert if error rate is high
                if metrics.error_rate() > 0.1 {
                    warn!("High error rate detected: {:.2}%", metrics.error_rate() * 100.0);
                }
            },
            Err(broadcast::error::RecvError::Closed) => {
                warn!("Channel closed, exiting");
                break;
            },
            Err(e) => {
                error!("Receive error: {}", e);
                break;
            }
        }
    }
}

struct ProcessingMetrics {
    total_messages: usize,
    error_count: usize,
}

impl ProcessingMetrics {
    fn new() -> Self {
        Self { total_messages: 0, error_count: 0 }
    }
    
    fn record_success(&mut self, count: usize) {
        self.total_messages += count;
    }
    
    fn record_error(&mut self) {
        self.error_count += 1;
        self.total_messages += 1;
    }
    
    fn error_rate(&self) -> f64 {
        if self.total_messages == 0 {
            0.0
        } else {
            self.error_count as f64 / self.total_messages as f64
        }
    }
}
```

## Common Error Scenarios

### Invalid API Credentials

**Symptoms:** Connection fails immediately with authentication error
**Solution:** Verify API key and access token are correct and not expired

### Rate Limiting

**Symptoms:** Subscription requests fail with rate limit messages
**Solution:** Implement exponential backoff and respect API limits

### Network Connectivity Issues

**Symptoms:** Intermittent connection drops or timeout errors
**Solution:** Implement retry logic with proper backoff strategies

### Symbol Limit Exceeded

**Symptoms:** New subscriptions fail when approaching 3000 symbols per connection
**Solution:** Use the multi-connection manager to distribute symbols

### Memory Pressure

**Symptoms:** Increasing memory usage, possible OOM errors
**Solution:** Monitor buffer sizes and implement proper flow control

## Error Logging Configuration

```rust
use env_logger;
use log::LevelFilter;

fn setup_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .filter_module("kiteticker_async_manager", LevelFilter::Debug)
        .init();
}
```

## Testing Error Scenarios

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_invalid_credentials() {
        let result = KiteTickerAsync::connect("invalid", "invalid").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("authentication"));
    }
    
    #[tokio::test]
    async fn test_subscription_error_handling() {
        // Test with invalid symbols
        let mut manager = create_test_manager();
        let result = manager.subscribe_symbols(&[999999999], None).await;
        // Should handle gracefully without crashing
    }
}
```

This error handling documentation provides comprehensive coverage of error scenarios and best practices for building robust applications with kiteticker-async-manager.
