# f64 → Decimal Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all `f64` types in the Rust backend data model, API schemas, and engine calculations with `rust_decimal::Decimal` to eliminate floating-point precision errors in financial calculations, especially critical for multi-currency support.

**Architecture:** Add `rust_decimal` as a dependency, then systematically replace `f64` with `Decimal` across models, schemas, engines, and services. SQLite stores decimals as TEXT (string representation). Yahoo Finance API responses are parsed into Decimal immediately after fetch. `serde_json::Value`-based aggregation in stats_engine is the last tier to change and may retain f64 with a documented rationale.

**Tech Stack:** `rust_decimal` crate (with `serde` feature), SQLx custom `Type`/`Decode`/`Encode` impls for SQLite TEXT storage, `utoipa` for OpenAPI schema generation.

## Global Constraints

- All monetary values must use `Decimal`, never `f64`, in data model and API layers
- SQLite stores decimal values as TEXT to preserve exact representation
- Yahoo Finance API returns JSON numbers (f64) — parse into Decimal immediately at the service boundary
- `serde_json::Value` in stats_engine uses `f64` natively; this is acceptable for statistical aggregation where precision loss is negligible
- All existing tests must pass after migration
- The `rust_decimal` crate version must be compatible with the existing `serde` and `sqlx` versions

---

### Task 1: Add `rust_decimal` dependency and configure features

**Files:**
- Modify: `backend_rust/Cargo.toml`

**Interfaces:**
- Adds `rust_decimal` crate with `serde` feature enabled

- [ ] **Step 1: Add rust_decimal to Cargo.toml**
Add `rust_decimal = { version = "1.35", features = ["serde"] }` to `[dependencies]` in `backend_rust/Cargo.toml`

- [ ] **Step 2: Verify compilation**
Run: `cd backend_rust && cargo check`
Expected: Compiles with new dependency (no code changes yet)

- [ ] **Step 3: Commit**
```bash
git add backend_rust/Cargo.toml backend_rust/Cargo.lock
git commit -m "feat: add rust_decimal dependency for precision financial calculations"
```

---

### Task 2: Replace f64 in data model entities (`models.rs`)

**Files:**
- Modify: `backend_rust/src/models.rs`

**Interfaces:**
- `Transaction.quantity`, `Transaction.price`, `Transaction.fee` change from `f64` to `Decimal`
- `HistoricalPrice.close_price` changes from `f64` to `Decimal`
- All `FromRow` derive macros must still work (requires SQLx custom type impls for Decimal → TEXT in SQLite)

- [ ] **Step 1: Write failing test for Decimal transaction roundtrip**
Update the existing `test_transaction_roundtrip` test in `models.rs` to assert exact Decimal comparison instead of f64 epsilon comparison

- [ ] **Step 2: Replace f64 fields with Decimal in model structs**
In `models.rs`, change:
  - `Transaction.quantity: f64` → `Decimal`
  - `Transaction.price: f64` → `Decimal`
  - `Transaction.fee: f64` → `Decimal`
  - `HistoricalPrice.close_price: f64` → `Decimal`
Add `use rust_decimal::Decimal;` import

- [ ] **Step 3: Update test assertions**
Replace `assert!((deserialized.quantity - 50.0).abs() < f64::EPSILON)` with `assert_eq!(deserialized.quantity, Decimal::from_str("50.0").unwrap())`
Same pattern for `close_price` test

- [ ] **Step 4: Run tests**
Run: `cd backend_rust && cargo test --lib models::tests`
Expected: Tests pass (Decimal implements PartialEq)

- [ ] **Step 5: Commit**
```bash
git add backend_rust/src/models.rs
git commit -m "feat: replace f64 with Decimal in Transaction and HistoricalPrice models"
```

---

### Task 3: Replace f64 in API schema entities (`schemas.rs`)

**Files:**
- Modify: `backend_rust/src/schemas.rs`

**Interfaces:**
- All `f64` fields in `TransactionCreate`, `TransactionOut`, `TaxLot`, `AssetTaxSummary` become `Decimal`
- These are the request/response DTOs for the API — must match the model types

- [ ] **Step 1: Replace f64 fields in TransactionCreate**
Change `quantity`, `price`, `fee` from `f64` to `Decimal`

- [ ] **Step 2: Replace f64 fields in TransactionOut**
Change `quantity`, `price`, `fee` from `f64` to `Decimal`

