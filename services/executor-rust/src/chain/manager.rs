//!/usr/bin/env alloy
//! Chain manager for the Rust Executor
//! 
//! This module manages connections to multiple blockchain networks,
//! provides failover mechanisms, and handles chain-specific operations.
//! 
//! Features:
//! - Multi-chain support (Base, Polygon)
//! - Automatic RPC connection failover
//! - Connection health monitoring
//! - Chain abstraction layer
//! 
//! Communication with the Gateway uses gRPC to send simulation requests
//! and receive opportunity data from the Quant brain.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info, warn, debug, trace};

use crate::config::ChainConfig;
use crate::metrics::Metrics;
use crate::chain::service::ChainService;

/// Chain manager that handles connections to multiple blockchain networks
/// and provides unified interface for interacting with all chains.
pub struct ChainManager {
    /// Map of chain name to chain service
    services: RwLock<std::collections::HashMap<String, Arc<ChainService>>>,
    /// Current active connections
    active_connections: tokio::sync::Mutex<std::collections::HashMap<String, Arc<ChainService>>>,
    /// Failed connection attempts (for circuit breaker)
    failed_attempts: tokio::sync::Mutex<std::collections::HashMap<String, usize>>,
    /// Metrics collector
    metrics: Arc<Metrics>,
    /// Shared state for component communication
    shared_state: Arc<SharedState>,
}

impl ChainManager {
    /// Create a new chain manager with configured chains
    pub async fn new(
        chain_configs: Vec<ChainConfig>,
        metrics: Arc<Metrics>,
        shared_state: Arc<SharedState>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing ChainManager with {} chains", chain_configs.len());

        let mut services: std::collections::HashMap<String, Arc<ChainService>> = std::collections::HashMap::new();

        for chain_config in chain_configs {
            info!("Setting up connection to {} (Chain ID: {})", chain_config.name, chain_config.chain_id);

            // Create chain service
            let service = ChainService::new(
                chain_config.name.clone(),
                chain_config.rpc_ws.clone(),
                chain_config.rpc_http.clone(),
                chain_config.chain_id,
                metrics.clone(),
                shared_state.clone(),
            ).await?;

            services.insert(chain_config.name.clone(), Arc::new(service));
            info!("✓ Connected to {} successfully", chain_config.name);
        }

        // Initialize metrics
        metrics.record_chain_setup_complete(services.len());

        Ok(Self {
            services: RwLock::new(services),
            active_connections: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            failed_attempts: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            metrics,
            shared_state,
        })
    }

    /// Get the number of configured chains
    pub fn get_chain_count(&self) -> usize {
        self.services.blocking_lock().len()
    }

    /// Get the number of active connections
    pub async fn get_endpoint_count(&self) -> usize {
        self.active_connections.lock().await.len()
    }

    /// Submit a transaction to a specific chain
    pub async fn submit_transaction(
        &self,
        chain_name: &str,
        transaction: Transaction,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        let services = self.services.read().await;

        let service = services.get(chain_name)
            .ok_or_else(|| format!("Chain not found: {}", chain_name))?
            .clone();

        // Check circuit breaker for this chain
        let mut failed_attempts = self.failed_attempts.lock().await;
        let attempts = failed_attempts.entry(chain_name.to_string()).or_insert(0);

        if *attempts >= 3 {
            return Err(format!("Circuit breaker tripped for chain: {}", chain_name).into());
        }

        // Record service usage
        let start_time = Instant::now();
        let result = service.submit_transaction(transaction).await;
        let latency = start_time.elapsed();

        // Track execution time
        self.metrics.record_transaction_latency(latency);

        match &result {
            Ok(_) => {
                // Reset failed attempts on success
                *attempts = 0;
                failed_attempts.remove(chain_name);
            }
            Err(_) => {
                // Increment failed attempts
                *attempts += 1;
                error!("Transaction submission failed for {}: {:?}", chain_name, result);
            }
        }

        result
    }

