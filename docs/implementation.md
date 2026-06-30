# Implementation Plan: Simulation-First MEV System

## Executive Summary

This document details the implementation of a **simulation-first** Aave V3 MEV system targeting Base and Polygon mainnets. The core philosophy: **prove profitability in simulation before risking any capital**.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SIMULATION MODE (Phases 1-3)                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐                 │
│  │  Python      │────▶│  Go Gateway  │────▶│  Rust        │                 │
│  │  Quant       │     │  (REST/gRPC) │     │  Executor    │                 │
│  │              │     │              │     │              │                 │
│  │ • Pricing    │     │ • Router     │     │ • Mempool    │                 │
│  │ • Risk Model │     │ • Watchdog   │     │ • Simulator  │                 │
│  │ • Strategy   │     │ • Aave Cache │     │ • Chain Mgr  │                 │
│  │ • Backtest   │     │ • Risk Engine│     │              │                 │
│  └──────────────┘     └──────────────┘     └──────────────┘                 │
│        ▲                                      │                              │
│        │              ┌──────────────┐        │                              │
│        └──────────────│  Anvil Fork  │◀───────┘                              │
│                       │  (Mainnet    │                                       │
│                       │   State)     │                                       │
│                       └──────────────┘                                       │
│                                                                              │
│  Paper P&L ──────▶ Grafana Dashboard ──▶ Go/No-Go Decision                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           EXECUTION MODE (Phase 4+)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  Same architecture + Signer + Flashbots + Hardware Wallet                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Phase Breakdown

### Phase 0: Foundation (Week 1)
**Goal**: Scaffold complete repo, CI/CD, local dev environment

| Task | Output |
|------|--------|
| Monorepo structure | `contracts/`, `services/executor-rust/`, `services/gateway-go/`, `services/quant-python/` |
| Docker + docker-compose | One-command local stack |
| GitHub Actions CI | Build, test, lint for all 3 languages |
| Anvil fork setup | Auto-updating mainnet fork |
| .gitignore | Exclude secrets, build artifacts, venv, target, .env |

**Acceptance**: `make up` starts all services + Anvil fork; `make test` passes.

---

### Phase 1: Contracts (Week 1-2)
**Goal**: Deploy-ready Solidity contracts with fork tests

#### Contracts

| Contract | Purpose | Key Functions |
|----------|---------|---------------|
| `Liquidator.sol` | Aave V3 liquidations | `liquidate(address user, address collateral, address debt)` |
| `FlashLoanArb.sol` | Cross-DEX arb via flash loan | `executeOperation(address[] assets, uint256[] amounts, ...)` |
| `RateArbLoop.sol` | Variable↔Stable rate arb + looping | `executeRateArb()`, `executeLoop()` |
| `MockExecutor.sol` | Simulation helper | `simulate(bytes calldata) returns (bool, bytes)` |

#### Interfaces
- `IAavePool.sol` — `IPool` subset
- `IERC20.sol`, `IERC20Metadata.sol`
- `IUniswapV3Router.sol`, `ICurvePool.sol`, `IBalancerVault.sol`

#### Tests (Foundry)
- Fork tests on Base/Polygon mainnet state
- Liquidation: create underwater position → liquidate → profit
- FlashLoanArb: find cross-DEX price diff → execute → repay + profit
- RateArbLoop: borrow variable → lend stable (or loop) → net yield > cost
- Coverage >90% on core logic

**Acceptance**: `forge test --fork-url $BASE_RPC -vvv` passes; gas reports captured.

---

### Phase 2: Rust Executor (Week 2-3)
**Goal**: Mempool listener + `eth_call` simulator + multi-chain manager

#### Crate Structure
```
executor-rust/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry, config, graceful shutdown
│   ├── config.rs            # Env parsing, chain configs
│   ├── chain/
│   │   ├── mod.rs
│   │   ├── manager.rs       # Multi-chain RPC pool, health checks
│   │   └── provider.rs      # Alloy provider wrapper
│   ├── mempool/
│   │   ├── mod.rs
│   │   ├── listener.rs      # WSS pending tx subscription
│   │   ├── filter.rs        # Target tx detection (liquidations, swaps)
│   │   └── decoder.rs       # Calldata decoding for known routers
│   ├── simulator/
│   │   ├── mod.rs
│   │   ├── engine.rs        # eth_call to MockExecutor + trace
│   │   ├── tracer.rs        # Trace parsing: gas, state diff, profit
│   │   └── types.rs         # SimResult, SimRequest
│   ├── api/
│   │   ├── mod.rs
│   │   ├── grpc.rs          # Tonic server for Gateway
│   │   └── rest.rs          # Actix-web for health/metrics
│   └── metrics.rs           # Prometheus metrics
```

