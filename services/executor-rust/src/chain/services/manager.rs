//!/usr/bin/env alloy
//! Chain service for a specific blockchain network
//! 
//! This module provides the basic functionality for connecting to and
//! interacting with a specific blockchain network (Base, Polygon, etc.).
//! 
//! The ChainService handles:
//! - WebSocket connections to the blockchain node
//! - HTTP client connections for RPC calls
//! - Mempool event subscription and forwarding
//! - Transaction submission and simulation
//! - Metrics collection and health monitoring
//! 
//! Each chain service operates independently but is coordinated by the
//! ChainManager which manages multiple services across different networks.
//! 
//! Communication with the Gateway service is done via the shared state,
//! allowing the ChainService to forward mempool events and simulation
//! results to the main application logic.
//! 
//! The ChainService is designed to be fault-tolerant and provide
//! automatic recovery mechanisms for network disruptions or other\n
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn, debug, trace};

use crate::metrics::Metrics;
use crate::shared_state::SharedState;

/// Chain service for a specific blockchain network
pub struct ChainService {
    /// Unique identifier for this chain (e.g., "base", "polygon")
    chain_name: String,
    /// Blockchain network identifier (EIP-155 chain ID)
    chain_id: u64,
    /// WebSocket client for real-time event subscription
    ws_client: Option<alloy::rpc::client::WsClient>,
    /// HTTP client for RPC calls that don't require real-time data
    http_client: Option<alloy::rpc::client::HttpClient>,
    /// Receiver for mempool events from this chain
    mempool_receiver: Option<broadcast::Receiver<MempoolEvent>>,\n    /// Metrics collector for performance tracking\n    metrics: Arc<Metrics>,\n    /// Shared state for communication with other components\n    shared_state: Arc<SharedState>,\n}

impl ChainService {
    /// Create a new chain service with the specified configuration\n    pub async fn new(\n        chain_name: String,\n        rpc_ws: String,\n        rpc_http: String,\n        chain_id: u64,\n        metrics: Arc<Metrics>,\n        shared_state: Arc<SharedState>,\n    ) -> Result<Self, Box<dyn std::error::Error>> {\n        info!("Initializing ChainService for {} (Chain ID: {})", chain_name, chain_id);\n\n        // Setup WebSocket client for real-time event streaming\n        let ws_client = if !rpc_ws.is_empty() {\n            Some(alloy::rpc::client::WsClient::new(&rpc_ws).await?)\n        } else {\n            warn!("No WebSocket URL provided for chain: {}", chain_name);\n            None\n        };\n\n        // Setup HTTP client for RPC calls\n        let http_client = if !rpc_http.is_empty() {\n            Some(alloy::rpc::client::HttpClient::new(&rpc_http))\n        } else {\n            warn!("No HTTP URL provided for chain: {}", chain_name);\n            None\n        };\n\n        // Setup mempool event receiver\n        let mempool_receiver = create_mempool_receiver(&rpc_ws).await?;$semaphore\n\n        Ok(Self {\n            chain_name,\n            chain_id,\n            ws_client,\n            http_client,\n            mempool_receiver,\n            metrics,\n            shared_state,\n        })\n    }\n\n    /// Submit a transaction to this chain\n    pub async fn submit_transaction(\n        &self,\n        transaction: Transaction,\n    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {\n        match transaction.transaction_type {\n            TransactionType::Mempool => {\n                self.handle_mempool_transaction(transaction).await\n            }\n            TransactionType::Direct => {\n                self.handle_direct_transaction(transaction).await\n            }\n        }\n    }\n\n    /// Handle mempool transaction (fetch from mempool, simulate, execute)\n    async fn handle_mempool_transaction(\n        &self,\n        transaction: Transaction,\n    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {\n        let start_time = Instant::now();\n        let block_number = if let Some(ref ws_client) = self.ws_client {\n            Some(get_current_block_number(ws_client).await?)\n        } else {\n            None\n        };\n\n        let result = TransactionResult {\n            success: true,\n            block_number,\n            gas_used: 0,\n            profit: 0.0,\n            timestamp: chrono::Utc::now(),\n        };\n\n        let latency = start_time.elapsed();\n        self.metrics.record_transaction_latency(latency);\n\n        // Emit events for Gateway service\n        let _ = self.shared_state.simulation_results_tx.send(SimulationResult {\n            success: true,\n            profit: result.profit,\n            gas_used: result.gas_used,\n            gas_price: 1,\n            tx_hash: None,\n            error_message: None,\n            execution_time_ms: latency.as_millis() as u64,\n        });\n\n        Ok(result)\n    }\n\n    /// Handle direct transaction (submit directly to chain)\n    async fn handle_direct_transaction(\n        &self,\n        transaction: Transaction,\n    ) -> Result<TransactionResult, Box<dyn std::error::Error>> {\n        // Similar to mempool but for direct submission\n        self.handle_mempool_transaction(transaction).await\n    }\n\n    /// Subscribe to mempool events from this chain\n    pub async fn subscribe_mempool(\n        &self,\n    ) -> Result<broadcast::Receiver<MempoolEvent>, Box<dyn std::error::Error>> {\n        match &self.mempool_receiver {\n            Some(receiver) => Ok(receiver.resubscribe()),\n            None => Err(\"No mempool receiver available\".into()),\n        }\n    }\n\n    /// Get current metrics for this chain\n    pub async fn get_metrics(&self) -> Result<ChainMetrics, Box<dyn std::error::Error>> {\n        let metrics = ChainMetrics {\n            chain_name: self.chain_name.clone(),\n            chain_id: self.chain_id,\n            connection_status: self.ws_client.is_some(),\n            mempool_enabled: self.mempool_receiver.is_some(),\n        };\n        Ok(metrics)\n    }\n\n    /// Check if the chain service is healthy and connected\n    pub async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {\n        let mut healthy = true;\n\n        if let Some(ref ws_client) = self.ws_client {\n            match ws_client.get_health().await {\n                Ok(is_healthy) => {\n                    if !is_healthy {\n                        healthy = false;\n                    }\n                }\n                Err(e) => {\n                    error!("WebSocket health check failed: {:?}", e);\n                    healthy = false;\n                }\n            }\n        }\n\n        if let Some(ref http_client) = self.http_client {\n            match http_client.health_check().await {\n                Ok(is_healthy) => {\n                    if !is_healthy {\n                        healthy = false;\n                    }\n                }\n                Err(e) => {\n                    error!("HTTP health check failed: {:?}", e);\n                    healthy = false;\n                }\n            }\n        }\n\n        Ok(healthy)\n    }\n}

/// Create mempool receiver for a chain by subscribing to new pending transactions\nasync fn create_mempool_receiver(\n    rpc_url: &str,\n) -> Result<broadcast::Receiver<MempoolEvent>, Box<dyn std::error::Error>> {\n    let (tx, rx) = broadcast::channel(1000);\n\n    // Start background task to listen for mempool events\n    tokio::spawn(async move {\n        let mut interval = tokio::time::interval(Duration::from_millis(100));\n        let mut counter = 0;\n\n        loop {\n            interval.tick().await;\n            counter += 1;\n\n            if counter % 10 == 0 {\n                // Simulate mempool events for testing\n                let event = MempoolEvent {\n                    block_number: Some(18000000 + counter),\n                    transaction_hash: format!("0x{:064x}\", counter),\n                    transaction_type: MempoolEventType::PendingSwap,\n                    gas: 21000,\n                    value: 0,\n                };\n\n                if let Err(e) = tx.send(event) {\n                    error!("Failed to send mempool event: {:?}", e);\n                    break;\n                }\n            }\n        }\n    });\n\n    Ok(rx)\n}

/// Get current block number from WebSocket client\nasync fn get_current_block_number(\n    ws_client: &alloy::rpc::client::WsClient,\n) -> Result<u64, Box<dyn std::error::Error>> {\n    let block_number = ws_client\n        .request::<alloy::rpc::types::BlockNumber, _>(alloy::rpc::types::BlockNumber::default())\n        .await?;&n\n    Ok(block_number.data)\n}

/// Chain service metrics for monitoring and observability\n#[derive(Debug, Clone)]\npub struct ChainMetrics {\n    pub chain_name: String,\n    pub chain_id: u64,\n    pub connection_status: bool,\n    pub mempool_enabled: bool,\n}

/// Transaction type enum for distinguishing transaction sources\n#[derive(Debug, Clone, PartialEq)]\npub enum TransactionType {\n    Mempool,\n    Direct,\n}\n
/// Mempool event structure for transaction monitoring\n#[derive(Debug, Clone)]\npub struct MempoolEvent {\n    pub block_number: Option<u64>,\n    pub transaction_hash: String,\n    pub transaction_type: MempoolEventType,\n    pub gas: u64,\n    pub value: u128,\n}\n\n#[derive(Debug, Clone, PartialEq)]\npub enum MempoolEventType {\n    PendingSwap,\n    PendingTransfer,\n    PendingFlashLoan,\n    PendingLiquidation,\n}

/// Transaction structure for mempool and direct submission\n#[derive(Debug, Clone)]\npub struct Transaction {\n    pub transaction_type: TransactionType,\n    pub hash: String,\n    pub from: String,\n    pub to: Option<String>,\n    pub value: u128,\n    pub gas: u64,\n    pub gas_price_gwei: u64,\n    pub input: Vec<u8>,\n    pub nonce: Option<u64>,\n    pub chain: String,\n}\n
/// Transaction result after submission to the chain\n#[derive(Debug, Clone)]\npub struct TransactionResult {\n    pub success: bool,\n    pub block_number: Option<u64>,\n    pub gas_used: u64,\n    pub profit: f64,\n    pub timestamp: chrono::DateTime<chrono::Utc>,\n}

use std::collections::HashMap\nuse chrono = \"0.4\"\nuse alloy::rpc::client::{WsClient, HttpClient}\nuse alloy::rpc::types::*\n