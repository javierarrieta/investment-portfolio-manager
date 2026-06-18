# Multi-Currency Portfolio Tracking Implementation Plan

We will extend the Investment Portfolio Manager to support holding assets in different currencies (e.g., USD, EUR, GBP) within a single portfolio, and reporting performance and tax lots in the portfolio's base currency.

---

## User Review Required

Please verify the proposed currency conversion approach:
> [!IMPORTANT]
> **Tax Conversion Rules**: Under standard tax rules, capital gains on foreign assets must be calculated by converting both the purchase cost and the sales proceeds into the local reporting currency (portfolio base currency) using the exchange rate **on the actual transaction date**. 
> - We will fetch daily historical exchange rates from Yahoo Finance (e.g., `EURUSD=X` or `USDEUR=X`) for transaction dates.
> - The FIFO/LIFO matching engine will perform calculations in the portfolio's base currency.
> Please let us know if you prefer a different conversion logic.

---

## Open Questions

1. **Automatic Schema Migration**: Since we are adding a `currency` field to the `Asset` database model, we will write a self-healing script on backend startup that runs `ALTER TABLE assets ADD COLUMN currency VARCHAR DEFAULT 'USD'` to preserve any portfolios and transactions you have already logged. Is this automatic schema upgrade acceptable?

---

## Proposed Changes

### 1. Backend Extensions

#### [MODIFY] [models.py](file:///Users/javier/code/investment-portfolios/backend/app/models.py)
- Add `currency` column to the `Asset` model (String, defaults to `"USD"`).

#### [MODIFY] [main.py](file:///Users/javier/code/investment-portfolios/backend/app/main.py)
- Import `text` from SQLAlchemy.
- Run a migration check on startup: if `currency` column is missing in `assets`, execute `ALTER TABLE assets ADD COLUMN currency VARCHAR DEFAULT 'USD'` to upgrade the SQLite schema without wiping data.

#### [MODIFY] [routes/analytics.py](file:///Users/javier/code/investment-portfolios/backend/app/routes/analytics.py)
- Fix bug: Import `pandas as pd` globally at the top of the file to resolve the `'pd' is not defined` error when fetching ticker price fallback data.
- Integrate exchange rate lookups: When calculating tax lots and metrics, check if `asset.currency != portfolio.currency`. If so, fetch the corresponding exchange rate (e.g., `EURUSD=X` or `USDEUR=X`) from Yahoo Finance for the required dates.

#### [MODIFY] [tax_engine.py](file:///Users/javier/code/investment-portfolios/backend/app/tax_engine.py)
- Accept exchange rate conversion tables.
- Convert transaction prices to the portfolio base currency during matching to compute realized tax gains/losses.

#### [MODIFY] [stats_engine.py](file:///Users/javier/code/investment-portfolios/backend/app/stats_engine.py)
- Sync historical daily prices for exchange rates (e.g., `USDEUR=X`) alongside asset tickers.
- Convert daily asset market values to the portfolio base currency before summing the daily portfolio net value.

---

### 2. Frontend Extensions

#### [MODIFY] [App.jsx](file:///Users/javier/code/investment-portfolios/frontend/src/App.jsx)
- Fix bug: Import `<Trash2>` icon from `lucide-react` at the top of the file to resolve the reference crash in the ledger view.
- Update the "Register Asset" modal payload to include a selected `currency` dropdown (USD, EUR, GBP, BTC).

#### [MODIFY] [PortfolioDetail.jsx](file:///Users/javier/code/investment-portfolios/frontend/src/components/PortfolioDetail.jsx)
- Update "Add Asset" modal form to include a currency selector dropdown.
- Display the currency symbol alongside prices and holdings (e.g., `120.00 EUR` or `$150.00`).

---

## Verification Plan

### Automated Tests
- Extend the unit tests in `backend/tests/test_tax_engine.py` to mock multi-currency transactions (e.g., buying in EUR, selling in EUR, inside a USD base portfolio) and verify the converted cost basis and realized profit math.

### Manual Verification
1. Open the UI, create a new portfolio in **EUR** base currency.
2. Add a USD asset (e.g., `AAPL` in USD) and a EUR asset (e.g., `BBVA.MC` in EUR).
3. Log buy/sell transactions.
4. Verify that the asset values are converted correctly using real exchange rates on the history chart and holdings breakdown.
