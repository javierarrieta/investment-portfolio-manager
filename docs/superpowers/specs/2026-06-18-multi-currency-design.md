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
- **Modification**: Instead of just processing raw transaction amounts, the engine will use the `CurrencyService` to convert both purchase cost and sales proceeds into the portfolio's base currency using the exchange rate effective on the transaction date.
- **Benefit**: Ensures capital gains/losses are calculated accurately according to standard tax rules (converting to local currency at the time of transaction).

#### `StatsEngine`
- **Modification**: When computing daily portfolio values for TWR, Volatility, and Sharpe Ratio, the engine will use the `CurrencyService` to convert each asset's market value (in its own currency) into the portfolio's base currency for that specific date.
- **Benefit**: Accurate historical performance tracking despite currency fluctuations.

### 3. Database Schema Changes
- **`Asset` Model**: Add a `currency` column (String, e.g., "USD", "EUR"). Defaults to `"USD"`.
- **Migration Strategy**: A self-healing script will run on backend startup. If the `currency` column is missing in the `assets` table, it will execute:
  `ALTER TABLE assets ADD COLUMN currency VARCHAR DEFAULT 'USD'`
  This ensures zero data loss for existing portfolios.

## Data Flow Example
1. **Setup**: Portfolio base currency is **USD**.
2. **Asset Creation**: User adds `SAP.DE` with currency **EUR**.
3. **Transaction**: User logs a BUY of 10 units at `150.00 EUR` on `2026-01-01`.
4. **Conversion**:
   - `CurrencyService` provides `EURUSD` rate for `2026-01-01` (e.g., `1.10`).
   - Effective cost in USD: `10 * 150.00 * 1.10 = 1650.00 USD`.
5. **Reporting**: All P&L and holdings are displayed in **USD**.

## Testing Strategy
- **Unit Tests**: Mock the `CurrencyService` to provide deterministic exchange rates. Verify that `TaxLotEngine` correctly computes realized/unrealized P&L using these converted rates.
- **Integration Tests**: Verify the automatic schema migration works correctly on a fresh database.

## Documentation Updates
- Update `AGENTS.md` to reflect the new architecture and components.
- Update `docs/implementation_plan.md` to reflect the switch to a `CurrencyService` architecture.