#### Key Implementation Details

**Mempool Listener**
```rust
// Subscribe to pending transactions via eth_subscribe("newPendingTransactions")
// Filter for: Aave liquidation calls, Uniswap/Curve/Balancer swaps, flash loan callbacks
// Decode calldata using alloy-sol-types generated bindings
// Forward candidates to Simulator
```

**Simulator**
```rust
// eth_call to MockExecutor.simulate(targetCalldata)
// Parse trace: stateDiff → token balance changes → profit calculation
// Return SimResult { profit_usd, gas_used, success, trace }
```

**Multi-Chain Manager**
```rust
// ChainConfig { name, rpc_ws, rpc_http, chain_id, contracts }
// Health check: block lag < 2, RPC latency < 500ms
// Automatic failover to backup RPC
```

#### gRPC API (Gateway ↔ Executor)
```protobuf
service Executor {
  rpc Simulate(SimulateRequest) returns (SimulateResponse);
  rpc Health(HealthRequest) returns (HealthResponse);
  rpc SubscribeMempool(MempoolFilter) returns (stream MempoolEvent);
}
```

**Acceptance**: 24h WSS uptime; simulates 100 tx/min with <50ms p99 latency.

---

### Phase 3: Go Gateway (Week 3-4)
**Goal**: Central coordinator, watchdog, risk engine, Aave state cache

#### Module Structure
```
gateway-go/
├── go.mod
├── cmd/server/main.go
├── internal/
│   ├── config/           # Viper config
│   ├── router/           # Chi HTTP + gRPC gateway
│   │   ├── http.go       # REST: /health, /metrics, /simulate, /positions
│   │   └── grpc.go       # gRPC client to Executor
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
```

#### Risk Engine (Paper Trading)
```go
type PaperPosition struct {
    Strategy     string
    Chain        string
    Collateral   string
    Debt         string
    SizeUSD      float64
    EntryPrice   float64
    CurrentPrice float64
    UnrealizedPnL float64
}

// On simulation result:
func (r *RiskEngine) EvaluateSimResult(sim *SimResult) *RiskDecision {
    // 1. Check strategy daily loss limit
    // 2. Check max concurrent positions
    // 3. Check capital allocation per chain
    // 4. Return Approve/Reject with size adjustment
}
```

#### Aave Cache
- Subscribe to `Pool` events via `eth_subscribe("logs")`
- Maintain in-memory map: `reserve → ReserveData { liquidityRate, variableBorrowRate, stableBorrowRate, price, ... }`
- TTL: 1 block; refresh on new block header

**Acceptance**: Gateway routes simulate request → executor → returns result in <200ms; watchdog trips circuit breaker in <1s on RPC failure injection.

---

### Phase 4: Python Quant (Week 4-5)
**Goal**: Signal generation, pricing, risk modeling, backtesting

#### Module Structure
```
quant-python/
├── pyproject.toml
├── src/quant/
│   ├── __init__.py
│   ├── main.py              # FastAPI app
│   ├── config.py            # Pydantic settings
│   ├── pricing/
│   │   ├── __init__.py
│   │   ├── dex.py           # Uniswap V3 quoter, Curve get_dy, Balancer query
│   │   ├── aave.py          # Rate calculations, health factor
│   │   └── oracle.py        # Chainlink + Aave price aggregation
│   ├── risk_model/
│   │   ├── __init__.py
│   │   ├── health_factor.py # HF projection under price shocks
│   │   ├── liquidation.py   # Liquidation threshold, bonus, penalty
│   │   └── loop.py          # Loop optimization: max leverage s.t. HF > 1.05
│   ├── strategies/
│   │   ├── __init__.py
│   │   ├── base.py          # Strategy interface
│   │   ├── liquidator.py    # Scan for liquidatable positions
│   │   ├── flash_arb.py     # Cross-DEX arb detection
│   │   └── rate_arb.py      # Variable/stable rate arb + looping
│   ├── backtest/
│   │   ├── __init__.py
│   │   ├── runner.py        # Replay historical blocks via Anvil snapshots
│   │   ├── data.py          # Load block traces, events
│   │   └── metrics.py       # Sharpe, max DD, win rate, profit factor
│   └── api/
│       ├── __init__.py
│       ├── routes.py        # POST /signal, GET /backtest, GET /health
│       └── schemas.py       # Pydantic models
```

