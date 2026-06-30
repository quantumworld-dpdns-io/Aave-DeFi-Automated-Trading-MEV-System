# ==========================================
# Aave DeFi MEV System - Repository Structure
# ==========================================

本倉庫是一個多語言(Aave DeFi自動化交易與MEV捕獲系統。系統採用微服務架構，結合Solidity智能合約與Rust、Go、Python三種語言的底層優勢，實現低延遲監聽、高併發調度與動態量化定價。

## 主要架構

### 区块链层 (Blockchain Layer)
- **智能合约 (Smart Contracts)**: Solidity合约，使用Foundry进行开发与测试
  - FlashLoanArb.sol: 跨池套利逻辑
  - Liquidator.sol: 清算执行合约
  - RateArbLoop.sol: 动态利率套利与循环逻辑
  - MockExecutor.sol: 模拟执行合约
  - interfaces/: Aave V3与DEX接口定义

### 应用层 (Application Layer)
- **Executor (Rust)**: 极速链上执行引擎
  - mempool.rs: Mempool监听与WebSocket连接
  - signer.rs: 交易签名与Gas竞价逻辑
  - simulator.rs: 模拟执行与利润计算
  - main.rs: 核心执行引擎
- **Gateway (Go)**: 系统枢纽
  -  gateway: API路由转发器
  - watchdog: 断路器与系统健康监控
  - aave: Aave协议状态缓存
  - internal: 服务间通信
- **Quant (Python)**: 量化分析与风险模型
  - pricing.py: 即时定价与滑点计算模型
  - risk_model.py: LTV与健康系数动态模型
  - strategy_selector.py: 策略选择与优化
  - main.py: FastAPI服务端

### 基础设施层 (Infrastructure Layer)
- CI/CD: GitHub Actions workflows
- Docker Compose: 本地开发环境
- Orchestrator: Choreo.dev部署配置
- Monitoring: 指标收集与可视化

## 项目结构

```
├── .choreo/                    # Choreo.dev部署配置
│   ├── component.yaml          # 服务构建环境与变量映射
│   └── endpoints.yaml          # 服务对外/对内端口定义
├── .github/workflows/         # CI/CD管道
├── .gitignore                  # 忽略文件
├── contracts/                  # 智能合约层
│   ├── src/                   # 合约源代码
│   │   ├── FlashLoanArb.sol
│   │   ├── Liquidator.sol
│   │   ├── RateArbLoop.sol
│   │   ├── MockExecutor.sol
│   │   └── interfaces/
│   ├── test/                   # Foundry测试
│   │   ├── FlashLoanArb.t.sol
│   │   ├── Liquidator.t.sol
│   │   ├── RateArbLoop.t.sol
│   │   └── Integration.t.sol
│   └── foundry.toml           # Foundry配置
├── docker-compose.yml         # 本地集成测试环境
├── Makefile                    # 构建与测试命令
├── README.md                   # 项目说明文档
├── docs/                       # 技术文档
│   ├── implementation.md       # 技术实现文档
│   ├── structure.md           # 项目结构说明
│   └── Phases & Acceptance Criteria.md  # 开发阶段与验收标准
├── CONTRIBUTING.md            # 贡献指南
├── Implementation Plan.md      # 项目实施计划
├── LICENSE                     # 许可证
├── counter.json                # 文件计数器
└── auto_commit.sh              # 自动提交脚本
```

## 部署指南

### 本地开发
```bash
# 1. 编译智能合约
cd contracts
forge build

# 2. 执行本地分叉测试
forge test --rpc-url <YOUR_FORK_RPC_URL> -vvv

# 3. 启动本地微服务集群
cd ..
docker-compose up --build
```

### 云端部署 (Choreo.dev)
1. 连接GitHub仓库至Choreo项目
2. 配置环境变量
3. 启动部署

## 系统设计理念

### 1. 微服务架构
- **Executor (Rust)**: 负责区块链数据监听与交易执行
- **Gateway (Go)**: 负责服务协调与风险控制
- **Quant (Python)**: 负责市场分析与策略生成

### 2. 模拟执行机制
- **重要**: 生产环境前进行所有策略验证
- **MockExecutor.sol**: 智能合约模拟执行器
- **Paper Trading**: 离线资金验证
- **风险控制**: 严格的暴露限制与断路器

### 3. 多链支持
- 支持Base与Polygon网络
- 可扩展至更多L2网络
- 动态路由优化

### 4. 实时监控
- P&L跟踪
- 系统健康状态
- 交易延迟监控
- 风险指标预警

## 技术选型

| 组件 | 语言 | 原因 |
|------|--------|------|
| 执行引擎 | Rust | 高性能网络栈，内存安全 |
| API网关 | Go | 简单并发性，成熟生态 |
| 量化分析 | Python | 数据科学库，快速原型 |
| 智能合约 | Solidity | 最佳DeFi开发环境 |

## 验证标准

### 阶段一：智能合约与本地分叉 (Phase 1)
- 合约编译无警告与错误
- Flash loan流程验证
- 测试覆盖率>90%
- 安全审计通过

### 阶段二：核心引擎与系统看门狗 (Phase 2)
- Rust引擎24小时稳定运行
- WebSocket自动重连
- 断路器响应<1秒
- 系统健康监控

### 阶段三：量化大脑与系统串接 (Phase 3)
- Python模型<100ms响应
- 全链路E2E通过
- 纸交易验证
- 风险控制验证

### 阶段四：云端部署与测试网验证 (Phase 4)
- Choreo.dev部署
- 测试网验证
- 环境变量验证
- 交易成功与Gas估算准确

### 阶段五：主网上线与极限调优 (Phase 5)
- 主网部署
- 动态Gas竞价优化
- P&L监控面板
- 性能优化与调优

## 安全与风险控制

### 智能合约安全
- 形式验证
- 静态分析 (Slither, Mythril)
- 模糊测试 (Echidna)
- 多方代码审查

### 运行时风险控制
- 断路器模式
- 暴露限制
- 紧急平仓
- 实时监控

### 审计与合规
- 定期安全审计
- 交易记录
- 合规检查
- 风险报告

## 成功指标

### 技术指标
- 系统可用性:>99.9%
- 交易延迟:<100ms
- 资金利用率:>20%

### 商业指标
- 收益率:>15% APR
- 胜率:>65%
- 最大回撤:<10%
- 夏普比率:>2.0

## 开发工具

### 本地开发
- Docker Compose
- GitHub Actions
- Foundry (Solidity)
- cargo (Rust)
- go (Go)
- poetry (Python)

### 测试
- Robot Framework (集成测试)
- Foundry (合约测试)
- `make test` (全流程)

### 监控
- Prometheus
- Grafana
- 日志聚合

## 未来演进

1. **高级策略**: 机器学习模型，AI交易信号
2. **多链扩展**: 支援更多L2网络
3. **跨协议**: 支持更多DeFi协议
4. **机构级**: 批量处理与专业功能
5. **智能合约**: 自动更新与升级

## 协作指南

### 本地开发
1. `make build` - 构建所有Docker镜像
2. `make up` - 启动本地开发环境
3. `make test` - 运行所有测试
4. `make delete` - 清理所有容器与数据

### 代码贡献
1. 遵循CONTRIBUTING.md中的准则
2. 所有PR需经过代码审查
3. 需要测试覆盖率>90%
4. 安全审计通过

### 部署
1. 配置Choreo.dev环境变量
2. 使用`make build`构建镜像
3. 使用`make up`启动服务
4. 验证系统健康状态

## 免责声明

本项目为技术研究用途，旨在探索分布式系统设计与实现。所有投资风险由用户自行承担，项目团队不对任何交易损失负责。

---

*最后更新: $(date)*
*版本: v1.0.0*