- [ ] **Step 3: Replace f64 fields in TaxLot**
Change `buy_price`, `original_qty`, `remaining_qty`, `latent_gain_loss`, `latent_roi` from `f64` to `Decimal`

- [ ] **Step 4: Replace f64 fields in AssetTaxSummary**
Change `current_shares`, `average_cost`, `current_price`, `total_cost`, `market_value`, `unrealized_pnl`, `unrealized_roi`, `realized_pnl` from `f64` to `Decimal`

- [ ] **Step 5: Update test assertions**
Replace all `f64::EPSILON` comparisons in `schemas.rs` tests with `Decimal` equality assertions

- [ ] **Step 6: Run tests**
Run: `cd backend_rust && cargo test --lib schemas::tests`
Expected: All schema tests pass

- [ ] **Step 7: Commit**
```bash
git add backend_rust/src/schemas.rs
git commit -m "feat: replace f64 with Decimal in API schema DTOs"
```

---

### Task 4: Add SQLx custom type impls for Decimal in SQLite

**Files:**
- Create: `backend_rust/src/db_types.rs` (new file)
- Modify: `backend_rust/src/lib.rs` (add module declaration)

**Interfaces:**
- `Decimal` can be stored in and retrieved from SQLite `TEXT` columns
- `FromRow` derive works for structs containing `Decimal` fields
- `Type<Sqlite>` returns `TEXT` for `Decimal`
- `Encode<Sqlite>` converts `Decimal` to string for storage
- `Decode<Sqlite>` converts string back to `Decimal`

- [ ] **Step 1: Create db_types.rs with Decimal SQLite impls**
```rust
use rust_decimal::Decimal;
use sqlx::sqlite::{SqliteTypeInfo, SqliteValueRef};
use sqlx::{Type, Decode, Encode, Error as SqlxError};
use std::str::FromStr;

impl Type<sqlx::Sqlite> for Decimal {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<sqlx::Sqlite>>::type_info()
    }
    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl Encode<'_, sqlx::Sqlite> for Decimal {
    fn encode_by_ref(&self, buf: &mut sqlx::sqlite::SqliteArgumentBuffer) -> Result<(), SqlxError> {
        let s = self.to_string();
        <String as Encode<sqlx::Sqlite>>::encode(s, buf)
    }
}

impl Decode<'_, sqlx::Sqlite> for Decimal {
    fn decode(value: SqliteValueRef<'_>) -> Result<Self, SqlxError> {
        let s = <String as Decode<sqlx::Sqlite>>::decode(value)?;
        Decimal::from_str(&s).map_err(|e| SqlxError::RowNotFound { source: std::io::Error::new(std::io::ErrorKind::InvalidData, e) })
    }
}
```

- [ ] **Step 2: Add module declaration to lib.rs**
Add `pub mod db_types;` to `backend_rust/src/lib.rs`

- [ ] **Step 3: Add `use crate::db_types::Decimal;` in models.rs and schemas.rs**
(Or use `rust_decimal::Decimal` directly — the sqlx impls are in scope via the module)

- [ ] **Step 4: Run cargo check**
Run: `cd backend_rust && cargo check`
Expected: No compilation errors related to Decimal SQLx impls

- [ ] **Step 5: Commit**
```bash
git add backend_rust/src/db_types.rs backend_rust/src/lib.rs
git commit -m "feat: add SQLx Decimal type impls for SQLite TEXT storage"
```

---

### Task 5: Update database schema (`lib.rs` init_db)

**Files:**
- Modify: `backend_rust/src/lib.rs` (`init_db` function)

**Interfaces:**
- `transactions.quantity`, `transactions.price`, `transactions.fee` change from `REAL` to `TEXT`
- `historical_prices.close_price` changes from `REAL` to `TEXT`
- Existing data migration: convert REAL values to TEXT representation

- [ ] **Step 1: Update CREATE TABLE for transactions**
Change `quantity REAL NOT NULL` → `quantity TEXT NOT NULL`, `price REAL NOT NULL` → `price TEXT NOT NULL`, `fee REAL NOT NULL` → `fee TEXT NOT NULL`

- [ ] **Step 2: Update CREATE TABLE for historical_prices**
Change `close_price REAL NOT NULL` → `close_price TEXT NOT NULL`

