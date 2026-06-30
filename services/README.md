# Services Overview

This directory contains the off-chain microservices for the Aave MEV system:

## Python Services (quant-python)
Located at `services/quant-python/`
- Signal generation and market analysis
- Risk modeling and position sizing
- Integration with on-chain data sources

## Go Services (gateway-go)  
Located at `services/gateway-go/`
- API Gateway
- Circuit breaker and monitoring
- Service coordination and risk management

## Rust Services (executor-rust)
Located at `services/executor-rust/`
- Mempool monitoring and transaction parsing
- Transaction simulation and execution
- WebSocket connections to blockchain