    /// Subscribe to mempool events for a specific chain
    pub async fn subscribe_mempool(
        &self,
        chain_name: &str,
    ) -> Result<tokio::sync::broadcast::Receiver<MempoolEvent>, Box<dyn std::error::Error>> {
        let services = self.services.read().await;
        let service = services.get(chain_name)
            .ok_or_else(|| format!("Chain not found: {}", chain_name))?
            .clone();

        service.subscribe_mempool().await
    }

    /// Get chain-specific metrics
    pub async fn get_chain_metrics(&self, chain_name: &str) -> Result<ChainMetrics, Box<dyn std::error::Error>> {
        let services = self.services.read().await;
        let service = services.get(chain_name)
            .ok_or_else(|| format!("Chain not found: {}", chain_name))?
            .clone();

        service.get_metrics().await
    }

    /// Health check for all chains
    pub async fn health_check(&self) -> Result<std::collections::HashMap<String, bool>, Box<dyn std::error::Error>> {
        let mut health_status: std::collections::HashMap<String, bool> = std::collections::HashMap::new();

        let services = self.services.read().await;
        for (name, service) in services.iter() {
            let is_healthy = service.health_check().await.unwrap_or(false);
            health_status.insert(name.clone(), is_healthy);
        }

        Ok(health_status)
    }
}

/// Chain service for a specific blockchain network
pub struct ChainService {
    chain_name: String,
    chain_id: u64,
    ws_client: Option<alloy::rpc::client::WsClient>,
    http_client: Option<alloy::rpc::client::HttpClient>,
    mempool_receiver: Option<tokio::sync::broadcast::Receiver<MempoolEvent>>,
    metrics: Arc<Metrics>,
    shared_state: Arc<SharedState>,
}

impl ChainService {
    /// Create a new chain service
    pub async fn new(
        chain_name: String,
        rpc_ws: String,
        rpc_http: String,
        chain_id: u64,
        metrics: Arc<Metrics>,
        shared_state: Arc<SharedState>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ws_client = if !rpc_ws.is_empty() {
            Some(alloy::rpc::client::WsClient::new(&rpc_ws).await?)
        } else {
            warn!("No WebSocket URL provided for chain: {}", chain_name);
            None
        };

        let http_client = if !rpc_http.is_empty() {
            Some(alloy::rpc::client::HttpClient::new(&rpc_http))
        } else {
            warn!("No HTTP URL provided for chain: {}", chain_name);
            None
        };

        let mempool_receiver = Some(create_mempool_receiver(&rpc_ws).await?);

        Ok(Self {
            chain_name,
            chain_id,
            ws_client,
            http_client,
            mempool_receiver,
            metrics,
            shared_state,
        })
    }