- [ ] **Step 3: Add data migration for existing REAL columns**
Add migration that converts existing REAL values to TEXT:
```sql
ALTER TABLE transactions RENAME COLUMN quantity TO quantity_old;
ALTER TABLE transactions ADD COLUMN quantity TEXT NOT NULL DEFAULT '0';
UPDATE transactions SET quantity = printf('%.10f', quantity_old);
ALTER TABLE transactions DROP COLUMN quantity_old;
```
(Repeat for `price`, `fee`, and `historical_prices.close_price`)

- [ ] **Step 4: Run cargo check**
Run: `cd backend_rust && cargo check`
Expected: Compiles without errors

- [ ] **Step 5: Commit**
```bash
git add backend_rust/src/lib.rs
git commit -m "feat: change transaction and price DB columns from REAL to TEXT for Decimal storage"
```

---

### Task 6: Replace f64 in InternalLot and tax_engine.rs

**Files:**
- Modify: `backend_rust/src/engines/tax_engine.rs`

**Interfaces:**
- `InternalLot.price`, `InternalLot.qty`, `InternalLot.unit_cost` change from `f64` to `Decimal`
- `TaxLotEngine::calculate_lots` `current_price` parameter changes from `f64` to `Decimal`
- All local calculation variables use `Decimal`
- `CurrencyService::get_rate` return type changes from `f64` to `Decimal`
- `CurrencyService::get_price` return type changes from `f64` to `Decimal`

- [ ] **Step 1: Update InternalLot struct**
Change `price: f64` → `Decimal`, `qty: f64` → `Decimal`, `unit_cost: f64` → `Decimal`

- [ ] **Step 2: Update calculate_lots signature**
Change `current_price: f64` → `current_price: Decimal`

- [ ] **Step 3: Replace all f64 arithmetic in calculate_lots with Decimal**
- `realized_pnl` → `Decimal::ZERO`
- `unit_cost_asset` calculation uses `Decimal` division/multiplication
- `sell_unit_proceeds_asset` calculation uses `Decimal` division/multiplication
- All `lot.qty > 0.0` comparisons → `lot.qty > Decimal::ZERO`
- All `qty_to_sell <= 0.0` comparisons → `qty_to_sell <= Decimal::ZERO`
- `latent_roi` calculation handles division by Decimal
- `unrealized_roi` calculation handles division by Decimal

- [ ] **Step 4: Update test helper `make_tx`**
Change `qty: f64, price: f64, fee: f64` → `qty: Decimal, price: Decimal, fee: Decimal`
Use `Decimal::from_str("100.0").unwrap()` or `Decimal::new(1000, 1)` pattern

- [ ] **Step 5: Update test assertions**
Replace all `f64::EPSILON` comparisons with `Decimal` equality

- [ ] **Step 6: Run tests**
Run: `cd backend_rust && cargo test --lib engines::tax_engine`
Expected: All tax engine tests pass

- [ ] **Step 7: Commit**
```bash
git add backend_rust/src/engines/tax_engine.rs
git commit -m "feat: replace f64 with Decimal in tax engine and InternalLot"
```

---

### Task 7: Replace f64 in currency_service.rs

**Files:**
- Modify: `backend_rust/src/services/currency_service.rs`

**Interfaces:**
- `get_rate()` returns `Decimal` instead of `f64`
- `get_price()` returns `Decimal` instead of `f64`
- `fetch_yahoo_price()` returns `Decimal` instead of `f64`
- `fetch_eodhd_price()` returns `Decimal` instead of `f64`
- `cache_price()` accepts `Decimal` instead of `f64`
- `get_historical_prices()` returns `Vec<(NaiveDate, Decimal)>` instead of `Vec<(NaiveDate, f64)>`
- `get_historical_prices_from_db()` returns `Vec<(NaiveDate, Decimal)>` instead of `Vec<(NaiveDate, f64)>`
- Internal cache `HashMap<(String, String, NaiveDate), Decimal>` instead of `f64`
- Yahoo Finance JSON `close` prices parsed into `Decimal`
- EODHD `previous_close` parsed into `Decimal`

- [ ] **Step 1: Update Yahoo Finance response parsing**
Change `Quote.close: Vec<Option<f64>>` → `Quote.close: Vec<Option<String>>`
Parse each close price string into `Decimal` using `Decimal::from_str`

