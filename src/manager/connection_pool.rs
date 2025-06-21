use crate::models::{Mode, TickerMessage};
use crate::ticker::KiteTickerAsync;
use crate::manager::{KiteManagerConfig, ConnectionStats, ChannelId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;

/// Represents a single WebSocket connection with its metadata
#[derive(Debug)]
pub struct ManagedConnection {
    pub id: ChannelId,
    pub ticker: Option<KiteTickerAsync>,
    pub subscriber: Option<crate::ticker::KiteTickerSubscriber>,
    pub subscribed_symbols: HashMap<u32, Mode>,
    pub stats: Arc<RwLock<ConnectionStats>>,
    pub is_healthy: Arc<AtomicBool>,
    pub last_ping: Arc<AtomicU64>, // Unix timestamp
    pub task_handle: Option<JoinHandle<()>>,
    pub message_sender: mpsc::UnboundedSender<TickerMessage>,
    // Store credentials for dynamic operations
    api_key: String,
    access_token: String,
}

impl ManagedConnection {
    pub fn new(id: ChannelId, message_sender: mpsc::UnboundedSender<TickerMessage>) -> Self {
        let mut stats = ConnectionStats::default();
        stats.connection_id = id.to_index();
        
        Self {
            id,
            ticker: None,
            subscriber: None,
            subscribed_symbols: HashMap::new(),
            stats: Arc::new(RwLock::new(stats)),
            is_healthy: Arc::new(AtomicBool::new(false)),
            last_ping: Arc::new(AtomicU64::new(0)),
            task_handle: None,
            message_sender,
            api_key: String::new(),
            access_token: String::new(),
        }
    }
    
    /// Connect to WebSocket and start message processing
    pub async fn connect(
        &mut self,
        api_key: &str,
        access_token: &str,
        config: &KiteManagerConfig,
    ) -> Result<(), String> {
        // Store credentials for dynamic operations
        self.api_key = api_key.to_string();
        self.access_token = access_token.to_string();
        
        // Connect to WebSocket
        let ticker = timeout(
            config.connection_timeout,
            KiteTickerAsync::connect(api_key, access_token)
        )
        .await
        .map_err(|_| "Connection timeout".to_string())?
        .map_err(|e| format!("Connection failed: {}", e))?;
        
        self.ticker = Some(ticker);
        self.is_healthy.store(true, Ordering::Relaxed);
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.is_connected = true;
            stats.connection_uptime = Duration::ZERO;
        }
        
        Ok(())
    }
    
    /// Subscribe to symbols on this connection
    pub async fn subscribe_symbols(
        &mut self,
        symbols: &[u32],
        mode: Mode,
    ) -> Result<(), String> {
        if self.ticker.is_some() && !self.api_key.is_empty() {
            // Create a new ticker connection for the subscriber
            // This avoids consuming the main ticker
            let subscriber_ticker = crate::ticker::KiteTickerAsync::connect(
                &self.api_key, 
                &self.access_token
            ).await?;
            
            // Use the new ticker to create a subscriber
            let subscriber = subscriber_ticker.subscribe(symbols, Some(mode.clone())).await?;
            
            // Update our symbol tracking
            for &symbol in symbols {
                self.subscribed_symbols.insert(symbol, mode.clone());
            }
            
            // Store subscriber for message processing
            self.subscriber = Some(subscriber);
            
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.symbol_count = self.subscribed_symbols.len();
            }
            
            Ok(())
        } else {
            Err("Connection not established".to_string())
        }
    }

    /// Dynamically add new symbols to existing subscription
    pub async fn add_symbols(
        &mut self,
        symbols: &[u32],
        mode: Mode,
    ) -> Result<(), String> {
        if self.subscriber.is_some() {
            // Add to existing subscription
            let subscriber = self.subscriber.as_mut().unwrap();
            subscriber.subscribe(symbols, Some(mode.clone())).await?;
            
            // Update our symbol tracking
            for &symbol in symbols {
                self.subscribed_symbols.insert(symbol, mode.clone());
            }
            
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.symbol_count = self.subscribed_symbols.len();
            }
            
            log::info!("Added {} symbols to connection {}", symbols.len(), self.id.to_index());
            Ok(())
        } else {
            // First subscription - use the original method
            self.subscribe_symbols(symbols, mode).await
        }
    }

    /// Dynamically remove symbols from existing subscription
    pub async fn remove_symbols(&mut self, symbols: &[u32]) -> Result<(), String> {
        if let Some(subscriber) = &mut self.subscriber {
            // Remove from existing subscription
            subscriber.unsubscribe(symbols).await?;
            
            // Update our symbol tracking
            for &symbol in symbols {
                self.subscribed_symbols.remove(&symbol);
            }
            
            // Update stats
            {
                let mut stats = self.stats.write().await;
                stats.symbol_count = self.subscribed_symbols.len();
            }
            
            log::info!("Removed {} symbols from connection {}", symbols.len(), self.id.to_index());
            Ok(())
        } else {
            Err("No active subscription to remove symbols from".to_string())
        }
    }

    /// Start message processing for the subscriber
    pub async fn start_message_processing(&mut self) -> Result<(), String> {
        if let Some(subscriber) = self.subscriber.take() {
            let message_sender = self.message_sender.clone();
            let stats = Arc::clone(&self.stats);
            let is_healthy = Arc::clone(&self.is_healthy);
            let connection_id = self.id;
            
            let handle = tokio::spawn(async move {
                Self::message_processing_loop(
                    subscriber,
                    message_sender,
                    stats,
                    is_healthy,
                    connection_id,
                ).await;
            });
            
            self.task_handle = Some(handle);
            Ok(())
        } else {
            Err("No subscriber available for message processing".to_string())
        }
    }
    
    /// Message processing loop for this connection
    async fn message_processing_loop(
        mut subscriber: crate::ticker::KiteTickerSubscriber,
        message_sender: mpsc::UnboundedSender<TickerMessage>,
        stats: Arc<RwLock<ConnectionStats>>,
        is_healthy: Arc<AtomicBool>,
        connection_id: ChannelId,
    ) {
        let mut last_message_time = Instant::now();
        
        log::info!("Starting message processing loop for connection {}", connection_id.to_index());
        
        loop {
            match timeout(Duration::from_secs(30), subscriber.next_message()).await {
                Ok(Ok(Some(message))) => {
                    last_message_time = Instant::now();
                    
                    // Update stats
                    {
                        let mut stats = stats.write().await;
                        stats.messages_received += 1;
                        stats.last_message_time = Some(last_message_time);
                    }
                    
                    // Forward message to parser (non-blocking)
                    if let Err(_) = message_sender.send(message) {
                        log::warn!("Connection {}: Parser channel full, dropping message", connection_id.to_index());
                        
                        // Update error stats
                        let mut stats = stats.write().await;
                        stats.errors_count += 1;
                    }
                }
                Ok(Ok(None)) => {
                    log::info!("Connection {} closed", connection_id.to_index());
                    is_healthy.store(false, Ordering::Relaxed);
                    break;
                }
                Ok(Err(e)) => {
                    log::error!("Connection {} error: {}", connection_id.to_index(), e);
                    
                    // Update error stats
                    let mut stats = stats.write().await;
                    stats.errors_count += 1;
                    
                    // Continue trying to receive messages
                }
                Err(_) => {
                    // Timeout - check if connection is still alive
                    if last_message_time.elapsed() > Duration::from_secs(60) {
                        log::warn!("Connection {} timeout - no messages for 60s", connection_id.to_index());
                        is_healthy.store(false, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }
        
        // Update connection status
        {
            let mut stats = stats.write().await;
            stats.is_connected = false;
        }
        is_healthy.store(false, Ordering::Relaxed);
    }
    
    /// Check if connection can accept more symbols
    pub fn can_accept_symbols(&self, count: usize, max_per_connection: usize) -> bool {
        self.subscribed_symbols.len() + count <= max_per_connection
    }
    
    /// Get current symbol count
    pub fn symbol_count(&self) -> usize {
        self.subscribed_symbols.len()
    }
    
    /// Check if connection is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_healthy.load(Ordering::Relaxed)
    }
}