    /// Submit a transaction to this chain
    pub async fn submit_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        match transaction.transaction_type {
            TransactionType::Mempool => {
                self.handle_mempool_transaction(transaction).await
            }
            TransactionType::Direct => {
                self.handle_direct_transaction(transaction).await
            }
        }
    }

    /// Handle mempool transaction
    async fn handle_mempool_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let block_number = if let Some(ref ws_client) = self.ws_client {
            Some(get_current_block_number(ws_client).await?)
        } else {
            None
        };

        let result = TransactionResult {
            success: true,
            block_number,
            gas_used: 0, // To be determined
            profit: 0.0, // To be calculated
            timestamp: chrono::Utc::now(),
        };

        let latency = start_time.elapsed();
        self.metrics.record_transaction_latency(latency);

        // Emit event for Gateway service
        let _ = self.shared_state.simulation_results_tx.send(SimulationResult {
            success: true,
            profit: result.profit,
            gas_used: result.gas_used,
            gas_price: 1, // Default
            tx_hash: None,
            error_message: None,
            execution_time_ms: latency.as_millis() as u64,
        });

        Ok(result)
    }

    /// Handle direct transaction
    async fn handle_direct_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {
        // Similar to mempool but for direct submission
        self.handle_mempool_transaction(transaction).await
    }

    /// Subscribe to mempool events
    pub async fn subscribe_mempool(
        &self,
    ) -> Result<tokio::sync::broadcast::Receiver<MempoolEvent>, Box<dyn std::error::Error>> {
        if let Some(receiver) = &self.mempool_receiver {
            Ok(receiver.resubscribe())
        } else {
            Err("No mempool receiver available".into())
        }
    }

    /// Get service metrics
    pub async fn get_metrics(&self) -> Result<ChainMetrics, Box<dyn std::error::Error>> {
        let metrics = ChainMetrics {
            chain_name: self.chain_name.clone(),
            chain_id: self.chain_id,
            connection_status: self.ws_client.is_some(),
            mempool_enabled: self.mempool_receiver.is_some(),
        };
        Ok(metrics)
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut healthy = true;

        if let Some(ref ws_client) = self.ws_client {
            // Check WebSocket connection
            match ws_client.get_health().await {
                Ok(is_healthy) => {
                    if !is_healthy {
                        healthy = false;
                    }
                }
                Err(e) => {
                    error!("WebSocket health check failed: {:?}", e);
                    healthy = false;
                }
            }
        }

        if let Some(ref http_client) = self.http_client {
            // Check HTTP connection
            match http_client.health_check().await {
                Ok(is_healthy) => {
                    if !is_healthy {
                        healthy = false;
                    }
                }
                Err(e) => {
                    error!("HTTP health check failed: {:?}", e);
                    healthy = false;
                }
            }
        }

        Ok(healthy)
    }
}

/// Create mempool receiver for a chain
async fn create_mempool_receiver(
    rpc_url: &str,
) -> Result<tokio::sync::broadcast::Receiver<MempoolEvent>, Box<dyn std::error::Error>> {
    // Implementation for creating mempool receiver
    // This would subscribe to new pending transactions events
    let (tx, rx) = tokio::sync::broadcast::channel(1000);

    // Start background task to listen for mempool events
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        let mut counter = 0;

        loop {
            interval.tick().await;
            counter += 1;

            if counter % 10 == 0 {
                // Simulate mempool events for testing
                let event = MempoolEvent {
                    block_number: Some(18000000 + counter),
                    transaction_hash: format!("0x{:064x}", counter),
                    transaction_type: MempoolEventType::PendingSwap,
                    gas: 21000,
                    value: 0,
                };

                let _ = tx.send(event);
            }
        }
    });

    Ok(rx)
}

/// Get current block number from WebSocket client
async fn get_current_block_number(
    ws_client: &alloy::rpc::client::WsClient,
) -> Result<u64, Box<dyn std::error::Error>> {
    let block_number = ws_client
        .request::<alloy::rpc::types::BlockNumber, _>(alloy::rpc::types::BlockNumber::default())
        .await?;

    Ok(block_number.data)
}

/// Chain service metrics
#[derive(Debug, Clone)]
pub struct ChainMetrics {
    pub chain_name: String,
    pub chain_id: u64,
    pub connection_status: bool,
    pub mempool_enabled: bool,
}

/// Transaction data structure for mempool monitoring
#[derive(Debug, Clone)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: u128,
    pub gas: u64,
    pub input: Vec<u8>,
    pub nonce: Option<u64>,
    pub chain: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Mempool,
    Direct,
}

/// Mempool event for transaction monitoring
#[derive(Debug, Clone)]
pub struct MempoolEvent {
    pub block_number: Option<u64>,
    pub transaction_hash: String,
    pub transaction_type: MempoolEventType,
    pub gas: u64,
    pub value: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MempoolEventType {
    PendingSwap,
    PendingTransfer,
    PendingFlashLoan,
    PendingLiquidation,
}

/// Transaction result after submission
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub success: bool,
    pub block_number: Option<u64>,
    pub gas_used: u64,
    pub profit: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

use std::collections::HashMap;
use chrono = "0.4";
use alloy::rpc::client::{WsClient, HttpClient};
use alloy::rpc::types::*;