#### Strategy Interface
```python
class BaseStrategy(ABC):
    @abstractmethod
    async def scan(self, ctx: ScanContext) -> List[Opportunity]: ...

    @abstractmethod
    async def build_calldata(self, opp: Opportunity) -> bytes: ...

    @abstractmethod
    def estimate_profit(self, opp: Opportunity) -> ProfitEstimate: ...
```

#### Pricing Module
- **Uniswap V3**: `QuoterV2.quoteExactInputSingle` via `eth_call`
- **Curve**: `get_dy(i, j, dx)` via `eth_call`
- **Balancer**: `Vault.queryBatchSwap` via `eth_call`
- **Slippage model**: `expected_out * (1 - slippage_bps / 10000)`

#### Backtest Runner
```python
# 1. Load historical block range (e.g., last 10k blocks)
# 2. For each block: fork Anvil at block N
# 3. Run all strategies scan()
# 4. Simulate top opportunities via Executor
# 5. Record: would-have-profit, gas, success
# 6. Aggregate metrics
```

**Acceptance**: Backtest runs 10k blocks in <30min; produces strategy comparison report.

---

### Phase 5: Integration & Simulation (Week 5-6)
**Goal**: End-to-end paper trading on live fork

#### Docker Compose Stack
```yaml
services:
  anvil:
    image: ghcr.io/foundry-rs/foundry:latest
    command: anvil --fork-url ${BASE_RPC} --fork-block-number ${LATEST} --host 0.0.0.0 --port 8545 --block-time 2
    ports: ["8545:8545"]

  executor:
    build: ./services/executor-rust
    environment:
      - RPC_WS=ws://anvil:8545
      - RPC_HTTP=http://anvil:8545
      - CHAIN=base
      - MOCK_EXECUTOR_ADDRESS=0x...
    depends_on: [anvil]

  gateway:
    build: ./services/gateway-go
    environment:
      - EXECUTOR_GRPC=executor:50051
      - QUANT_HTTP=http://quant:8000
      - RPC_HTTP=http://anvil:8545
    ports: ["8080:8080", "9090:9090"]
    depends_on: [executor, quant]

  quant:
    build: ./services/quant-python
    environment:
      - GATEWAY_HTTP=http://gateway:8080
      - RPC_HTTP=http://anvil:8545
    ports: ["8000:8000"]
    depends_on: [gateway]

  grafana:
    image: grafana/grafana:latest
    ports: ["3000:3000"]
    volumes: ["./grafana:/etc/grafana/provisioning"]
    depends_on: [prometheus]

  prometheus:
    image: prom/prometheus:latest
    ports: ["9090:9090"]
    volumes: ["./prometheus.yml:/etc/prometheus/prometheus.yml"]
```

#### Paper Trading Loop
```python
# quant-python main loop
async def paper_trading_loop():
    while True:
        ctx = ScanContext(
            block=await get_latest_block(),
            aave_cache=await fetch_aave_state(),
            dex_quotes=await fetch_dex_quotes(),
        )
        
        for strategy in STRATEGIES:
            opportunities = await strategy.scan(ctx)
            for opp in opportunities:
                sim_req = SimulateRequest(calldata=strategy.build_calldata(opp))
                sim_res = await gateway.simulate(sim_req)
                
                if sim_res.profit_usd > MIN_PROFIT_USD:
                    risk_decision = await gateway.risk_check(sim_res)
                    if risk_decision.approved:
                        await log_paper_trade(opp, sim_res, risk_decision)
        
        await asyncio.sleep(BLOCK_TIME)
```

#### Metrics to Track
| Metric | Target |
|--------|--------|
| Opportunities scanned/block | >50 |
| Simulation success rate | >80% |
| Avg simulation latency | <100ms |
| Paper win rate (200+ trades) | >65% |
| Avg profit / gas | >3x |

**Acceptance**: 7-day continuous paper trading with positive expectancy.

---

### Phase 6: Mainnet Deployment (Week 7+)
**Only if Phase 5 passes Go/No-Go**

1. Deploy contracts to Base + Polygon mainnet
2. Update `.choreo/` configs with real addresses
3. Enable `signer.rs` with hardware wallet (Ledger/Trezor via `hidapi`)
4. Add Flashbots bundle submission for mev-share
5. Start with **Liquidator only**, $500 capital
6. Scale after 1 week profitable operation

