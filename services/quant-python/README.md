# Python Quant Service

This service serves as the quant analysis and trading signal generation component.

## Key Features

- **Pricing Models**: Real-time price oracles and slippage estimation
- **Risk Models**: Position sizing and exposure calculations
- **Strategy Optimization**: Multi-strategy opportunity selection
- **Backtesting**: Historical performance analysis
- **HTTP API**: FastAPI for external integration

## Architecture

```python
quant-python/
├── src/
│   ├── __init__.py
│   ├── main.py              # FastAPI application
│   ├── config.py            # Configuration management
│   ├── pricing/             # Price estimation modules
│   ├── risk_model/          # Risk assessment engine
│   ├── strategies/          # Trading strategy implementations
│   ├── backtest/            # Historical analysis
│   └── api/                 # API endpoints
├── Dockerfile
├── requirements.txt
└── pyproject.toml
```

## Role

This is the system's "大脑" (brain) responsible for:
- Analyzing market data and identifying opportunities
- Calculating optimal positions and risk parameters
- Generating trading signals for on-chain execution
- Continuous model improvement
