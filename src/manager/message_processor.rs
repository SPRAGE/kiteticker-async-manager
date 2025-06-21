use crate::models::TickerMessage;
use crate::manager::ChannelId;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, broadcast, RwLock};
use tokio::task::JoinHandle;

/// High-performance message processor with dedicated parsing task
#[derive(Debug)]
pub struct MessageProcessor {
    pub channel_id: ChannelId,
    pub input_receiver: Option<mpsc::UnboundedReceiver<TickerMessage>>,
    pub output_sender: broadcast::Sender<TickerMessage>,
    pub stats: Arc<RwLock<ProcessorStats>>,
    pub task_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    pub messages_processed: u64,
    pub messages_per_second: f64,
    pub processing_latency_avg: std::time::Duration,
    pub last_processed_time: Option<Instant>,
    pub queue_size: usize,
    pub errors_count: u64,
}

impl MessageProcessor {
    /// Create a new message processor
    pub fn new(
        channel_id: ChannelId,
        input_receiver: mpsc::UnboundedReceiver<TickerMessage>,
        buffer_size: usize,
    ) -> (Self, broadcast::Receiver<TickerMessage>) {
        let (output_sender, output_receiver) = broadcast::channel(buffer_size);
        let stats = Arc::new(RwLock::new(ProcessorStats::default()));
        
        let processor = Self {
            channel_id,
            input_receiver: Some(input_receiver),
            output_sender,
            stats,
            task_handle: None,
        };
        
        (processor, output_receiver)
    }
    
    /// Start the dedicated processing task
    pub fn start(&mut self) {
        // Only start if not already running
        if self.task_handle.is_some() {
            log::warn!("Message processor for channel {:?} already started", self.channel_id);
            return;
        }
        
        // Take the receiver to move it to the task (this fixes the bug!)
        let input_receiver = self.input_receiver.take()
            .expect("Receiver already taken - processor can only be started once");
        
        let channel_id = self.channel_id;
        let output_sender = self.output_sender.clone();
        let stats = Arc::clone(&self.stats);
        
        let handle = tokio::spawn(async move {
            Self::processing_loop(
                channel_id,
                input_receiver,
                output_sender,
                stats,
            ).await;
        });
        
        self.task_handle = Some(handle);
        log::info!("Started message processor task for channel {:?}", channel_id);
    }
    
    /// High-performance message processing loop
    async fn processing_loop(
        channel_id: ChannelId,
        mut input_receiver: mpsc::UnboundedReceiver<TickerMessage>,
        output_sender: broadcast::Sender<TickerMessage>,
        stats: Arc<RwLock<ProcessorStats>>,
    ) {
        let mut last_stats_update = Instant::now();
        let mut messages_since_last_update = 0u64;
        
        log::info!("Started message processor for channel {:?}", channel_id);
        
        while let Some(message) = input_receiver.recv().await {
            let processing_start = Instant::now();
            
            // Process the message (currently just forwarding, but can add logic here)
            let processed_message = Self::process_message(message, channel_id);
            
            // Send to output channel (non-blocking)
            match output_sender.send(processed_message) {
                Ok(receiver_count) => {
                    // Successfully sent to all receivers
                    if receiver_count == 0 {
                        log::debug!("Channel {:?}: No active receivers", channel_id);
                    }
                }
                Err(_) => {
                    // This shouldn't happen with broadcast channels unless buffer is full
                    log::warn!("Channel {:?}: Failed to send message", channel_id);
                    
                    let mut stats = stats.write().await;
                    stats.errors_count += 1;
                    continue;
                }
            }
            
            let processing_time = processing_start.elapsed();
            messages_since_last_update += 1;
            
            // Update stats periodically to avoid lock contention
            if last_stats_update.elapsed() >= std::time::Duration::from_secs(1) {
                let mut stats_guard = stats.write().await;
                stats_guard.messages_processed += messages_since_last_update;
                stats_guard.last_processed_time = Some(Instant::now());
                stats_guard.queue_size = input_receiver.len();
                
                // Calculate messages per second
                let elapsed = last_stats_update.elapsed();
                stats_guard.messages_per_second = 
                    messages_since_last_update as f64 / elapsed.as_secs_f64();
                
                // Update average processing latency (simple moving average)
                let current_avg = stats_guard.processing_latency_avg;
                stats_guard.processing_latency_avg = if current_avg.is_zero() {
                    processing_time
                } else {
                    // Weighted average: 90% old + 10% new
                    Duration::from_nanos(
                        (current_avg.as_nanos() as f64 * 0.9 + 
                         processing_time.as_nanos() as f64 * 0.1) as u64
                    )
                };
                
                drop(stats_guard);
                
                // Reset counters
                last_stats_update = Instant::now();
                messages_since_last_update = 0;
            }
        }
        
        log::info!("Message processor for channel {:?} stopped", channel_id);
    }
    
    /// Process individual message (can be extended for custom logic)
    fn process_message(message: TickerMessage, channel_id: ChannelId) -> TickerMessage {
        // Currently just passes through, but you can add:
        // - Message validation
        // - Data enrichment
        // - Format conversion
        // - Filtering logic
        // - Latency tagging
        
        match &message {
            TickerMessage::Ticks(ticks) => {
                log::debug!("Channel {:?}: Processed {} ticks", channel_id, ticks.len());
            }
            TickerMessage::Error(error) => {
                log::warn!("Channel {:?}: Error message: {}", channel_id, error);
            }
            _ => {
                log::debug!("Channel {:?}: Processed message: {:?}", channel_id, message);
            }
        }
        
        message
    }
    
    /// Get current processor statistics
    pub async fn get_stats(&self) -> ProcessorStats {
        self.stats.read().await.clone()
    }
    
    /// Get current queue size (non-blocking)
    pub fn queue_size(&self) -> usize {
        // Note: This is approximate since we can't access the receiver's len() from here
        // In a real implementation, you might want to track this differently
        0
    }
    
    /// Stop the processor
    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

use std::time::Duration;
