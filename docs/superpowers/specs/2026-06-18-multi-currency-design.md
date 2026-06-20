# Multi-Currency Support Design

**Date**: 2026-06-18
**Status**: Proposed

## Overview
Extend the Investment Portfolio Manager to support assets held in different currencies (e.g., USD, EUR, GBP) while reporting all performance metrics and tax calculations in a single portfolio base currency.

## Architecture

### 1. New Component: `CurrencyService`
A central service in the backend responsible for:
- **Exchange Rate Retrieval**: Fetching historical exchange rates from Yahoo Finance (e.g., `EURUSD=X`).
- **Caching**: Storing fetched rates in memory or a local cache to minimize API calls and improve performance.
- **Interface**: Providing a method `get_rate(from_currency: str, to_currency: str, date: datetime) -> float`.

### 2. Updated Core Engines

#### `TaxLotEngine`
- **Modification**: Instead of just processing raw transaction amounts, the engine will use the `CurrencyService` to convert both purchase cost and sales proceeds into the portfolio's `base_currency` using the exchange rate effective on the transaction date.
- **Benefit**: Ensures capital gains/losses are calculated accurately according to standard tax rules (converting to local currency at the time of transaction).

#### `StatsEngine`
- **Modification**: When computing daily portfolio values for TWR, Volatility, and Sharpe Ratio, the engine will use the `CurrencyService` to convert each asset's market value (in its own currency) into the portfolio's `base_currency` for that specific date.
- **Benefit**: Accurate historical performance tracking despite currency fluctuations.

### 3. Database Schema Changes
- **`Portfolio` Model**: Add a `base_currency` column (String, e.g., "USD", "EUR").
- **`Asset` Model**: Add a `currency` column (String, e.g., "USD", "EUR"). Defaults to `"USD"`.
- **Migration Strategy**: A self-healing script will run on backend startup. It will check for the existence of `base_currency` in `portfolios` and `currency` in `assets`, executing `ALTER TABLE` commands if missing:
  - `ALTER TABLE portfolios ADD COLUMN base_currency VARCHAR DEFAULT 'USD'`
  - `ALTER TABLE assets ADD COLUMN currency VARCHAR DEFAULT 'USD'`
  This ensures zero data loss for existing portfolios.

## Data Flow Example
1. **Setup**: User creates a portfolio with `base_currency` set to **EUR**.
2. **Asset Creation**: User adds `AAPL` (an asset with currency **USD**).
3. **Transaction**: User logs a BUY of 10 units at `$150.00 USD` on `2026-01-01`.
4. **Conversion**:
   - `CurrencyService` provides `USDEUR` rate for `2026-01-01` (e.g., `0.90`).
   - Effective cost in portfolio base currency (EUR): `10 * 150.00 * 0.90 = 1350.00 EUR`.
5. **Reporting**: All P&L and holdings are displayed in **EUR**.

## Testing Strategy
- **Unit Tests**: Mock the `CurrencyService` to provide deterministic exchange rates. Verify that `TaxLotEngine` correctly computes realized/unrealized P&L using these converted rates.
- **Integration Tests**: Verify the automatic schema migration works correctly on a fresh database.

## Documentation Updates
- Update `AGENTS.md` to reflect the new architecture and components.
- Update `docs/implementation_plan.md` to reflect the switch to a `CurrencyService` architecture.
