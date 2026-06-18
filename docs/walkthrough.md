# Walkthrough: Investment Portfolio Manager

We have successfully built and verified the full-stack **Investment Portfolio Manager**. The application is structured with a separate Python (FastAPI) backend and a React (Vite + Vanilla CSS) frontend.

---

## 🛠️ What Was Built

### 1. Backend Service (FastAPI)
- **Database Schema**: Utilizes SQLite via SQLAlchemy. The database caches daily price bars (`HistoricalPrice` table) for offline capabilities and rapid computations.
- **Tax Lot Accounting Engine (`tax_engine.py`)**:
  - Implements **FIFO** (First-In, First-Out) matching.
  - Implements **LIFO** (Last-In, First-Out) matching.
  - Implements a **Hybrid** method (applies LIFO matching on lots bought within a configurable window, e.g., 30 days, to shield short-term lots, falling back to FIFO for older lots).
  - Returns realized gains/losses, remaining open tax lots, and cost basis.
- **Advanced Metrics Engine (`stats_engine.py`)**:
  - Computes historical daily portfolio values.
  - Calculates daily returns and cumulative **Time-Weighted Return (TWR)** to accurately track performance independently of capital additions/withdrawals.
  - Computes **Annualized Volatility** and **Sharpe Ratio** (using a 2% risk-free rate).
  - Computes **Portfolio Beta** relative to the **SPY** S&P 500 index.
  - Computes **Beta-Adjusted Net Exposure** (total exposure scaled by portfolio beta).
  - Generates the asset **Correlation Matrix** based on daily returns log covariance.

### 2. Frontend Web App (React + Vite)
- **Design System (`index.css`)**: Premium slate glassmorphic dark theme styled with custom CSS variables, responsive layouts, Google Font 'Outfit', custom badges, and interactive animations.
- **Dashboard View**: Displays Net Worth, Realized Tax P&L, Latent P&L, and Total Cumulative Return. Displays a historical value timeline (Recharts Area Chart) and asset type allocation (Recharts Pie Chart).
- **Holdings & Tax Lots View**: Detail view listing current holdings, average cost, market value, realized and unrealized gains. Allows users to expand any asset to inspect individual open tax lots (dates, unit costs, and latent ROI). Supports registering new asset tickers and logging buy/sell orders.
- **Risk & Analytics View**: Displays advanced risk metrics (Volatility, Sharpe, Beta, Beta-Adjusted Net Exposure) and a color-scaled **Correlation Heatmap** grid.
- **Transaction Ledger View**: Chronological record of all transaction activities with details on quantity, execution price, broker fees, and net trade values.

---

## 🧪 Verification & Validation Results

### 1. Automated Unit Tests
We verified the tax lot engine algorithms using pytest:
`PYTHONPATH=backend pytest backend/tests/test_tax_engine.py`

- **FIFO Matching**: Validated correct lot division, fractional share parsing, unit fee inclusions, and realized gains.
- **LIFO Matching**: Verified newest shares match first.
- **Hybrid Matching**: Verified LIFO matches on assets with age $\le 30$ days and FIFO applies to older assets.

**Test Run Outcome**:
```bash
backend/tests/test_tax_engine.py ...                                     [100%]
============================== 3 passed in 0.05s ===============================
```

### 2. End-to-End API Integration Verification
We executed an integration verification script ([verify_app.py](file:///Users/javier/.gemini/antigravity/brain/fb89866f-f765-45d0-98a1-2b68c71256dc/scratch/verify_app.py)) that creates a portfolio, logs transactions, pulls live prices, and checks metrics.

**Math Verification Match**:
- **Transactions**:
  - BUY 10 AAPL at $170.00, fee = $5.00. (Unit Cost = $170.50).
  - SELL 3 AAPL at $190.00, fee = $2.00. (Net Proceeds = $568.00).
- **FIFO Profit Math**:
  - Matched 3 AAPL against the first lot.
  - Cost Basis = $3 \times 170.50 = 511.50$.
  - Realized Profit = $568.00 - 511.50 = 56.50$.
- **Verification Output**:
  - AAPL current price dynamically fetched from Yahoo Finance.
  - Total Portfolio Value: **$33,378.52**
  - Total Realized P&L: **$56.50** (100% match with math).
  - Portfolio Beta: **0.75** (Defensive asset posture relative to S&P 500).
  - Portfolio Volatility: **26.05%**.
  - Sharpe Ratio: **3.38**.

---

## 🚀 How to Run the Application

To run both backend and frontend servers concurrently, a unified startup script `start.sh` has been created in the project root folder.

### Run in Local Terminal
1. Open a new terminal on your Mac.
2. Navigate to your project folder:
   ```bash
   cd /Users/javier/code/investment-portfolios
   ```
3. Run the startup script:
   ```bash
   ./start.sh
   ```

Both servers will start up concurrently:
- **Web App**: Accessible at [http://localhost:5173](http://localhost:5173)
- **API Documentation**: Accessible at [http://127.0.0.1:8000/docs](http://127.0.0.1:8000/docs)

To stop both servers, simply press `CTRL+C` in your terminal.

> [!TIP]
> **macOS Localhost Fix**: We explicitly configured the React frontend to communicate with `http://127.0.0.1:8000` instead of `localhost:8000`. This prevents connection drops on macOS caused by `localhost` resolving to IPv6 `::1` while the local FastAPI server binds to IPv4.
