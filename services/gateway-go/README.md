# Go Gateway Service

This service serves as the API Gateway and control center of the MEV system.

## Key Features

- **API Router**: HTTP and gRPC routing between services
- **Watchdog**: Process monitoring and circuit breaker implementation
- **Aave Cache**: On-chain state caching and indexing
- **Risk Management**: Position tracking and exposure control
- **Service Coordination**: Integration between Executor and Quant services

## Architecture

```go
gateway-go/
├── cmd/server/main.go
├── internal/
│   ├── config/           # Configuration management
│   ├── router/           # HTTP/gRPC routing
│   ├── watchdog/         # Process monitoring and recovery
│   ├── aave/             # On-chain state caching
│   ├── risk/             # Risk calculation and limits
│   └── cache/            # Distributed caching
├── proto/               # gRPC service definitions
└── go.mod
```

## Role

This is the system's "中樞" (hub) responsible for:
- Coordinating cross-service communication
- Monitoring service health and performance
- Managing risk exposure and circuit breakers
- Caching critical on-chain data
