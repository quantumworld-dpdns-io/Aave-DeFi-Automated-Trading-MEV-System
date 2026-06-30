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
├── cmd/
│   └── server/main.go
├── internal/
│   ├── config/           # Viper configuration management
│   ├── router/           # HTTP and gRPC routing layer
│   │   ├── http.go       # REST: /health, /metrics, /simulate, /positions
│   │   └── grpc.go       # gRPC server for Executor communication
│   ├── watchdog/
│   │   ├── monitor.go    # Service heartbeats, RPC health
│   │   ├── circuitbreaker.go  # Trip on: RPC lag, margin drop, error rate
│   │   └── notifier.go   # Alert webhook (Slack/Discord)
│   ├── aave/
│   │   ├── cache.go      # Reserve data cache (rates, prices, caps)
│   │   ├── indexer.go    # Event indexing: Supply, Borrow, LiquidationCall
│   │   └── types.go
│   ├── risk/
│   │   ├── engine.go     # Paper position tracking, exposure limits
│   │   ├── limits.go     # Max position, max daily loss, per-strategy caps
│   │   └── pnl.go        # Realized/unrealized P&L calculation
│   └── quant/
│       └── client.go     # HTTP client to Python Quant
└── proto/               # gRPC definitions
    └── gateway.proto
```

## Role in System Architecture

The Go Gateway service is the system's "中樞" (hub) responsible for:
- Coordinating cross-service communication
- Monitoring service health and performance
- Managing risk exposure and circuit breakers
- Caching critical on-chain data
- Integrating Quant signals with Executor execution

## Key Implementation Details

### Router Module
```go
// REST API for external communication
func (s *Server) setupHTTPRoutes() {
    mux := mux.NewRouter()
    mux.HandleFunc("/health", s.handleHealthCheck)
    mux.HandleFunc("/api/v1/simulate", s.handleSimulateRequest)
    mux.HandleFunc("/api/v1/metrics", s.handleMetrics)
    mux.HandleFunc("/api/v1/positions", s.handlePositions)
    http.ListenAndServe(":8080", mux)
}

// gRPC server for Executor communication
func (s *Server) setupGrpcServer() {
    grpcServer := grpc.NewServer(
        grpc.ChainUnaryInterceptor(s.authInterceptor),
        grpc.ChainUnaryInterceptor(s.loggingInterceptor),
    )
    pb.RegisterGatewayServer(grpcServer, s)
    go func() {
        if err := grpcServer.Serve(listener); err != nil {
            log.Fatalf("gRPC server error: %v", err)
        }
    }()
}
```

### Watchdog Module
```go
// Service health monitoring
func (s *Watchdog) Start() {
    go s.monitorServices()
    go s.healthChecker()
    go s.circuitBreaker()
}

func (s *Watchdog) monitorServices() {
    ticker := time.NewTicker(30 * time.Second)
    defer ticker.Stop()

    for {
        select {
        case <-ticker.C:
            s.checkServiceHealth()
        case <-s.shutdown:
            return
        }
    }
}

func (s *Watchdog) checkServiceHealth() {\n    // Implementation for service health checking\n}\n\nfunc (s *Watchdog) circuitBreaker() {\n    // Implementation for circuit breaker logic\n}
```

## Metrics\n
Tracking performance and health of the gateway components:
\n```go\n// Service metrics tracking
metrics := prometheus.NewRegistry()
\\middleware := middleware.New(prometheus.NewMiddleware(prometheus.Get").Middleware)\n\\r\\n// Register custom metrics\nmetrics.MustRegister(prometheus.NewCounter(prometheus.CounterOpts{
    Name: \"gateway_requests_total\",
    Help: \"Total number of HTTP requests handled by the Gateway.\",
}))\n```\n\n## Performance Considerations\n\n### Concurrency
- **Non-blocking HTTP server**: Using `http.Server` for concurrent request handling\n- **gRPC streaming**: Low-latency gRPC for internal service communication\n\n### Resource Management
- **Circuit breakers**: Prevent cascading failures\n- **Connection pooling**: Efficient reuse of TCP connections\n- **Memory management**: Proper garbage collection for long-running connections\n\n### Security\n- **Authentication**: API key validation and rate limiting\n- **Authorization**: Role-based access control\n- **TLS**: Encrypted communication with mutual authentication\n
## Testing Strategy\n\n### Unit Tests\n```go\ntest gateway_test.go {\n    // Test router functionality\n    // Test health check endpoint\n    // Test error handling and validation\n}\n```\n\n### Integration Tests\n```bash\n# Test inter-service communication\n# Validate gateway routing and coordination\n# Test circuit breaker functionality\n# Validate integration with Executor and Quant services\n\nmake test-integration\n```\n\n### Load Testing\n```bash\n# Test gateway under high load\n# Validate latency and throughput\n# Test connection pooling and resource limits\n\ngodep run loadtest/gatewayLoadTest.go\n```\n\n## Future Enhancements\n\n### Advanced Features
1. **Dynamic Health Checks**: Customizable health check endpoints for each service\n2. **Distributed Tracing**: OpenTelemetry integration for request tracking\n3. **Service Discovery**: Kubernetes integration for service registration\n4. **Configuration Management**: External configuration store with hot reload\n\n### Performance Improvements
1. **gRPC Websockets**: Bi-directional streaming for real-time event processing\n2. **Load Balancing**: Multi-zone deployment with automatic failover\n3. **Caching**: Redis integration for frequently accessed data\n4. **Compression**: Request/response compression for reduced bandwidth\n\n## Development Workflow\n\n### Local Development\n```bash\n# Start the Go Gateway service\ncd services/gateway-go\ngo run ./cmd/server\n\n# Check service health\ncurl http://localhost:8080/health\n\n# Test API endpoints\ncurl http://localhost:8080/api/v1/chains\n```\n\n### Docker Development\n```bash\n# Build Docker image\ndocker build -t mev-system/gateway-go ./services/gateway-go\n\n# Run with Docker Compose\ndocker-compose up gateway-go\n```\n\n### Testing and Validation\n```bash\n# Run unit tests\ngo test ./internal/...\n\n# Run integration tests\ngo test ./integration/\n\n# Run load tests\ngo run loadtest/main.go\n```\n\n## Conclusion\n\nThe Go Gateway service is a critical component of the MEV system, providing:\n- **Centralized coordination** of all system services\n- **Robust monitoring** and health checks\n- **Risk management** and exposure controls\n- **Infrastructure for scaling** and future enhancements\n
It ensures reliable, secure, and efficient communication between the Quant analysis services and the Executor service, enabling profitable trading opportunities while maintaining system stability and security.