# Rust Backend Test Suite Design

**Date:** 2026-06-28

## Summary

Create a comprehensive test suite for the Rust (Rocket) backend at `backend_rust/`. Currently zero tests exist — no `#[test]` annotations, no `tests/` directory. This design covers unit tests (inline `mod tests`) and integration tests (separate `tests/` directory) using in-memory SQLite.

## Architecture

**Approach:** Idiomatic Rust — inline `mod tests` for unit tests, separate `tests/` directory for integration tests, zero new dependencies.

```
backend_rust/src/
  engines/tax_engine.rs       ← inline mod tests (FIFO, LIFO, HYBRID, edge cases)
  engines/stats_engine.rs     ← inline mod tests (empty inputs, TWR math)
  schemas.rs                  ← inline mod tests (serialization roundtrips)
  models.rs                   ← inline mod tests (struct field mapping)
  services/currency_service.rs← inline mod tests (cache, same-currency, no HTTP)
  api_routes/portfolios.rs    ← inline mod tests (CRUD logic)
  api_routes/transactions.rs  ← inline mod tests (asset/tx CRUD)

backend_rust/tests/
  common/mod.rs               ← shared test helpers (setup_db, seed data)
  integration/
    portfolios_test.rs        ← portfolio CRUD via Rocket TestClient
    transactions_test.rs      ← asset/transaction CRUD via Rocket TestClient
    analytics_test.rs         ← tax-summary & performance endpoints
```

- **Unit tests**: inline `mod tests` inside each source file, `#[cfg(test)]` gated
- **Integration tests**: separate `tests/` directory, in-memory SQLite via `SqlitePool::connect(":memory:")`
- **CurrencyService mocking**: unit tests use a test double (no HTTP calls); integration tests use a real instance but in-memory DB has no historical prices so stats routes return the empty-case JSON
- **Dependencies**: zero new dependencies — uses only what is already in `Cargo.toml`

## Unit Test Coverage

### TaxLotEngine (highest ROI — pure business logic, no I/O)
- **FIFO**: buy 3 lots at different dates/prices, sell partial → matches oldest lot first
- **LIFO**: buy 3 lots, sell partial → matches newest lot first
- **HYBRID short-term first**: buy lots spanning hybrid threshold → short-term lots matched first
- **HYBRID all long-term**: all buy lots beyond threshold → matches oldest first
- **SELL exceeds total buys**: sells all lots to zero quantity, no crash
- **No prior buys for a sell**: sell is ignored (no eligible lots)
- **Fee in cost basis**: buy with fee has higher unit cost than buy without fee
- **Multi-currency conversion**: EUR buy in USD-based portfolio → prices converted at transaction date rate
- **Empty transactions**: returns zero shares, zero values
- **Single buy, no sells**: one open lot, correct cost basis and market value
- **Partial sell across multiple lots**: sells from first lot, remainder from second

### StatsEngine
- **Empty assets/transactions**: returns zeroed metrics JSON (`volatility: 0`, `sharpe_ratio: 0`, `beta: 1.0`)
- **No historical prices**: with assets but no price data in DB, daily values resolve to 0

### CurrencyService
- **Same currency** (USD→USD): returns 1.0 without calling client
- **Cache hit**: returns cached rate, does not call client
- **Cache miss**: calls client, populates cache, returns rate

### Schemas / Models
- **Serialize/deserialize roundtrip**: each schema struct serializes and deserializes back to identical value
- **Optional fields**: `PortfolioCreate` with `None` description serializes without the field

## Integration Test Coverage

### portfolios_test.rs (Rocket TestClient, in-memory SQLite)
- Create portfolio → 201, correct JSON body
- List portfolios → 200, returns created portfolio
- Get portfolio by id → 200
- Get nonexistent portfolio → 404
- Delete portfolio → 204
- Delete nonexistent portfolio → 404

### transactions_test.rs
- Create asset in portfolio → 201
- Create duplicate asset → 400
- Create transaction for asset → 201
- Create transaction for nonexistent asset → 404
- List transactions for portfolio → 200, returns all transactions sorted by date desc
- Delete asset → 204
- Delete nonexistent asset → 404
- Delete transaction → 204
- Delete nonexistent transaction → 404

### analytics_test.rs
- Tax summary for empty portfolio → 200, `assets: []`
- Tax summary with assets and transactions (FIFO) → 200, correct summary structure
- Tax summary with assets and transactions (LIFO) → 200, correct summary structure
- Performance endpoint → 200, returns metrics JSON

## Shared Test Helpers (`tests/common/mod.rs`)

```rust
// Creates in-memory SQLite pool with all required tables
pub async fn setup_db() -> SqlitePool

// Inserts a portfolio with default values
pub async fn seed_portfolio(pool: &SqlitePool, id: i32, name: &str, currency: &str)

// Inserts an asset in a portfolio
pub async fn seed_asset(pool: &SqlitePool, portfolio_id: i32, symbol: &str, name: &str)

// Inserts a transaction for an asset
pub async fn seed_transaction(pool: &SqlitePool, asset_id: i32, tx_type: &str, qty: f64, price: f64, fee: f64, date: DateTime<Utc>)
```

## Error Handling Approach

- DB setup failures → `panic!` in test (tests should be deterministic, not flaky)
- Rocket handlers → assert on `Status` codes (201, 204, 400, 404, 500)
- JSON responses → parse with `serde_json` and assert on key fields
- `CurrencyService` in unit tests → use a mock struct implementing the same interface, no HTTP calls

## CI

`cargo test` runs both unit and integration tests automatically. No extra configuration needed. Integration tests use in-memory SQLite so they are fast and don't touch the persistent `portfolio.db`.

## What This Does NOT Cover (Future)

- `CurrencyService` integration tests that actually hit Yahoo Finance (would need `mockito` or similar)
- `StatsEngine` price matrix tests against real historical data
- OpenAPI spec validation tests
- Performance/benchmark tests

These can be added later as the codebase matures.