- [ ] **Step 2: Update EODHD response parsing**
Change `EodhdSearchResult.previous_close: Option<f64>` → `Option<Decimal>`
Parse the JSON number into `Decimal`

- [ ] **Step 3: Update cache type**
Change `HashMap<(String, String, NaiveDate), f64>` → `HashMap<(String, String, NaiveDate), Decimal>`

- [ ] **Step 4: Update all method signatures and return types**
`get_rate` → `Result<Decimal>`, `get_price` → `Decimal`, etc.

- [ ] **Step 5: Update `fetch_yahoo_price` and `fetch_eodhd_price`**
Return `Decimal` instead of `f64`. Parse JSON numbers via `serde_json` then convert: `Decimal::from_str(&value.to_string()).unwrap()`

- [ ] **Step 6: Update `cache_price` and `get_historical_prices`**
Accept/return `Decimal` types

- [ ] **Step 7: Update test assertions**
Replace `f64::EPSILON` comparisons with `Decimal` equality

- [ ] **Step 8: Run tests**
Run: `cd backend_rust && cargo test --lib services::currency_service`
Expected: All currency service tests pass

- [ ] **Step 9: Commit**
```bash
git add backend_rust/src/services/currency_service.rs
git commit -m "feat: replace f64 with Decimal in currency service"
```

---

### Task 8: Replace f64 in stats_engine.rs

**Files:**
- Modify: `backend_rust/src/engines/stats_engine.rs`

**Interfaces:**
- `price_map: HashMap<(NaiveDate, String), Decimal>` instead of `f64`
- `rate_cache: HashMap<String, Decimal>` instead of `f64`
- `asset_qtys: HashMap<String, Decimal>` instead of `f64`
- `prev_value: Option<Decimal>` instead of `Option<f64>`
- `twr_acc` stays as `f64` (statistical aggregation — documented rationale)
- `daily_val`, `daily_ret`, `final_val` change to `Decimal`
- All arithmetic uses `Decimal` operations
- `serde_json::Value` output still uses f64 for `value`, `daily_return`, `twr` (JSON serialization limitation)

- [ ] **Step 1: Update HashMap types**
Change `price_map`, `rate_cache`, `asset_qtys` to use `Decimal` values

- [ ] **Step 2: Update local variables**
Change `prev_value`, `daily_val`, `daily_ret`, `final_val` to `Decimal`
Keep `twr_acc` as `f64` with a comment explaining the statistical aggregation rationale

- [ ] **Step 3: Replace f64 arithmetic with Decimal arithmetic**
- `daily_val += qty * final_price` → uses Decimal multiplication
- `daily_ret = (daily_val - prev_val) / prev_val` → uses Decimal division
- Portfolio value calculations use Decimal

- [ ] **Step 4: Handle serde_json::Value conversion**
When pushing to `history` vec and `weekly_history`, convert `Decimal` to `f64` for JSON:
`serde_json::json!({"value": value.to_f64().unwrap_or(0.0), ...})`
Add a comment that JSON serialization uses f64 for transport compatibility

- [ ] **Step 5: Update test assertions**
Replace `f64::EPSILON` comparisons with `Decimal` equality where applicable

- [ ] **Step 6: Run tests**
Run: `cd backend_rust && cargo test --lib engines::stats_engine`
Expected: All stats engine tests pass

- [ ] **Step 7: Commit**
```bash
git add backend_rust/src/engines/stats_engine.rs
git commit -m "feat: replace f64 with Decimal in stats engine"
```

---

### Task 9: Replace f64 in analytics.rs route handler

**Files:**
- Modify: `backend_rust/src/api_routes/analytics.rs`

**Interfaces:**
- `total_value`, `total_realized`, `total_unrealized` change from `f64` to `Decimal`
- JSON output uses `serde_json::Value` — convert `Decimal` to string or f64 for transport

- [ ] **Step 1: Update aggregate variables**
Change `total_value`, `total_realized`, `total_unrealized` from `f64` to `Decimal`

- [ ] **Step 2: Update JSON response construction**
Convert `Decimal` values in the JSON response using `.to_string()` or `.to_f64().unwrap_or(0.0)`

- [ ] **Step 3: Run cargo check**
Run: `cd backend_rust && cargo check`
Expected: Compiles without errors

- [ ] **Step 4: Commit**
```bash
git add backend_rust/src/api_routes/analytics.rs
git commit -m "feat: replace f64 with Decimal in analytics route handler"
```

