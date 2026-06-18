# Multi-Currency Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable multi-currency support by adding currency fields to assets/portfolios and a central `CurrencyService` for conversions.

**Architecture:** A new `CurrencyService` handles exchange rate fetching and caching. Core engines (`TaxLotEngine`, `StatsEngine`) are updated to use this service to perform all calculations in a portfolio's specified `base_currency`.

**Tech Stack:** Python (FastAPI, SQLAlchemy), React (Vite), Yahoo Finance API.

---

### Task 1: Backend Models & Migration

**Files:**
- Modify: `backend/app/models.py`
- Modify: `backend/app/main.py`

- [ ] **Step 1: Add `base_currency` to `Portfolio` and `currency` to `Asset` in `models.py`**

```python
# backend/app/models.py (conceptual)
class Portfolio(Base):
    # ... existing fields
    base_currency = Column(String, default="USD")

class Asset(Base):
    # ... existing fields
    currency = Column(String, default="USD")
```

- [ ] **Step 2: Implement automatic migration in `main.py`**

```python
# backend/app/main.py (conceptual)
from sqlalchemy import text

@app.on_event("startup")
def run_migrations():
    with engine.connect() as conn:
        # Migrate Portfolios
        try:
            conn.execute(text("ALTER TABLE portfolios ADD COLUMN base_currency VARCHAR DEFAULT 'USD'"))
            conn.commit()
        except Exception: # Column exists
            pass
        # Migrate Assets
        try:
            conn.execute(text("ALTER TABLE assets ADD COLUMN currency VARCHAR DEFAULT 'USD'"))
            conn.commit()
        except Exception: # Column exists
            pass
```

- [ ] **Step 3: Run and verify migration**
  Run the backend and check `backend/portfolio.db` to ensure columns exist.

- [ ] **Step 4: Commit**

```bash
git add backend/app/models.py backend/app/main.py
git commit -m "feat: add currency fields and migration logic"
```

### Task 2: Currency Service

**Files:**
- Create: `backend/app/services/currency_service.py`
- Test: `backend/tests/test_currency_service.py`

- [ ] **Step 1: Write the failing test for `CurrencyService`**

```python
# backend/tests/test_currency_service.py
import pytest
from app.services.currency_service import CurrencyService

@pytest.mark.asyncio
async def test_get_rate_success():
    service = CurrencyService()
    # Mocking the Yahoo Finance fetch in the implementation
    rate = await service.get_rate("EUR", "USD", datetime(2026, 1, 1))
    assert rate > 0
```

- [ ] **Step 2: Run test to verify it fails**
Run: `PYTHONPATH=backend pytest backend/tests/test_currency_service.py -v`
Expected: FAIL (ModuleNotFoundError or Import error)

- [ ] **Step 3: Implement `CurrencyService` with caching and Yahoo Finance integration**

```python
# backend/app/services/currency_service.py
import httpx
from datetime import datetime

class CurrencyService:
    def __init__(self):
        self.cache = {}

    async def get_rate(self, from_curr: str, to_curr: str, date: datetime) -> float:
        # 1. Check cache
        # 2. If not in cache, fetch from Yahoo Finance (e.g. yfinance or direct API)
        # 3. Update cache and return
        pass
```

- [ ] **Step 4: Run test to verify it passes**
Run: `PYTHONPATH=backend pytest backend/tests/test_currency_service.py -v`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add backend/app/services/currency_service.py backend/tests/test_currency_service.py
git commit -m "feat: implement CurrencyService with caching"
```

### Task 3: Tax Engine Integration

**Files:**
- Modify: `backend/app/tax_engine.py`
- Test: `backend/tests/test_tax_engine.py`

- [ ] **Step 1: Update `TaxLotEngine.calculate_lots` signature and logic**

```python
# backend/app/tax_engine.py
class TaxLotEngine:
    @staticmethod
    async def calculate_lots(
        symbol: str,
        asset_type: str,
        transactions: list,
        current_price: float,
        strategy: str,
        base_currency: str,          # New
        currency_service: Any,       # New
        hybrid_threshold_days: int = 30
    ):
        # Logic to convert transaction prices to base_currency using currency_service
        pass
```

- [ ] **Step 2: Update existing tests in `test_tax_engine.py` to match new signature**

- [ ] **Step 3: Write new multi-currency test case**

```python
# backend/tests/test_tax_engine.py
@pytest.mark.asyncio
async def test_multi_currency_fifo():
    # Buy 10 AAPL (USD) at 100, Sell 5 AAPL (USD) at 150. Portfolio is EUR.
    # Mock CurrencyService to return EURUSD = 0.9
    # Expected realized P&L in EUR = (5 * 150 * 0.9) - (5 * 100 * 0.9) = 225 EUR
    pass
```

- [ ] **Step 4: Run tests and verify pass**

- [ ] **Step 5: Commit**

```bash
git add backend/app/tax_engine.py backend/tests/test_tax_engine.py
git commit -m "feat: integrate CurrencyService into TaxLotEngine"
```

### Task 4: Stats Engine Integration

**Files:**
- Modify: `backend/app/stats_engine.py`
- Test: `backend/tests/test_stats_engine.py`

- [ ] **Step 1: Update `StatsEngine` to handle base currency conversion for daily values**

- [ ] **Step 2: Write/Update tests to verify metrics (TWR, Volatility) are calculated in base currency**

- [ ] **Step 3: Commit**

### Task 5: Frontend UI Updates

**Files:**
- Modify: `frontend/src/components/PortfolioDetail.jsx`
- Modify: `frontend/src/App.jsx`

- [ ] **Step 1: Add currency selector to "Add Asset" modal in `PortfolioDetail.jsx`**

- [ ] **Step 2: Update asset registration payload in `App.jsx` to include the selected currency**

- [ ] **Step 3: Update UI to display currency symbols (e.g., "$", "€")**

- [ ] **Step 4: Commit**

### Task 6: Documentation Cleanup

**Files:**
- Modify: `docs/implementation_plan.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: Update documentation to reflect the completed architecture**

- [ ] **Step 2: Commit**
