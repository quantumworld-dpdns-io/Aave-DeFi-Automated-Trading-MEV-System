# Rust Executor Service

This service implements the mempool monitoring and transaction execution engine.

## Key Features

- **Mempool Monitoring**: WebSocket connections to blockchain nodes for real-time transaction monitoring
- **Transaction Parsing**: Identification and analysis of MEV opportunities
- **Execution Engine**: Simulated and real transaction execution with gas optimization
- **Chain Management**: Multi-chain support with failover capabilities

## Architecture

```rust
executor-rust/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── chain/
│   │   ├── manager.rs      # RPC connection management
│   │   └── provider.rs     # WebSocket client
│   ├── mempool/
│   │   ├── listener.rs      # Transaction subscription
│   │   ├── filter.rs        # MEV opportunity detection
│   │   └── analyzer.rs     # Transaction analysis
│   ├── simulator/
│   │   ├── engine.rs        # Simulation logic
│   │   └── tracer.rs       # Debug trace processing
│   ├── api/
│   │   ├── grpc.rs          # gRPC server
│   │   └── http.rs          # HTTP API
│   └── metrics.rs           # Monitoring and metrics
└── Cargo.toml
```

## Role

This is the system's "四肢" (limbs) responsible for:
- Real-time blockchain monitoring
- Transaction parsing and filtering
- Simulated execution for profit validation
- Real-world execution in production