---

### Task 10: Update frontend TypeScript types

**Files:**
- Modify: `frontend/src/types.ts`
- Modify: `frontend/src/types/api.d.ts` (auto-generated, but needs updating)

**Interfaces:**
- All `number` fields that represent monetary values or quantities in `Transaction`, `TaxLot`, `AssetTaxSummary`, `HistoryItem`, `PerformanceMetrics`, `PortfolioPerformance`, `TaxSummary` should be documented as `Decimal`-backed (string or number)
- Since the API now serializes `Decimal` as strings (for precision), the frontend types need updating

- [ ] **Step 1: Update Transaction type**
Change `quantity`, `price`, `fee` from `number` to `string` (API returns Decimal as string)

- [ ] **Step 2: Update TaxLot type**
Change `buy_price`, `original_qty`, `remaining_qty`, `latent_gain_loss`, `latent_roi` from `number` to `string`

- [ ] **Step 3: Update AssetTaxSummary type**
Change all `number` fields to `string`

- [ ] **Step 4: Update HistoryItem type**
Change `value`, `daily_return`, `twr`, `cash_flow` from `number` to `string`

- [ ] **Step 5: Update PerformanceMetrics type**
Change all `number` fields to `string`

- [ ] **Step 6: Update PortfolioPerformance type**
Update `correlation_matrix` values to `string`

- [ ] **Step 7: Update TaxSummary type**
Change all `number` fields to `string`

- [ ] **Step 8: Update formatters.ts**
`formatCurrency` and `formatPercent` now receive `string` inputs — parse with `parseFloat` or use `Intl.NumberFormat` with string conversion

- [ ] **Step 9: Commit**
```bash
git add frontend/src/types.ts frontend/src/utils/formatters.ts
git commit -m "feat: update frontend types to string for Decimal-backed API fields"
```

---

### Task 11: Regenerate OpenAPI spec and update api.d.ts

**Files:**
- Modify: `docs/openapi/openapi.json` (regenerate)
- Modify: `frontend/src/types/api.d.ts` (regenerate)

**Interfaces:**
- OpenAPI spec reflects `Decimal` fields as `type: string` with `format: decimal` (or similar)
- Frontend auto-generated types update accordingly

- [ ] **Step 1: Regenerate OpenAPI spec**
Run the update command per AGENTS.md instructions

- [ ] **Step 2: Regenerate frontend API types**
Run: `cd frontend && npm run generate-api` (or equivalent openapi-typescript command)
Check `package.json` for the exact command

- [ ] **Step 3: Verify generated types**
Confirm `api.d.ts` now has string types for Decimal fields

- [ ] **Step 4: Commit**
```bash
git add docs/openapi/openapi.json frontend/src/types/api.d.ts
git commit -m "feat: regenerate OpenAPI spec and frontend API types after Decimal migration"
```

---

### Task 12: Run full test suite and fix any remaining issues

**Files:**
- All test files

**Interfaces:**
- Full `cargo test` must pass
- Frontend tests must pass

- [ ] **Step 1: Run full Rust test suite**
Run: `cd backend_rust && cargo test`
Expected: All tests pass

- [ ] **Step 2: Run frontend tests**
Run: `cd frontend && npm test`
Expected: All tests pass

- [ ] **Step 3: Fix any remaining f64 usages**
Search for any remaining `f64` in non-test, non-JSON code paths and fix

- [ ] **Step 4: Run cargo clippy**
Run: `cd backend_rust && cargo clippy -- -D warnings`
Expected: No warnings

- [ ] **Step 5: Commit**
```bash
git add -A
git commit -m "fix: resolve remaining issues after Decimal migration"
```

---

### Task 13: Create the PR

**Files:**
- None (git operation)

**Interfaces:**
- PR title and description summarize the migration

- [ ] **Step 1: Ensure all commits are on a feature branch**
Run: `git checkout -b feat/decimal-precision-migration`
(If already on main, create branch and cherry-pick or rebase commits)

- [ ] **Step 2: Push the branch**
Run: `git push origin feat/decimal-precision-migration`

- [ ] **Step 3: Create the PR**
Use `gh pr create` with appropriate title and body describing the migration

- [ ] **Step 4: Verify PR is created**
Run: `gh pr view`
Expected: PR exists and is open
