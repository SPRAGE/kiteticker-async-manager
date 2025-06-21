use crate::manager::{ConnectionStats, ManagerStats};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;

/// Health monitor for tracking connection and system health
#[derive(Debug)]
pub struct HealthMonitor {
    pub connection_stats: Vec<Arc<RwLock<ConnectionStats>>>,
    pub manager_start_time: Instant,
    pub monitoring_task: Option<JoinHandle<()>>,
    pub health_check_interval: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(
        connection_stats: Vec<Arc<RwLock<ConnectionStats>>>,
        health_check_interval: Duration,
    ) -> Self {
        Self {
            connection_stats,
            manager_start_time: Instant::now(),
            monitoring_task: None,
            health_check_interval,
        }
    }
    
    /// Start the health monitoring task
    pub fn start(&mut self) {
        let connection_stats = self.connection_stats.clone();
        let health_check_interval = self.health_check_interval;
        let manager_start_time = self.manager_start_time;
        
        let handle = tokio::spawn(async move {
            Self::monitoring_loop(
                connection_stats,
                health_check_interval,
                manager_start_time,
            ).await;
        });
        
        self.monitoring_task = Some(handle);
    }
    
    /// Health monitoring loop
    async fn monitoring_loop(
        connection_stats: Vec<Arc<RwLock<ConnectionStats>>>,
        health_check_interval: Duration,
        manager_start_time: Instant,
    ) {
        log::info!("Health monitor started with interval: {:?}", health_check_interval);
        
        loop {
            sleep(health_check_interval).await;
            
            // Collect health information
            let mut healthy_connections = 0;
            let mut total_symbols = 0;
            let mut total_messages = 0;
            let mut total_errors = 0;
            
            for (i, stats_arc) in connection_stats.iter().enumerate() {
                let mut stats = stats_arc.write().await;
                
                // Update connection uptime
                if stats.is_connected {
                    stats.connection_uptime = manager_start_time.elapsed();
                    healthy_connections += 1;
                }
                
                total_symbols += stats.symbol_count;
                total_messages += stats.messages_received;
                total_errors += stats.errors_count;
                
                // Log individual connection health
                log::debug!(
                    "Connection {}: healthy={}, symbols={}, messages={}, errors={}",
                    i,
                    stats.is_connected,
                    stats.symbol_count,
                    stats.messages_received,
                    stats.errors_count
                );
            }
            
            // Log overall health
            log::info!(
                "Health Check: {}/{} connections healthy, {} total symbols, {} messages, {} errors",
                healthy_connections,
                connection_stats.len(),
                total_symbols,
                total_messages,
                total_errors
            );
            
            // Alert on issues
            if healthy_connections == 0 {
                log::error!("CRITICAL: All connections are unhealthy!");
            } else if healthy_connections < connection_stats.len() {
                log::warn!(
                    "WARNING: {}/{} connections are unhealthy",
                    connection_stats.len() - healthy_connections,
                    connection_stats.len()
                );
            }
        }
    }
    
    /// Get comprehensive manager statistics
    pub async fn get_manager_stats(&self) -> ManagerStats {
        let mut manager_stats = ManagerStats::default();
        manager_stats.uptime = self.manager_start_time.elapsed();
        
        let mut active_connections = 0;
        
        for stats_arc in &self.connection_stats {
            let stats = stats_arc.read().await;
            
            if stats.is_connected {
                active_connections += 1;
            }
            
            manager_stats.total_symbols += stats.symbol_count;
            manager_stats.total_messages_received += stats.messages_received;
            manager_stats.total_errors += stats.errors_count;
            manager_stats.connection_stats.push(stats.clone());
        }
        
        manager_stats.active_connections = active_connections;
        manager_stats
    }
    
    /// Get health summary
    pub async fn get_health_summary(&self) -> HealthSummary {
        let mut summary = HealthSummary::default();
        
        for (i, stats_arc) in self.connection_stats.iter().enumerate() {
            let stats = stats_arc.read().await;
            
            if stats.is_connected {
                summary.healthy_connections += 1;
            } else {
                summary.unhealthy_connections.push(i);
            }
            
            summary.total_symbols += stats.symbol_count;
            summary.total_messages += stats.messages_received;
            summary.total_errors += stats.errors_count;
            
            // Calculate message rate (messages per second over last minute)
            if let Some(last_msg_time) = stats.last_message_time {
                if last_msg_time.elapsed() < Duration::from_secs(60) {
                    summary.active_message_flows += 1;
                }
            }
        }
        
        summary.uptime = self.manager_start_time.elapsed();
        summary
    }
    
    /// Stop the health monitor
    pub async fn stop(&mut self) {
        if let Some(handle) = self.monitoring_task.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

/// Health summary for quick status checks
#[derive(Debug, Clone, Default)]
pub struct HealthSummary {
    pub healthy_connections: usize,
    pub unhealthy_connections: Vec<usize>,
    pub total_symbols: usize,
    pub total_messages: u64,
    pub total_errors: u64,
    pub active_message_flows: usize,
    pub uptime: Duration,
}

impl HealthSummary {
    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        self.unhealthy_connections.is_empty() && self.total_errors == 0
    }
    
    /// Check if the system is degraded (some connections unhealthy)
    pub fn is_degraded(&self) -> bool {
        !self.unhealthy_connections.is_empty() && self.healthy_connections > 0
    }
    
    /// Check if the system is critical (all connections unhealthy)
    pub fn is_critical(&self) -> bool {
        self.healthy_connections == 0
    }
    
    /// Get health percentage (0-100)
    pub fn health_percentage(&self) -> f64 {
        let total_connections = self.healthy_connections + self.unhealthy_connections.len();
        if total_connections == 0 {
            100.0
        } else {
            (self.healthy_connections as f64 / total_connections as f64) * 100.0
        }
    }
}
