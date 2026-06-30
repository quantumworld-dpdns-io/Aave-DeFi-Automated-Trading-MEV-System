# 專案目錄結構 (Monorepo)

此專案採用 Monorepo 架構，將鏈上智能合約與鏈下三語言（Rust, Go, Python）微服務整合，以便於跨組件的 API 協定與 CI/CD 部署。

```text
.
├── contracts/                  # 鏈上智能合約 (Solidity / Foundry)
│   ├── src/
│   │   ├── FlashLoanArb.sol    # 閃電貸跨池套利邏輯合約
│   │   ├── Liquidator.sol      # 清算執行合約
│   │   └── interfaces/         # Aave V3 與 DEX (Uniswap/Curve) 介面
│   ├── test/                   # Foundry 測試檔 (Solidity)
│   └── foundry.toml            # Foundry 設定檔
│
├── services/                   # 鏈下微服務集群
│   ├── executor-rust/          # [四肢] 極速鏈上執行引擎
│   │   ├── src/
│   │   │   ├── mempool.rs      # Mempool 監聽與 WebSocket 連線
│   │   │   ├── signer.rs       # 交易簽名與 Gas 競價邏輯
│   │   │   └── main.rs         # 核心執行緒
│   │   ├── Dockerfile
│   │   └── Cargo.toml          # 依賴套件 (ethers-rs / alloy)
│   │
│   ├── gateway-go/             # [中樞] 系統調度與 Watchdog 守護進程
│   │   ├── cmd/
│   │   │   └── server/main.go  # API Gateway 進入點
│   │   ├── internal/
│   │   │   ├── watchdog/       # 斷路器與系統健康監控模組
│   │   │   ├── router/         # gRPC/HTTP 路由轉發
│   │   │   └── aave/           # Aave 協議狀態快取
│   │   ├── Dockerfile
│   │   └── go.mod
│   │
│   └── quant-python/           # [大腦] 量化分析與風險模型
│       ├── src/
│       │   ├── pricing.py      # 即時定價與滑點計算模型
│       │   ├── risk_model.py   # LTV 與健康係數動態模型
│       │   └── main.py         # FastAPI 伺服器
│       ├── Dockerfile
│       └── requirements.txt
│
├── .choreo/                    # Choreo.dev 平台部署配置
│   ├── endpoints.yaml          # 服務對外/對內 Port 定義
│   └── component.yaml          # 各微服務構建環境與變數映射
│
├── docker-compose.yml          # 本地端整合測試環境配置
├── README.md                   # 專案主說明文檔
└── structure.md                # 結構說明文檔 (本檔)
```
