# Aave DeFi MEV System - Rust Executor

This service implements the mempool monitoring and transaction execution engine for the Aave MEV system. It's responsible for real-time monitoring of pending transactions on the blockchain and executing profitable opportunities found by the Quant brain.

## Key Features

- **Mempool Monitoring**: WebSocket connections to blockchain nodes for real-time transaction monitoring
- **Opportunity Detection**: Identification and analysis of MEV opportunities (arbitrage, liquidations, rate arbitrage)
- **Transaction Execution**: Simulated and real transaction execution with gas optimization
- **Chain Management**: Multi-chain support with automatic failover

## Architecture

```rust
executor-rust/
├── Cargo.toml
├── src/
│   ├── main.rs              # Application entry point and configuration
│   ├── config.rs            # Environment and connection settings
│   ├── chain/
│   │   ├── mod.rs          # Chain abstraction layer
│   │   ├── manager.rs      # RPC connection management
│   │   └── provider.rs     # WebSocket client implementation
│   ├── mempool/
│   │   ├── mod.rs          # Mempool monitoring subsystem
│   │   ├── listener.rs      # WSS connection and subscription handling
│   │   ├── filter.rs        # Transaction filtering for MEV targets
│   │   └── analyzer.rs     # Transaction analysis and target identification
│   ├── simulator/
│   │   ├── mod.rs          # Simulation engine interface
│   │   ├── engine.rs        # Core simulation logic
│   │   └── tracer.rs       # Call trace processing
│   ├── api/
│   │   ├── grpc.rs          # gRPC server implementation
│   │   └── http.rs          # HTTP API for monitoring
│   └── metrics.rs           # Prometheus metrics exporter
```

## Role in System Architecture

The Executor service is the system's "四肢" (limbs) responsible for:
- Real-time blockchain monitoring via WebSocket connections
- Transaction parsing and filtering for MEV opportunities
- Simulated execution for profit validation
- Real-world execution in production

## Main Components

### Mempool Listener
```rust
// Connect to WebSocket RPC nodes via alloy client
// Subscribe to pending transactions via eth_subscribe("newPendingTransactions")
// Filter for: large swaps, liquidation calls, flash loan callbacks
// Decode calldata using alloy-sol-types generated bindings
// Forward filtered transactions to Simulator
```

### Transaction Simulator
```rust
// Use debug_traceTransaction RPC method for state validation
// Extract profit from ERC20 balance changes
// Gas estimation and optimization
// Flashbots bundle submission support
```

### Chain Manager
```rust
// Manage connections to multiple blockchain networks (Base, Polygon)
// Handle automatic failover between RPC providers
// Maintain connection health and reconnection logic
// Track mempool metrics and performance
```

## Key Implementation Details

### Connection Management
The Executor service establishes and maintains WebSocket connections to blockchain nodes:

```rust
// Establish connections to multiple chains
const CHAINS: &[ChainConfig] = &[
    ChainConfig {
        name: "base",
        rpc_ws: std::env::var("RPC_WSS_URL_BASE").unwrap(),
        rpc_http: std::env::var("RPC_HTTP_URL_BASE").unwrap(),
        chain_id: 8453,
    },
    // ... other chains
];
```

### Message Passing
The Executor communicates with the Gateway service via gRPC:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct SimulateRequest {
    pub target: String,           // Target transaction hash or calldata
    pub calldata: Vec<u8>,       // Transaction calldata
    pub gas_limit: u64,          // Estimated gas limit
    pub max_profit: u128,        // Maximum expected profit (USD)
    pub strategy: StrategyType,  // Arbitrage, Liquidation, or Rate Arbitrage
    pub chain: String,           // Chain identifier
}
```

## Performance Requirements

### Latency Targets
- **Mempool Processing**: <50ms p99
- **WebSocket Connection**: 24/7 uptime with <3s reconnect
- **gRPC Communication**: <200ms round-trip time

### Throughput
- **Transaction Filtering**: 100,000 tx/min
- **Simulation Processing**: 1,000 tx/sec
- **Opportunity Detection**: <1ms per opportunity

## Security Considerations

### Memory Safety
- All Rust code benefits from Rust's memory safety guarantees
- No use of unsafe blocks except where absolutely necessary
- Comprehensive fuzzing with Echidna

### Network Security
- TLS-secured WebSocket connections
- gRPC authentication and authorization
- Circuit breaker implementation in Gateway

### Data Validation
- Input validation for all transaction calldata
- Signature verification for simulated transactions
- Profit extraction from trusted sources only

## Testing Strategy

### Unit Tests
```rust
// mempool/filter.rs
#[test]
fn test_liquidation_detection() {
    let tx = mock_transaction("0x1234...", "LiquidationCall");
    let filter = MempoolFilter::new();
    let result = filter.should_process(&tx);
    assert_eq!(result, true);
}
```

### Integration Tests
```rust
// tests/integration.rs
#[tokio::test]
async fn test_end_to_end_simulation() {
    // Start Executor service
    // Submit opportunity to Gateway
    // Verify execution results
    // Check profit calculation accuracy
}
```

## Configuration

### Environment Variables
```bash
# WebSocket RPC URLs
RPC_WSS_URL_BASE=wss://base-mainnet.g.alchemy.com/v2/YOUR_API_KEY
RPC_WSS_URL_POLYGON=wss://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY

# gRPC Configuration
EXECUTOR_GRPC_PORT=50051
GATEWAY_GRPC_URL=ws://localhost:9090

# Gas Configuration
MIN_GAS_PRICE_GWEI=1
MAX_GAS_PRICE_GWEI=100
GAS_LIMIT_BUFFER_PERCENT=110

# Profit Thresholds
MIN_PROFIT_USD=10
MIN_PROFIT_PERCENT=0.1
```

## Development Commands

### Building
```bash
# Build all services
cargo build --release

# Build specific services
cargo build --lib --tests
```

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test integration

# Run performance tests
cargo test performance
```

### Running
```bash
# Run the Executor service
cargo run --release

# With development features enabled
cargo run --features dev
```

## Future Enhancements

### Advanced Features
1. **Flashbots Integration**: Direct bundle submission to Flashbots relays
2. **MEV-Share**: Participation in private mempool
3. **Dynamic Gas Optimization**: Real-time gas price prediction
4. **Multi-strategic Orchestration**: Coordinated execution across strategies

### Performance Improvements
1. **Asynchronous Processing**: Non-blocking mempool parsing
2. **Parallel Simulation**: Multiple transactions processed concurrently
3. **Caching**: Frequently accessed data cached for faster access
4. **Compression**: Transaction data compression for reduced bandwidth

## Monitoring & Observability

### Metrics
- **Connection Metrics**: Active connections, reconnects, errors
- **Processing Metrics**: Transactions processed, opportunities found, execution successes
- **Performance Metrics**: P99 latency, throughput, memory usage

### Logging
- **Structured Logging**: JSON logs for easy parsing
- **Alerting**: Integration with monitoring systems
- **Debugging**: Detailed logs for troubleshooting

The Executor service is the foundation of the MEV system, providing the real-time blockchain monitoring and execution capabilities that enable profitable trading opportunities while maintaining security and reliability.