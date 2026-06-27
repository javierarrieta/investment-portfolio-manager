# Migration Plan: Python Backend to Rust

This document outlines the plan to migrate the Investment Portfolio Manager backend from Python (FastAPI) to Rust (Rocket).

## 1. Technical Stack Mapping

| Component | Python (Current) | Rust (Target) | Note |
| :--- | :--- | :--- | :--- |
| **Web Framework** | FastAPI | **Rocket** | Requested framework |
| **Database ORM/Driver** | SQLAlchemy | **SQLx** | Asynchronous SQLite support |
| **Data Validation** | Pydantic | **Serde** | Standard for JSON serialization |
| **Numerical Ops** | NumPy / Pandas | **Polars** | High-performance DataFrame library |
| **External API** | `yfinance` | **reqwest** + Custom Parser | Direct calls to Yahoo Finance API |
| **Date/Time** | `datetime` | **chrono** | Standard for date/time handling |
| **Testing** | `pytest` | **cargo test** | Integrated Rust testing framework |

## 2. Implementation Phases

### Phase 1: Foundation & Data Model
- [ ] **Project Initialization**: Create a new Rust project structure within the workspace.
- [ ] **Dependency Configuration**: Add `rocket`, `sqlx`, `serde`, `polars`, `reqwest`, and `chrono` to `Cargo.toml`.
- [ ] **Schema Definition**: Implement Rust structs that mirror `models.py` and `schemas.py`.
- [ ] **DB Connection**: Setup SQLx connection pool for `backend/portfolio.db`.

### Phase 2: Core Logic Porting
- [ ] **`CurrencyService`**: Port the exchange rate fetching and caching logic using `reqwest`.
- [ ] **`TaxLotEngine`**: Port the FIFO, LIFO, and Hybrid tax lot matching algorithms.
- [ ] **`StatsEngine`**: Port the TWR, Volatility, Beta, and Sharpe Ratio calculations using `Polars`.

### Phase 3: API Route Implementation
- [ ] **Portfolio Routes**: Port CRUD endpoints for portfolios.
- [ ] **Transaction/Asset Routes**: Port endpoints for managing assets and logging transactions.
- [ ] **Analytics Routes**: Implement `/tax-summary` and `/performance` endpoints, integrating the ported engines.

### Phase 4: Parity & Verification
- [ ] **Test Suite Porting**: Translate Python tests in `backend/tests/` to Rust `#[test]` functions.
- [ ] **OpenAPI Validation**: 
    - Generate the OpenAPI specification for the Rocket implementation.
    - Compare the Rust spec against the Python spec to ensure 100% endpoint and schema parity.
- [ ] **Integration Testing**: Verify frontend compatibility with the new Rust backend.

## 3. Critical Technical Challenges
- **yfinance Replacement**: Rust lacks a direct `yfinance` port. A custom client will be built to fetch data from Yahoo Finance's JSON API.
- **Pandas $\rightarrow$ Polars**: Mapping complex Pandas operations (like `pivot`, `ffill`, `pct_change`) to Polars expressions.
- **Async Synchronization**: Ensuring non-blocking I/O for both database access and external API calls using Rocket's async handlers.
