# Decimal Migration Fix Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the defects introduced by `feat/decimal-precision-migration` so that Decimal precision is real end-to-end and the existing frontend and legacy databases keep working.

**Architecture:** Adopt a single consistent API contract — **every Decimal-backed field is transported as a JSON string** (exact). JSON deserialization accepts strings, integers, and floats (lenient input, exact for strings/integers). Existing SQLite databases with legacy `REAL` columns are migrated to `TEXT` idempotently. The frontend normalizes the returned string values into JS numbers at the API boundary and sends `quantity`/`price`/`fee` as raw strings.

**Tech Stack:** `rust_decimal` (serde feature), SQLx/SQLite, Rocket, React+Vite with tests via Vitest + MSW.

## Global Constraints

- All monetary/quantity values cross the API as JSON **strings** (using `decimal_json`).
- `Decimal::from_str` is the only exact conversion for textual decimals; `from_f64_retain` is a documented, lenient fallback only (never a "precision" path).
- Never silently coerce an unparseable decimal to zero: log a warning at the boundary (DB reads / internal conversions) or fail (JSON input).
- `cargo test` must pass before every backend commit; `npm run lint` and `npm test` before every frontend commit.
- Do not touch unrelated files. Follow existing code style. No comments unless necessary.
- OpenAPI regenerated per AGENTS.md; frontend types regenerated via `npm run generate-types`.

---

### Task 1: Make `decimal_json` serialize as string and deserialize strings/integers/floats

