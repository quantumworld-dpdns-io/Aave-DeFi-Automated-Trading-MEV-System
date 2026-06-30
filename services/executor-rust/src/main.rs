// SPDX-License-Identifier: MIT
//! Aave DeFi MEV System - Rust Executor Service
//! 
//! This is the core execution engine of the MEV system, responsible for:
//! - Mempool monitoring via WebSocket connections
//! - Transaction parsing and MEV opportunity detection
//! - Simulated execution validation using MockExecutor
//! - Real-world transaction execution with gas optimization
//! - Multi-chain support with automatic failover
//! 
//! The service runs continuously, monitoring for profitable opportunities
//! and executing them with proper risk management controls.

use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tracing::{error, info, warn};

mod chain;
mod config;
mod mempool;
mod simulator;
mod api;
mod metrics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Aave MEV Executor Service");

    // Load configuration
    let config = config::Config::from_env()?;

    // Create shared metrics
    let metrics = Arc::new(metrics::Metrics::new());

    // Initialize chain manager
    let chain_manager = chain::manager::ChainManager::new(config.chains)?;

    // Start mempool monitoring
    let mempool_monitor = mempool::monitor::MempoolMonitor::new(
        chain_manager.clone(),
        metrics.clone(),
    );

    // Start simulator
    let simulator = simulator::Simulator::new(
        chain_manager.clone(),
        metrics.clone(),
    );

    // Start API server (gRPC + HTTP)
    let api_server = api::server::ApiServer::new(
        simulator.clone(),
        metrics.clone(),
    );

    // Start all components concurrently
    let mempool_handle = tokio::spawn(async move {
        if let Err(e) = mempool_monitor.run().await {
            error!("Mempool monitor error: {:?}", e);
        }
    });

    let simulator_handle = tokio::spawn(async move {
        if let Err(e) = simulator.run().await {
            error!("Simulator error: {:?}", e);
        }
    });

    let api_handle = tokio::spawn(async move {
        if let Err(e) = api_server.run().await {
            error!("API server error: {:?}", e);
        }
    });

    // Display service status
    info!("Executor Service Started");
    info!("  Chains configured: {}", config.chains.len());
    info!("  RPC endpoints: {}", chain_manager.get_endpoint_count());
    info!("  API gRPC listening on port 50051");
    info!("  HTTP API listening on port 8080");
    info!("\nService Commands:");
    info!("  - gRPC client: tonic --proto=path/to/proto/executor.proto");
    info!("  - Check metrics: curl http://localhost:8080/metrics");
    info!("  - Health check: curl http://localhost:8080/health");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;

    info!("Shutting down executor service...");

    // Gracefully shutdown all components
    mempool_handle.abort();
    simulator_handle.abort();
    api_handle.abort();

    info!("Executor service shutdown complete.");

    Ok(())
}