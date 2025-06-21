use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use kiteticker_async_manager::{KiteTickerAsync, TickerMessage, Mode, Tick, TickMessage};
use std::time::Duration;
use tokio::time::timeout;

async fn benchmark_message_processing(binary_data: &[u8]) -> Option<TickerMessage> {
    // Simulate basic message processing without private functions
    if binary_data.len() < 4 {
        return None;
    }
    
    // Create a mock tick message for benchmarking
    let tick = Tick {
        mode: Mode::LTP,
        instrument_token: 256265,
        ..Default::default()
    };
    
    Some(TickerMessage::Ticks(vec![TickMessage::new(256265, tick)]))
}

fn create_mock_binary_data(num_packets: u16, packet_size: usize) -> Vec<u8> {
    let mut data = Vec::new();
    
    // Add packet count header
    data.extend_from_slice(&num_packets.to_be_bytes());
    
    // Add mock packets
    for i in 0..num_packets {
        // Packet length
        data.extend_from_slice(&(packet_size as u16).to_be_bytes());
        
        // Mock packet data - simplified tick data
        let mut packet_data = vec![0u8; packet_size];
        
        // Instrument token (first 4 bytes)
        let token = 400000 + i as u32;
        packet_data[0..4].copy_from_slice(&token.to_be_bytes());
        
        // Last price (bytes 4-8)
        let price = 1500.0 + (i as f64 * 0.5);
        packet_data[4..8].copy_from_slice(&(price as u32).to_be_bytes());
        
        data.extend_from_slice(&packet_data);
    }
    
    data
}

fn benchmark_binary_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("binary_processing");
    
    // Test with different numbers of packets
    for num_packets in [1, 10, 50, 100, 500].iter() {
        let data = create_mock_binary_data(*num_packets, 44); // Full mode packet size
        
        group.bench_with_input(
            BenchmarkId::new("process_packets", num_packets),
            &data,
            |b, data| {
                b.to_async(&rt).iter(|| async {
                    benchmark_message_processing(data).await
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_connection_health_check(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("connection_health_check", |b| {
        b.to_async(&rt).iter(|| async {
            // Create a mock ticker without actual connection
            let ticker = create_mock_ticker().await;
            ticker.is_connected()
        });
    });
}

async fn create_mock_ticker() -> KiteTickerAsync {
    // This would require mocking the WebSocket connection
    // For now, we'll just measure the health check on a disconnected ticker
    use tokio::sync::{broadcast, mpsc};
    use tokio::task::JoinHandle;
    
    KiteTickerAsync {
        api_key: "test".to_string(),
        access_token: "test".to_string(),
        cmd_tx: None,
        msg_tx: broadcast::channel(1000).0,
        writer_handle: None,
        reader_handle: None,
    }
}

fn benchmark_broadcast_channel_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("broadcast_throughput");
    
    // Test with different buffer sizes
    for buffer_size in [100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("buffer_size", buffer_size),
            buffer_size,
            |b, &buffer_size| {
                b.to_async(&rt).iter(|| async {
                    use tokio::sync::broadcast;
                    use kiteticker_async_manager::{TickerMessage, TickMessage, Tick, Mode};
                    
                    let (tx, mut rx) = broadcast::channel(buffer_size);
                    
                    // Send 100 messages
                    for i in 0..100 {
                        let tick = Tick {
                            mode: Mode::LTP,
                            instrument_token: i,
                            ..Default::default()
                        };
                        let msg = TickerMessage::Ticks(vec![TickMessage::new(i, tick)]);
                        let _ = tx.send(msg);
                    }
                    
                    // Try to receive all messages
                    let mut count = 0;
                    while let Ok(_) = timeout(Duration::from_millis(1), rx.recv()).await {
                        count += 1;
                        if count >= 100 {
                            break;
                        }
                    }
                    count
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_binary_processing,
    benchmark_connection_health_check,
    benchmark_broadcast_channel_throughput
);
criterion_main!(benches);
