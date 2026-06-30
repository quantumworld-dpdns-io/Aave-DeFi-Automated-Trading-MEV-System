# DeFi Automated Trading & MEV System

本專案是一個基於 Aave V3 協議的自動化交易與最大可提取價值 (MEV) 捕捉系統。系統採用微服務架構，結合 Solidity 智能合約與 Rust、Go、Python 三種語言的底層優勢，實現低延遲監聽、高併發調度與動態量化定價。

## 系統架構設計

系統分為「鏈上合約」與「鏈下微服務」兩大核心：

### 1. 鏈下微服務 (Off-chain Microservices)
* **Executor (Rust):** 擔任系統的「四肢」。透過 WebSocket 與區塊鏈 RPC 節點保持長連線，負責 Mempool 監聽、交易打包、簽名與極速廣播，以應對高頻 Gas 競爭。
* **Gateway / Watchdog (Go):** 擔任系統的「中樞」。負責處理跨容器通訊、API 路由轉發。內建 Watchdog 守護進程，嚴密監控合約資產水位與各服務的健康度，具備毫秒級的斷路器 (Circuit Breaker) 與緊急平倉功能。
* **Quant Brain (Python):** 擔任系統的「大腦」。專注於計算圖論最佳路徑、動態利率模型與 DEX 滑點預估，並將運算出的最佳參數下發給 Go 中樞。

### 2. 鏈上智能合約 (On-chain Contracts - Solidity)
* 負責資金接收與核心 DeFi 邏輯執行，確保交易的原子性 (Atomicity)。
* 包含 Aave 閃電貸回調介面 (`executeOperation`) 以及與第三方 DEX 的路由交互。合約設計嚴格限制調用權限 (OnlyOwner)，防止資金池遭駭客惡意抽乾。

## 部署指南 (Choreo.dev)

本系統支援部署至雲端原生 PaaS 平台 [Choreo.dev](https://choreo.dev/)。

### 前置作業
1. 確保已連接 GitHub 儲存庫至 Choreo 專案。
2. 準備好各個區塊鏈網路的 RPC 節點端點 (推薦使用 Alchemy 或 Infura)。
3. 準備包含原生 Gas 代幣與測試用穩定幣的 EVM 錢包 (如 Phantom)。

### 環境變數設定
請在 Choreo 的 Environment Variables 區塊中配置以下變數：
* `RPC_WSS_URL`: 區塊鏈 WebSocket 端點 (如 Base 或 Polygon)。
* `PRIVATE_KEY`: 系統操作者錢包私鑰 (請妥善保管，勿提交至版本控制)。
* `CONTRACT_ADDRESS`: 部署完成的自定義 Solidity 合約地址。
* `AAVE_POOL_ADDRESS`: Aave V3 Pool 合約地址。

### 啟動順序
為確保依賴關係正確，建議依以下順序啟動微服務組件：
1. **Quant Python:** 啟動量化模型與 API 端點。
2. **Gateway Go:** 啟動 Watchdog 守護進程，開始檢查系統依賴。
3. **Executor Rust:** 最後啟動，建立 RPC 連線並開始監聽區塊鏈狀態。

## 本地開發與測試

使用 Foundry 進行智能合約的本地分叉測試 (Fork Testing)，並透過 Docker Compose 啟動本地微服務集群。

```bash
# 1. 編譯智能合約
cd contracts
forge build

# 2. 執行主網分叉測試 (以 Base 網路為例)
forge test --rpc-url <YOUR_BASE_RPC_URL> -vvv

# 3. 啟動本地微服務集群
cd ..
docker-compose up --build


## 參考資料與技術文檔
### Aave V3 Developers Documentation

### Foundry Book - Smart Contract Development Framework

### Alloy / Ethers-rs (Rust Ethereum ecosystem)

### Choreo.dev Documentation