---

## Technical Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| RPC Provider | QuickNode (primary) + Alchemy (backup) | Best WSS stability, mev-share support |
| gRPC vs REST | gRPC (Gateway↔Executor), REST (Quant→Gateway) | Low latency for hot path; simplicity for Quant |
| Mempool | Native `eth_subscribe` + Flashbots mev-share | Capture both public and private order flow |
| Simulation | `eth_call` to `MockExecutor` + `debug_traceCall` | Accurate gas + state diff without mining |
| Multi-chain | Single Executor process, multi-provider pool | Simpler ops; ChainManager handles routing |
| Secrets | `.env` (local), Choreo secrets (cloud), Hardware wallet (mainnet) | No keys in repo or container images |
| Observability | Prometheus + Grafana + structured JSON logs | Standard, free, scales |

---

## File Structure (Final)

```
.
├── .github/workflows/ci.yml
├── .gitignore
├── docker-compose.yml
├── docker-compose.prod.yml
├── Makefile
├── README.md
├── docs/
│   ├── implementation.md      # This file
│   ├── structure.md
│   └── phases.md
├── contracts/
│   ├── foundry.toml
│   ├── src/
│   │   ├── Liquidator.sol
│   │   ├── FlashLoanArb.sol
│   │   ├── RateArbLoop.sol
│   │   ├── MockExecutor.sol
│   │   ├── interfaces/
│   │   │   ├── IAavePool.sol
│   │   │   ├── IUniswapV3Router.sol
│   │   │   ├── ICurvePool.sol
│   │   │   └── IBalancerVault.sol
│   │   └── libraries/
│   │       ├── Math.sol
│   │       └── CallbackValidation.sol
│   ├── test/
│   │   ├── Liquidator.t.sol
│   │   ├── FlashLoanArb.t.sol
│   │   ├── RateArbLoop.t.sol
│   │   └── fork/
│   │       ├── BaseLiquidation.t.sol
│   │       └── PolygonArb.t.sol
│   └── script/
│       ├── Deploy.s.sol
│       └── Verify.s.sol
├── services/
│   ├── executor-rust/
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   ├── .env.example
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── config.rs
│   │   │   ├── chain/
│   │   │   ├── mempool/
│   │   │   ├── simulator/
│   │   │   ├── api/
│   │   │   └── metrics.rs
│   │   └── tests/
│   │       └── integration_tests.rs
│   ├── gateway-go/
│   │   ├── go.mod
│   │   ├── go.sum
│   │   ├── Dockerfile
│   │   ├── .env.example
│   │   ├── cmd/server/main.go
│   │   ├── internal/
│   │   │   ├── config/
│   │   │   ├── router/
│   │   │   ├── watchdog/
│   │   │   ├── aave/
│   │   │   ├── risk/
│   │   │   └── quant/
│   │   └── proto/
│   │       └── executor.proto
│   └── quant-python/
│       ├── pyproject.toml
│       ├── poetry.lock
│       ├── Dockerfile
│       ├── .env.example
│       ├── src/quant/
│       │   ├── main.py
│       │   ├── config.py
│       │   ├── pricing/
│       │   ├── risk_model/
│       │   ├── strategies/
│       │   ├── backtest/
│       │   └── api/
│       └── tests/
│           ├── test_pricing.py
│           ├── test_risk_model.py
│           ├── test_strategies.py
│           └── test_backtest.py
├── tests/
│   └── robot/
│       ├── resources/
│       │   ├── common.robot
│       │   └── variables.py
│       ├── suites/
│       │   ├── simulation.robot
│       │   ├── contracts.robot
│       │   ├── integration.robot
│       │   └── backtest.robot
│       └── results/
├── grafana/
│   ├── dashboards/
│   │   ├── paper_trading.json
│   │   ├── system_health.json
│   │   └── strategy_performance.json
│   └── datasources/
│       └── prometheus.yml
├── prometheus.yml
└── .choreo/
    ├── component.yaml
    └── endpoints.yaml
```

---

## RobotFramework Test Strategy

### Test Suites

| Suite | Scope | Key Keywords |
|-------|-------|--------------|
| `contracts.robot` | Foundry fork tests | `Deploy Contract`, `Simulate Liquidation`, `Verify Profit` |
| `simulation.robot` | Executor sim accuracy | `Start Anvil Fork`, `Submit Tx To Mempool`, `Verify Sim Result` |
| `integration.robot` | E2E paper trading loop | `Start Stack`, `Wait For Opportunity`, `Verify Paper PnL` |
| `backtest.robot` | Quant backtest validity | `Run Backtest`, `Check Metrics`, `Compare Strategies` |

