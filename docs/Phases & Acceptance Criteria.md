# 專案開發階段與驗收標準 (Phases & Acceptance Criteria)

本專案分為五個核心階段，從本地端合約測試到最終的主網高頻調優。每個階段必須嚴格通過驗收標準 (Acceptance Criteria, AC) 才能進入下一階段，以最大程度降低智能合約與基礎設施帶來的資金風險。

## Phase 1: 鏈上合約與本地分叉環境 (Smart Contracts & Local Fork)
**目標：** 完成核心 Solidity 合約的撰寫，並在本地端模擬真實區塊鏈狀態進行邏輯驗證。

* **執行項目：**
  1. 撰寫 `FlashLoanArb.sol` 或 `Liquidator.sol`。
  2. 整合 Aave V3 的 `IPool` 與閃電貸回調介面。
  3. 使用 Foundry 建立 Base 或 Polygon 網路的 Local Fork 測試環境。
* **驗收標準 (AC)：**
  * [ ] 合約編譯無警告 (0 Warnings, 0 Errors)。
  * [ ] 成功在 Foundry Fork 環境中觸發 Aave 閃電貸並完成還款。
  * [ ] 測試涵蓋率 (Test Coverage) 針對核心業務邏輯大於 90%。
  * [ ] 刻意構造的惡意參數交易能被合約正確 Revert（例如權限錯誤或利潤小於零）。

## Phase 2: 核心執行緒與系統看門狗 (Core Engine & Watchdog)
**目標：** 建立 Rust 鏈上執行緒的底層連線能力，以及 Go 語言網關的系統監控機制。這部分的穩定性是防禦惡意交易與避免資金耗損的底線。

* **執行項目：**
  1. **Rust (Executor):** 使用 `ethers-rs` 或 `alloy` 實作 WebSocket 客戶端，直接監聽區塊鏈的 Pending Transactions (Mempool)。
  2. **Go (Gateway):** 實作 API Router，並建立一個獨立的 Watchdog 背景執行緒，持續輪詢各服務的心跳 (Heartbeat) 與 RPC 節點延遲。
* **驗收標準 (AC)：**
  * [ ] Rust 引擎能穩定維持 24 小時的 WSS 長連線，斷線後能在 3 秒內自動重連。
  * [ ] Rust 能成功解析 Mempool 中的特定目標交易，並在 50 毫秒內輸出日誌。
  * [ ] Go Watchdog 在偵測到模擬的「RPC 延遲飆高」或「保證金水位低於閾值」時，能在一秒內成功觸發「斷路器 (Circuit Breaker)」事件並暫停後續交易請求。

## Phase 3: 量化大腦與系統串接 (Quant Brain & End-to-End Integration)
**目標：** 引入 Python 模型進行動態計算，並透過 Docker Compose 在本地端完成三語言微服務與智能合約的全鏈路整合。

* **執行項目：**
  1. **Python (Quant):** 開發 FastAPI 端點，接收即時價格並計算最佳 LTV 或滑點參數。
  2. **Integration:** 定義 gRPC 或 RESTful 內部通訊格式，將 Python 的計算結果傳遞給 Go，再轉交 Rust 構造區塊鏈交易。
  3. 配置 `docker-compose.yml` 啟動所有容器。
* **驗收標準 (AC)：**
  * [ ] Python 模型能在 100 毫秒內回傳正確的定價與利潤預估結果。
  * [ ] 全鏈路測試通過：由 Python 發起訊號 -> Go 驗證與紀錄 -> Rust 簽名並發送交易至 Foundry Local Node -> 交易成功上鏈。

## Phase 4: 雲端部署與測試網驗證 (Cloud Deployment & Testnet PoC)
**目標：** 將系統部署至 Choreo.dev，並對接真實的測試網 (如 Sepolia 或 Base Sepolia)，驗證雲端網路延遲與 PaaS 環境變數配置。

* **執行項目：**
  1. 撰寫與配置 Choreo.dev 的 `component.yaml` 與 `endpoints.yaml`。
  2. 將智能合約部署至測試網。
  3. 啟動系統，準備測試網原生代幣進行實彈演練。
* **驗收標準 (AC)：**
  * [ ] 三個微服務容器在 Choreo.dev 上成功 Build 且狀態為 Running。
  * [ ] 微服務能正確讀取雲端環境注入的 `PRIVATE_KEY` 與 `RPC_WSS_URL`。
  * [ ] 系統能在測試網真實環境中，成功捕捉到觸發條件並自動完成至少一筆套利或清算交易，且 Gas 估算無誤。

## Phase 5: 主網上線與極限調優 (Mainnet Execution & Tuning)
**目標：** 轉移至低手續費的 Layer 2 主網 (Base / Polygon) 進行真實資金運作，並著重於 Gas 競價演算法與基礎設施調優。

* **執行項目：**
  1. 合約部署至 L2 主網，並存入初期驗證資金 (如 $100 - $500 USD)。
  2. Rust 引擎引入動態 Gas 競價邏輯 (EIP-1559 最佳化)。
  3. 系統日誌與盈虧 (P&L) 監控面板建立。
* **驗收標準 (AC)：**
  * [ ] Watchdog 在主網環境下穩定運行，未出現誤判或死鎖 (Deadlock)。
  * [ ] 系統成功在主網完成首筆獲利交易，且實際 Gas 消耗與 Python 模組的預期誤差小於 5%。
  * [ ] 穩定運行一週，統計勝率與資金耗損率，確立後續擴大資金池的參數配置。

---
### 資料來源與參考準則：

軟體工程實務指南：微服務架構的斷路器模式 (Circuit Breaker Pattern in Microservices) - Microsoft Architecture Docs

Foundry 官方技術文檔：本地分叉與智能合約測試標準 (Foundry Book - Fork Testing)

雲端原生運算基金會 (CNCF)：容器化服務的健康探針與 Watchdog 實作規範