**Files:**
- Modify: `backend_rust/src/db_types.rs`
- Test: `backend_rust/src/db_types.rs` (add `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: `rust_decimal::Decimal`, `rust_decimal::prelude::FromPrimitive`, `serde`.
- Produces: `decimal_json::serialize` (output JSON string), `decimal_json::deserialize` (accepts string / i64 / u64 / f64). `parse_decimal(&str) -> Result<Decimal, rust_decimal::Error>` and `str_to_decimal(&str) -> Decimal` (warning-logging fallback).

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Wrap { #[serde(with = "decimal_json")] d: Decimal }

    #[test]
    fn serialize_is_exact_string() {
        let w = Wrap { d: Decimal::from_str("9.99").unwrap() };
        assert_eq!(serde_json::to_string(&w).unwrap(), r#"{"d":"9.99"}"#);
    }

    #[test]
    fn integer_input_is_exact() {
        let w: Wrap = serde_json::from_str(r#"{"d":100}"#).unwrap();
        assert_eq!(w.d, Decimal::from_str("100").unwrap());
        let w: Wrap = serde_json::from_str(r#"{"d":-7}"#).unwrap();
        assert_eq!(w.d, Decimal::from_str("-7").unwrap());
    }

    #[test]
    fn string_input_is_exact() {
        let w: Wrap = serde_json::from_str(r#"{"d":"9.99"}"#).unwrap();
        assert_eq!(w.d, Decimal::from_str("9.99").unwrap());
    }

    #[test]
    fn fractional_sum_is_exact() {
        let sum = Decimal::from_str("0.1").unwrap() + Decimal::from_str("0.2").unwrap();
        assert_eq!(sum, Decimal::from_str("0.3").unwrap());
        assert_eq!(serde_json::to_string(&Wrap { d: sum }).unwrap(), r#"{"d":"0.3"}"#);
    }

    #[test]
    fn invalid_string_input_errors() {
        let res = serde_json::from_str::<Wrap>(r#"{"d":"abc"}"#);
        assert!(res.is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend_rust && cargo test --lib db_types::tests -v`
Expected: FAIL — serialization is currently a number and `visit_i64/u64` are not implemented.

- [ ] **Step 3: Implement `parse_decimal` / `str_to_decimal` and rewrite `decimal_json`**

Replace the helpers at the top of `db_types.rs`:

```rust
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn parse_decimal(s: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str(s)
}

pub fn str_to_decimal(s: &str) -> Decimal {
    match parse_decimal(s) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("WARN: invalid decimal string '{s}': {e}");
            Decimal::ZERO
        }
    }
}

pub fn decimal_to_str(d: &Decimal) -> String {
    d.to_string()
}
```

Rewrite the `decimal_json` module body:

```rust
pub mod decimal_json {
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(d: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&d.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DecimalVisitor;

        impl<'de> serde::de::Visitor<'de> for DecimalVisitor {
            type Value = Decimal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or number")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Decimal, E> {
                super::parse_decimal(v).map_err(|_| E::custom(format!("invalid decimal: {v}")))
            }

            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Decimal, E> {
                Decimal::from_i64(v).ok_or_else(|| E::custom("i64 does not fit decimal"))
            }

            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Decimal, E> {
                Decimal::from_u64(v).ok_or_else(|| E::custom("u64 does not fit decimal"))
            }

            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Decimal, E> {
                // Lenient fallback for legacy numeric clients; inherently lossy.
                Decimal::from_f64_retain(v).ok_or_else(|| E::custom("f64 does not fit decimal"))
            }
        }

        deserializer.deserialize_any(DecimalVisitor)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend_rust && cargo test --lib db_types::tests -v`
Expected: PASS.

- [ ] **Step 5: Verify the whole crate still compiles**

Run: `cd backend_rust && cargo check`
Expected: compiles. (Existing schema round-trip tests in `schemas.rs` use `serde_json::to_string` then `from_str` on decimal strings now; review them in Task 3.)

- [ ] **Step 6: Commit**

```bash
git add backend_rust/src/db_types.rs
git commit -m "fix: serialize decimals as exact JSON strings and accept int/float input"
```

---

### Task 2: Add SQLite `REAL -> TEXT` data migration for legacy databases

**Files:**
- Modify: `backend_rust/src/lib.rs` (add `migrate_decimal_columns` and call it from `init_db`)
- Test: `backend_rust/tests/decimal_migration_test.rs` (new integration test)

**Interfaces:**
- Consumes: `sqlx::SqlitePool`.
- Produces: `async fn migrate_decimal_columns(pool: &SqlitePool) -> Result<(), sqlx::Error>` — idempotent; converts `transactions.{quantity,price,fee}` and `historical_prices.close_price` from `REAL` to `TEXT` when they are still `REAL`. Called at the end of `init_db`, after the `CREATE TABLE IF NOT EXISTS` statements.

- [ ] **Step 1: Write failing integration test (legacy REAL schema).**

Create `backend_rust/tests/decimal_migration_test.rs`:

```rust
use sqlx::sqlite::SqlitePool;
use backend_rust::db_types::str_to_decimal;

async fn create_legacy_schema(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            quantity REAL NOT NULL,
            price REAL NOT NULL,
            fee REAL NOT NULL,
            date TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "CREATE TABLE historical_prices (
            symbol TEXT NOT NULL,
            date DATE NOT NULL,
            close_price REAL NOT NULL
        )",
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO transactions (asset_id, type, quantity, price, fee, date) VALUES (1, 'BUY', 100.0, 9.99, 0.0, '2024-01-01T00:00:00Z')")
        .execute(pool).await.unwrap();
    sqlx::query("INSERT INTO historical_prices (symbol, date, close_price) VALUES ('AAPL', '2024-01-01', 9.99)")
        .execute(pool).await.unwrap();
}

async fn column_affinity(pool: &SqlitePool, table: &str, column: &str) -> String {
    // PRAGMA table_info returns 6 columns: cid, name, type, notnull, dflt_value, pk
    let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as(&format!("PRAGMA table_info({table})")).fetch_all(pool).await.unwrap();
    rows.into_iter().find(|r| r.1 == column).unwrap().2
}

#[tokio::test]
async fn migration_converts_real_to_text_and_preserves_values() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();

    assert_eq!(column_affinity(&pool, "transactions", "quantity").await, "TEXT");
    assert_eq!(column_affinity(&pool, "transactions", "fee").await, "TEXT");
    assert_eq!(column_affinity(&pool, "historical_prices", "close_price").await, "TEXT");

    // values survive and parse as decimals ("100.0" came from REAL 100.0 via CAST AS TEXT)
    let (qty,): (String,) = sqlx::query_as("SELECT quantity FROM transactions WHERE id = 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(str_to_decimal(&qty), rust_decimal::Decimal::from_str("100.0").unwrap());
}

#[tokio::test]
async fn migration_is_idempotent() {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    create_legacy_schema(&pool).await;
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();
    backend_rust::migrate_decimal_columns(&pool).await.unwrap();
    assert_eq!(column_affinity(&pool, "transactions", "price").await, "TEXT");
}
```

Add `use rust_decimal::Decimal; use std::str::FromStr;` at the top of the test file (used by the `Decimal::from_str("100.0")` assertion).

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd backend_rust && cargo test --test decimal_migration_test`
Expected: FAIL — `migrate_decimal_columns` does not exist yet (compile error). `lib.rs` is the library crate root, so a `pub async fn migrate_decimal_columns` there is reachable as `backend_rust::migrate_decimal_columns`.

- [ ] **Step 3: Implement the migration helper in `lib.rs`**

Add near the end of `lib.rs` (after `init_db`):

```rust
async fn migrate_one_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
) -> Result<(), sqlx::Error> {
    let rows: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as(&format!("PRAGMA table_info({table})"))
            .fetch_all(pool)
            .await?;
    let current_affinity = rows.iter().find(|r| r.1 == column).map(|r| r.2.clone());
    if current_affinity.as_deref() == Some("TEXT") || current_affinity.is_none() {
        return Ok(());
    }

    sqlx::query(&format!("ALTER TABLE {table} RENAME COLUMN {column} TO {column}_old"))
        .execute(pool)
        .await?;
    sqlx::query(&format!(
        "ALTER TABLE {table} ADD COLUMN {column} TEXT NOT NULL DEFAULT '0'"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!(
        "UPDATE {table} SET {column} = CAST({column}_old AS TEXT)"
    ))
    .execute(pool)
    .await?;
    sqlx::query(&format!("ALTER TABLE {table} DROP COLUMN {column}_old"))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn migrate_decimal_columns(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for column in ["quantity", "price", "fee"] {
        migrate_one_column(pool, "transactions", column).await?;
    }
    migrate_one_column(pool, "historical_prices", "close_price").await
}
```

Call `migrate_decimal_columns(pool).await?;` at the end of `init_db`, after all `CREATE TABLE IF NOT EXISTS` statements and index creation. Requires SQLite >= 3.35 (verified with the project's bundled `libsqlite3-sys`; macOS system SQLite is newer).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd backend_rust && cargo test --test decimal_migration_test`
Expected: PASS.

- [ ] **Step 5: Run the full crate test suite**

Run: `cd backend_rust && cargo test`
Expected: ALL pass (includes existing DB integration tests that run `init_db` against `:memory:`).

- [ ] **Step 6: Commit**

```bash
git add backend_rust/src/lib.rs backend_rust/tests/decimal_migration_test.rs
git commit -m "fix: migrate legacy REAL price/quantity columns to TEXT for Decimal storage"
```

---

### Task 3: Harden silent parse sites and reconcile schema tests with string serialization

**Files:**
- Modify: `backend_rust/src/api_routes/transactions.rs`
- Modify: `backend_rust/src/api_routes/portfolios.rs`
- Modify: `backend_rust/src/services/currency_service.rs`
- Modify: `backend_rust/src/schemas.rs` (test expectations)

**Interfaces:**
- Consumes: `crate::db_types::str_to_decimal` (already warning-logging on failure).

- [ ] **Step 1: Replace `unwrap_or(Decimal::ZERO)` reads with `str_to_decimal` in api_routes**

In `backend_rust/src/api_routes/transactions.rs`, the three `Decimal::from_str(&res.quantity).unwrap_or(Decimal::ZERO)` (and `price`, `fee`) become `crate::db_types::str_to_decimal(&res.quantity)` (and `price`, `fee`). Same three in `list_portfolio_transactions`. In `backend_rust/src/api_routes/portfolios.rs`, `fetch_assets_for_portfolio` — change the three `Decimal::from_str(&tx.quantity).unwrap_or(Decimal::ZERO)` to `str_to_decimal`.

Remove now-unused `use std::str::FromStr;` and `use rust_decimal::Decimal;` imports where the call sites no longer need them; keep `Decimal` if still referenced for the totals (it is in `analytics.rs`, not these files). Verify with `cargo check` and remove unused-import warnings.

- [ ] **Step 2: Use `str_to_decimal` in currency_service DB reads**

In `backend_rust/src/services/currency_service.rs`, two DB decode sites return from strings:
- `get_price`: `return str_to_decimal(&hp.close_price);` (replace `crate::db_types::str_to_decimal` import — already used).
- `get_historical_prices_from_db`: `Ok(prices.into_iter().map(|p| (p.date, str_to_decimal(&p.close_price))).collect())`.

Remove the now-unused local `use std::str::FromStr;` import if no other usage remains (run `cargo check`).

- [ ] **Step 3: Update `schemas.rs` round-trip tests for string transport**

The existing tests serialize a `TransactionCreate`/`TaxLot`/`AssetTaxSummary` with `serde_json::to_string` then deserialize. With string serialization this still works (string in, string out). Tighten the earlier weak assertions:
- `test_transaction_create_roundtrip`: change the tolerance-based asserts to exact equality, e.g. `assert_eq!(deserialized.quantity, Decimal::from_str("100.0").unwrap());` for `quantity`/`price`, and for `fee` use `Decimal::from_str("9.99").unwrap()` **exactly** (this now holds because string transport is exact).
- `test_tax_lot_roundtrip` / `test_asset_tax_summary_roundtrip`: keeps `assert_eq!` (already exact).

- [ ] **Step 4: Run crate tests**

Run: `cd backend_rust && cargo test`
Expected: PASS.

- [ ] **Step 5: Run clippy**

Run: `cd backend_rust && cargo clippy --all-targets`
Expected: no new warnings introduced (existing unrelated clippy warnings may remain).

- [ ] **Step 6: Commit**

```bash
git add backend_rust/src/api_routes/transactions.rs backend_rust/src/api_routes/portfolios.rs backend_rust/src/services/currency_service.rs backend_rust/src/schemas.rs
git commit -m "fix: surface invalid decimal parsing and reconcile schema tests with string contract"
```

---

### Task 4: Fix stats engine holiday-week TWR carry-forward regression

**Files:**
- Modify: `backend_rust/src/engines/stats_engine.rs`
- Test: `backend_rust/src/engines/stats_engine.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes/Produces: `StatsEngine::aggregate_weekly(...)` unchanged signature; behavior of the zero-value (holiday/weekend) branch corrected.

- [ ] **Step 1: Write failing test**

Add to `stats_engine.rs` tests:

```rust
#[tokio::test]
async fn test_aggregate_weekly_carries_twr_across_zero_value_week() {
    // week1 gains 10% (twr 0.10), week2 is a holiday week (zero value),
    // week3 is a normal week. The holiday week must report the carried twr (0.10),
    // not 0.0.
    let daily_history = vec![
        serde_json::json!({"date":"2024-01-01","value":"100.0","daily_return":"0.0","twr":"0.0"}),
        serde_json::json!({"date":"2024-01-02","value":"110.0","daily_return":"0.10","twr":"0.10"}),
        // week2: single entry with value 0 -> holiday
        serde_json::json!({"date":"2024-01-08","value":"0","daily_return":"0.0","twr":"0.0"}),
        serde_json::json!({"date":"2024-01-15","value":"110.0","daily_return":"0.0","twr":"0.10"}),
    ];
    let weekly = StatsEngine::aggregate_weekly(daily_history).await;
    // week2 (index 1) must carry forward the 0.10 twr
    assert_eq!(weekly[1].get("twr").unwrap().as_str().unwrap(), "0.1");
}
```

Note: `value` arrives as a string in the new contract; the filter parses it and a `0`/`"0"` value is treated as a non-trading week.

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend_rust && cargo test --lib engines::stats_engine::tests::test_aggregate_weekly_carries_twr_across_zero_value_week`
Expected: FAIL — holiday branch currently emits `"twr":"0.0"`.

- [ ] **Step 3: Fix the holiday branch**

In `aggregate_weekly`, change the `_ => { ... }` branch that handles zero-value weeks from:

```rust
_ => {
    twr_acc *= 1.0;
    prev_value = Some(value);
    weekly_history.push(serde_json::json!({
        "date": date,
        "value": value.to_string(),
        "daily_return": "0.0",
        "twr": "0.0",
    }));
    continue;
}
```

to:

```rust
_ => {
    twr_acc *= 1.0;
    prev_value = Some(value);
    weekly_history.push(serde_json::json!({
        "date": date,
        "value": value.to_string(),
        "daily_return": "0.0",
        "twr": (twr_acc - 1.0).to_string(),
    }));
    continue;
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend_rust && cargo test --lib engines::stats_engine`
Expected: ALL PASS.

- [ ] **Step 5: Commit**

```bash
git add backend_rust/src/engines/stats_engine.rs
git commit -m "fix: carry forward TWR across zero-value (holiday) weeks in stats engine"
```

---

### Task 5: Regenerate OpenAPI spec and frontend API types

**Files:**
- Modify: `docs/openapi/openapi.json` (regenerated)
- Modify: `frontend/src/types/api.d.ts` (regenerated)

**Interfaces:**
- Consumes: `docs/openapi/openapi.json` (aka `docs/openapi/openapi.json` under AGENTS.md).

- [ ] **Step 1: Regenerate OpenAPI spec**

Run per AGENTS.md:

```bash
cd backend_rust
touch portfolio.db
cargo run --release &
sleep 12
curl -s http://127.0.0.1:8000/api-docs/openapi.json > ../docs/openapi/openapi.json
kill $(jobs -p)
```

- [ ] **Step 2: Regenerate frontend types**

Run: `cd frontend && npm run generate-types`
Expected: `frontend/src/types/api.d.ts` regenerated from the new spec.

- [ ] **Step 3: Commit**

```bash
git add docs/openapi/openapi.json frontend/src/types/api.d.ts
git commit -m "chore: regenerate OpenAPI and frontend API types after decimal contract"
```

---

### Task 6: Update frontend types and add a normalization layer for string-backified decimals

**Files:**
- Modify: `frontend/src/types.ts` (keep — see below)
- Modify: `frontend/src/utils/formatters.ts`
- Create: `frontend/src/utils/decimal.ts`
- Modify: `frontend/src/App.tsx` (apply normalizers to `perfData`, `taxData`, transaction payloads)

**Interfaces:**
- Consumes: existing `PortfolioPerformance`, `TaxSummary`, `Transaction`, `HistoryItem`, `PerformanceMetrics` shapes, which stay **numeric** in `types.ts`.
- Produces:
  - `utils/decimal.ts`: `toNumber(v: string | number | null | undefined): number`, `normalizePerformance(p: any): PortfolioPerformance`, `normalizeTaxSummary(t: any): TaxSummary`, `normalizeTransactionList(txs: any[]): Transaction[]`.
  - `App.tsx` applies normalizers on fetch and stops sending `parseFloat` values.
  - `PortfolioDetail.tsx` sends `quantity`/`price`/`fee` as raw strings in the create payload (locally typed, not via `types.ts`).

**Design note (type contract):** The API transports decimals as strings, but `types.ts` is kept as the **normalized, in-memory** shape (all numerics are `number`). The `decimal.ts` normalizers convert the string-form API payload into the numeric `types.ts` shapes at the fetch boundary, so existing components continue to receive and display plain numbers. Only the create-transaction *request* payload is string-typed (local interface in `PortfolioDetail.tsx`).

- [ ] **Step 1: Leave `frontend/src/types.ts` numeric (no signature change)**

No edits needed to `types.ts`. It already describes the numeric in-memory shapes that the normalizers produce. Confirm `Transaction`, `TaxLot`, `AssetTaxSummary`, `HistoryItem`, `PerformanceMetrics`, `TaxSummary` remain all-`number`.

- [ ] **Step 2: Create `frontend/src/utils/decimal.ts`**

Imports the existing numeric types and returns the same numeric shapes:

```ts
import type {
  HistoryItem, PerformanceMetrics, TaxSummary, Transaction,
} from '../types';

export type DecimalLike = string | number | null | undefined;

export const toNumber = (v: DecimalLike): number => {
  if (v === null || v === undefined || v === '') return 0;
  const n = typeof v === 'number' ? v : Number(v);
  return Number.isFinite(n) ? n : 0;
};

export const normalizeHistory = (h: any): HistoryItem => ({
  date: h.date ?? '',
  value: toNumber(h.value),
  daily_return: toNumber(h.daily_return),
  twr: toNumber(h.twr),
  cash_flow: toNumber(h.cash_flow),
});

export const normalizeMetrics = (m: any): PerformanceMetrics => ({
  volatility: toNumber(m?.volatility),
  sharpe_ratio: toNumber(m?.sharpe_ratio),
  beta: toNumber(m?.beta),
  portfolio_value: toNumber(m?.portfolio_value),
  beta_adjusted_exposure: toNumber(m?.beta_adjusted_exposure),
  unrealized_pnl: m?.unrealized_pnl !== undefined ? toNumber(m.unrealized_pnl) : undefined,
  realized_pnl: m?.realized_pnl !== undefined ? toNumber(m.realized_pnl) : undefined,
});

export const normalizePerformance = (p: any): PortfolioPerformance => ({
  history: (p?.history ?? []).map(normalizeHistory),
  correlation_matrix: p?.correlation_matrix ?? {},
  metrics: normalizeMetrics(p?.metrics),
});

export const normalizeTaxSummary = (t: any): TaxSummary => ({
  currency: t?.currency ?? 'USD',
  total_portfolio_value: toNumber(t?.total_portfolio_value),
  total_realized_pnl: toNumber(t?.total_realized_pnl),
  total_unrealized_pnl: toNumber(t?.total_unrealized_pnl),
  assets: (t?.assets ?? []).map((a: any) => ({
    symbol: a.symbol ?? '', asset_type: a.asset_type ?? '',
    current_shares: toNumber(a.current_shares), average_cost: toNumber(a.average_cost),
    current_price: toNumber(a.current_price), total_cost: toNumber(a.total_cost),
    market_value: toNumber(a.market_value), unrealized_pnl: toNumber(a.unrealized_pnl),
    unrealized_roi: toNumber(a.unrealized_roi), realized_pnl: toNumber(a.realized_pnl),
    tax_lots: (a.tax_lots ?? []).map((l: any) => ({
      buy_date: l.buy_date ?? '',
      buy_price: toNumber(l.buy_price), original_qty: toNumber(l.original_qty),
      remaining_qty: toNumber(l.remaining_qty), latent_gain_loss: toNumber(l.latent_gain_loss),
      latent_roi: toNumber(l.latent_roi),
    })),
  })),
});

export const normalizeTransactionList = (txs: any[]): Transaction[] => txs.map((t: any) => ({
  id: t.id, asset_id: t.asset_id, type: t.type, date: t.date,
  quantity: toNumber(t.quantity),
  price: toNumber(t.price),
  fee: toNumber(t.fee),
}));
```

- [ ] **Step 3: Update `frontend/src/utils/formatters.ts` to accept string|number**

```ts
import { toNumber } from './decimal';

export const formatCurrency = (value: string | number | null | undefined, currencyCode: string = 'USD'): string => {
  const v = toNumber(value);
  const CRYPTO_SYMBOLS: Record<string, string> = { BTC: '₿', ETH: 'Ξ' };
  if (CRYPTO_SYMBOLS[currencyCode]) {
    return `${CRYPTO_SYMBOLS[currencyCode]} ${v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}`;
  }
  try {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: currencyCode }).format(v);
  } catch {
    return `${currencyCode} ${v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  }
};

export const formatPercent = (value: string | number | null | undefined): string => {
  const v = toNumber(value);
  return `${v >= 0 ? '+' : ''}${(v * 100).toFixed(2)}%`;
};
```

- [ ] **Step 4: Update `App.tsx` fetches and payloads**

In `fetchPortfolioData`, after `const perfData = await perfRes.json();` add `const normalizedPerf = normalizePerformance(perfData);` and set it into state (replace `setPerformance(perfData)` with `setPerformance(normalizedPerf)`); after `const taxData = await taxRes.json();` use `normalizeTaxSummary(taxData)`; for transactions use `normalizeTransactionList(await txRes.json())`. Import the normalizers.

In `PortfolioDetail.tsx`'s create-transaction submit (the payload built before `POST .../transactions/`), stop converting to numbers for the decimal fields — send the raw strings. Defines a local payload type and passes `quantity: txForm.quantity`, `price: txForm.price`, `fee: txForm.fee` (the form already holds them as strings). This is what makes `9.99` reach the backend exactly (fixes C2 at the source).

- [ ] **Step 5: Run frontend tests and lint**

Run: `cd frontend && npm test`
Run: `cd frontend && npm run lint`
Expected: tests pass; lint clean. (See Task 7 for direct edits to components that still do `.toFixed` on strings and the mock payloads.)

- [ ] **Step 6: Commit**

```bash
git add frontend/src/utils/decimal.ts frontend/src/utils/formatters.ts frontend/src/App.tsx frontend/src/components/PortfolioDetail.tsx
git commit -m "feat: normalize string-backed decimals to numbers at the fetch boundary"
```

---

### Task 7: Update components and test mocks to the string/normalized contract

**Files:**
- Modify: `frontend/src/components/AnalyticsView.tsx`
- Modify: `frontend/src/components/Dashboard.tsx`
- Modify: `frontend/src/test/mocks/handlers.ts`
- Modify: `frontend/src/test/components/Dashboard.test.tsx`
- Modify: `frontend/src/test/components/PortfolioDetail.test.tsx`

**Interfaces:**
- Consumes: `toNumber` from `utils/decimal`; normalizers from Task 6 keep component props as plain numbers, so most component math already works. This task adds defensive coercion (for any raw-string data) and mirrors the real string contract in mocks/tests.

- [ ] **Step 1: Make `AnalyticsView` safe against any residual string values**

`AnalyticsView` receives `performance` (normalized to numbers by App.tsx), but defensive coercion is cheap and protects against raw-string props in tests. Replace direct numeric reads with `toNumber`:

```ts
import { toNumber } from '../utils/decimal';
// line ~51: {formatPercent(toNumber(metrics.volatility))}
// line ~61: {(toNumber(metrics.sharpe_ratio) || 0).toFixed(2)}
// line ~71: {(toNumber(metrics.beta) || 1).toFixed(2)}
// change numeric comparisons accordingly, e.g. const vol = toNumber(metrics.volatility)
```

Update the comparison expressions (`< 0.25`, `>= 1.0`, `Math.abs(... - 1.0) < 0.2`, `> 1.1`, `< 0.9`) to operate on the `toNumber(...)` result.

- [ ] **Step 2: Make `Dashboard.tsx` normalise the summary values it uses**

```ts
import { toNumber } from '../utils/decimal';
const totalValue = toNumber(taxSummary?.total_portfolio_value) || toNumber(metrics?.portfolio_value);
const realizedPnl = toNumber(taxSummary?.total_realized_pnl) || toNumber(metrics?.realized_pnl);
const unrealizedPnl = toNumber(taxSummary?.total_unrealized_pnl) || toNumber(metrics?.unrealized_pnl);
```

Keep the rest of the component logic (it already receives normalized history from App.tsx for the recharts `Area`/`Line` data). Update the allocation row `((item.value / totalValue) * 100)` — `item.value` is already a number post-normalization.

- [ ] **Step 3: Update MSW mock handlers to mirror the string contract**

In `frontend/src/test/mocks/handlers.ts`, the transaction/asset mocks currently return numbers for `quantity`/`price`/`fee`/`market_value` etc. Update them to return strings to mirror the backend (e.g. `quantity: '100.0'`, `price: '160.0'`, `total_portfolio_value: '17500'`). Keep the shape fields consistent with `types.ts`.

- [ ] **Step 4: Keep component-test fixtures numeric (they are normalized props)**

`Dashboard.test.tsx` and `PortfolioDetail.test.tsx` pass fixtures directly as component props, i.e. already-normalized data, so their numeric `total_portfolio_value`/`total_realized_pnl`/history values stay **numbers** — consistent with `types.ts`. No fixture type changes are required unless the test exercises the full `App` fetch path through MSW (then the responses go through `normalize*` and numeric assertions still hold). Run the tests and adjust only if a test asserts on a raw MSW response shape.

- [ ] **Step 5: Run frontend tests and lint**

Run: `cd frontend && npm test`
Run: `cd frontend && npm run lint`
Expected: PASS / clean.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/components/AnalyticsView.tsx frontend/src/components/Dashboard.tsx frontend/src/test/mocks/handlers.ts frontend/src/test/components/Dashboard.test.tsx frontend/src/test/components/PortfolioDetail.test.tsx
git commit -m "fix: handle string-backified decimals in analytics/dashboard and test mocks"
```

---

### Task 8: Full regression pass and final verification

**Files:** none (verification only).

- [ ] **Step 1: Run the full backend suite + clippy**

Run: `cd backend_rust && cargo test`
Run: `cd backend_rust && cargo clippy --all-targets`
Expected: all pass; no new warnings.

- [ ] **Step 2: Run the full frontend suite + lint**

Run: `cd frontend && npm test`
Run: `cd frontend && npm run lint`
Expected: all pass.

- [ ] **Step 3: Manual smoke test with a legacy database**

Start the API (per AGENTS.md `./start.sh`), point it at an existing (pre-migration) `backend/portfolio.db`, and confirm:
- `GET /api/portfolios/:id/transactions/` returns string `quantity`/`price`/`fee` and no errors (legacy REAL columns are migrated to TEXT).
- `GET /api/portfolios/:id/tax-summary` returns consistent string/string values.
- `GET /api/portfolios/:id/performance` returns string `value`/`daily_return`/`twr` and `metrics` as strings.
- Creating a transaction with integer `quantity` succeeds (no 422); with fractional `price` `9.99` is stored and returned exactly (no `9.9900000...`).

- [ ] **Step 4: Verify coverage of the changed decimals**

Run: `cd backend_rust && cargo llvm-cov --workspace --summary-only --ignore-filename-regex 'tests/'`
Expected: `db_types.rs` climbs to ~95%+ lines with `visit_str`/`visit_i64`/`visit_u64` all exercised; no new regressions elsewhere.

- [ ] **Step 5: Final commit of any stragglers**

```bash
git add -A
git commit -m "chore: final verification fixes after decimal migration hardening"
```
(Only commit if there are changes.)