### Example Robot Test
```robot
*** Settings ***
Library    requests
Library    JSONLibrary
Resource   ../resources/common.robot

*** Test Cases ***
Liquidation Opportunity Found And Simulated
    [Documentation]    End-to-end: Quant detects liquidatable position -> Gateway routes -> Executor simulates -> Profit verified
    Start Full Stack
    ${position}=    Create Underwater Position    collateral=WETH    debt=USDC    health_factor=0.95
    Wait Until Keyword Succeeds    30    2s    Opportunity Detected    strategy=liquidator    position=${position}
    ${sim_result}=    Get Latest Simulation Result    strategy=liquidator
    Should Be True    ${sim_result.profit_usd} > 10
    Should Be Equal As Numbers    ${sim_result.success}    ${True}
    Log    Paper trade logged: ${sim_result.profit_usd} USD profit
```

---

## Go/No-Go Criteria for Capital Deployment

| Criterion | Threshold | Measurement Period |
|-----------|-----------|-------------------|
| Paper win rate | ≥ 65% | 7 days, ≥ 200 sims |
| Avg profit / gas | ≥ 3.0x | 7 days |
| Max drawdown (paper) | < 5% | 7 days |
| Simulation accuracy (vs actual) | < 10% gas error | 50 real txns |
| Zero critical bugs | 0 | 7 days uptime |
| Watchdog false positive rate | < 1% | 7 days |

**Only proceed to Phase 6 if ALL criteria met.**

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| RPC failure during opportunity | Multi-provider pool; automatic failover; watchdog pauses trading |
| Simulation ≠ execution | MockExecutor replicates exact execution path; trace validation |
| MEV competition | Flashbots Protect + mev-share; simulate with `block.basefee * 2` |
| Smart contract bug | 90%+ coverage; formal verification on core math; 2-week testnet |
| Liquidation race | Priority gas auction simulation; bid shading model |
| Regulatory | Geo-fencing; no US persons; legal review before mainnet |

---

## Appendix: Environment Variables

### Shared (all services)
```bash
# Chain config
BASE_RPC_WS=wss://base.rpc.url
BASE_RPC_HTTP=https://base.rpc.url
POLYGON_RPC_WS=wss://polygon.rpc.url
POLYGON_RPC_HTTP=https://polygon.rpc.url

# Contracts (set after deploy)
LIQUIDATOR_ADDRESS=0x...
FLASH_LOAN_ARB_ADDRESS=0x...
RATE_ARB_LOOP_ADDRESS=0x...
MOCK_EXECUTOR_ADDRESS=0x...

# Aave V3
AAVE_POOL_ADDRESS_BASE=0x...
AAVE_POOL_ADDRESS_POLYGON=0x...
AAVE_ORACLE_ADDRESS_BASE=0x...
AAVE_ORACLE_ADDRESS_POLYGON=0x...
```

### Executor
```bash
EXECUTOR_GRPC_PORT=50051
EXECUTOR_HTTP_PORT=8081
CHAINS=base,polygon
MIN_PROFIT_USD=5
MAX_GAS_PRICE_GWEI=100
```

### Gateway
```bash
GATEWAY_HTTP_PORT=8080
GATEWAY_GRPC_PORT=9090
EXECUTOR_GRPC_URL=http://executor:50051
QUANT_HTTP_URL=http://quant:8000
WATCHDOG_INTERVAL_MS=1000
CIRCUIT_BREAKER_THRESHOLD=5
```

### Quant
```bash
QUANT_HTTP_PORT=8000
STRATEGIES_ENABLED=liquidator,flash_arb,rate_arb
MIN_PROFIT_USD=10
MAX_POSITION_USD=5000
BACKTEST_BLOCKS=10000
```

---

## Next Actions

1. ✅ Create this `implementation.md`
2. ⬜ Scaffold monorepo with `.gitignore`, `Makefile`, `docker-compose.yml`
3. ⬜ Implement contracts + Foundry tests
4. ⬜ Implement Rust Executor
5. ⬜ Implement Go Gateway
6. ⬜ Implement Python Quant
7. ⬜ Write RobotFramework tests
8. ⬜ Update all docs
9. ⬜ Run integration test suite