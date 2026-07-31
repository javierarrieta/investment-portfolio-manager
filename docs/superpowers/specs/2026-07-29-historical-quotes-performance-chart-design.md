# Historical Quote Fetching for Performance Chart — Design Spec

**Date**: 2026-07-29
**Status**: Approved
**Branch target**: `feat/historical-quotes-performance-chart`

---

## 1. Problem

The Historical Performance (Time-Weighted) graph shows no data for the period before the first portfolio transaction. The backend sets `start_date` to the earliest transaction date, and when no historical prices exist in the `historical_prices` table for a symbol, it falls back to fetching only today's price from Yahoo Finance — which then gets cached as a single daily snapshot. Repeated requests accumulate stale "today-only" values in the cache, producing a chart with flat lines and sudden spikes instead of real historical performance.

## 2. Goal

Display a continuous performance chart from ~1 year before the first portfolio transaction to today, using daily historical prices fetched from Yahoo Finance and aggregated to weekly averages for the chart display.

## 3. Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Lookback window | 1 year before first transaction | Gives context for pre-purchase price action; configurable later if needed |
| Storage granularity | Daily raw prices in `historical_prices` | Preserves flexibility for future daily view or different aggregation |
| Chart aggregation | Weekly averages | Reduces chart points for readability; daily is overkill for multi-month view |
| Fetch strategy | Single Yahoo Finance `/chart` call per symbol with `period1`/`period2` | One API call, full daily fidelity, well within Yahoo's limits (~365 points/year) |
| API surface | No changes — no date params on endpoint | Backend determines range internally; frontend is unchanged |

## 4. Architecture

### 4.1 Currency Service (`backend_rust/src/services/currency_service.rs`)

Add new method:

```
get_historical_prices(symbol: &str, start_date: NaiveDate, end_date: NaiveDate, pool: &SqlitePool) -> Result<Vec<(NaiveDate, f64)>>
```

Behavior:
1. Convert `start_date` and `end_date` to Unix timestamps (seconds since epoch)
2. Call Yahoo Finance `/chart` with explicit `period1`/`period2` params:
   ```
   https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?period1={start_ts}&period2={end_ts}&interval=1d&includePrePost=false
   ```
3. Parse the response into `(date, close_price)` tuples
4. Batch upsert all daily prices into `historical_prices` (INSERT OR REPLACE)
5. Return the complete daily series
6. If Yahoo fetch fails or returns no data, log a warning and return whatever exists in the DB

### 4.2 Stats Engine (`backend_rust/src/engines/stats_engine.rs`)

**Date range**:
```
start_date = first_transaction_date - 1 year
end_date = today
```
Yahoo's chart API returns whatever historical data it has — if the symbol has less than 1 year of history, the chart renders from the earliest available date.

**Replace** `get_historical_price_matrix` with calls to `currency_service.get_historical_prices()`.

**New method** — `aggregate_weekly(daily_history: Vec<HistoryItem>) -> Vec<HistoryItem>`:
- Group daily entries by ISO week number + year
- For each week, use the **last trading-day close** as the weekly value (not the average — more accurate for portfolio valuation)
- Compute weekly `value` and `twr` from the grouped data
- Return weekly-aggregated `HistoryItem` array

**Chart response**:
- `history` → weekly-aggregated data
- `metrics` → computed from daily data (unchanged)

### 4.3 API Route (`backend_rust/src/api_routes/analytics.rs`)

No changes to route signature or request parameters. The existing `GET /api/portfolios/{id}/performance` endpoint works as-is — the backend now determines the correct date range internally.

### 4.4 Frontend (`frontend/src/components/Dashboard.tsx`)

No changes to the chart component. The `history` array already contains `date`, `value`, and `twr` keys that render correctly. The XAxis will automatically adjust to weekly intervals.

Optional enhancement: add a small "data as of [today's date]" label below the chart title.

## 5. Data Flow

```
GET /api/portfolios/{id}/performance
  │
  ├─ 1. Fetch portfolio, assets, transactions from DB
  │
  ├─ 2. Compute date range:
  │     start_date = earliest_tx_date - 1 year
  │     end_date = today
  │
  ├─ 3. For each unique asset symbol:
  │     currency_service.get_historical_prices(symbol, start_date, end_date)
  │       ├─ Check existing prices in historical_prices
  │       ├─ Identify missing dates
  │       ├─ Fetch from Yahoo Finance (single /chart call with date range)
  │       ├─ Upsert all daily prices into historical_prices
  │       └─ Return complete daily series
  │
  ├─ 4. Compute daily portfolio values using the full daily price series
  │
  ├─ 5. Compute daily TWRR using the full daily price series
  │
  ├─ 6. Aggregate daily history → weekly history for chart
  │
  ├─ 7. Compute performance metrics (TWR total, volatility, Sharpe, beta)
  │     — still from daily data, no change
  │
  └─ 8. Return { history: [...weekly...], metrics: {...}, correlation_matrix: {...} }
```

## 6. Error Handling

| Scenario | Behavior |
|----------|----------|
| Yahoo API returns no data for symbol | Log warning, skip asset in chart, continue with other assets |
| Yahoo API timeout / network error | Fall back to whatever exists in `historical_prices`; return partial data |
| Symbol is delisted or invalid ticker | Log warning, skip asset, don't fail the request |
| Partial historical data (e.g. only 6 months) | Chart renders from earliest available date; no error |
| All assets have no historical data | Return empty `history` array with zero-value metrics; no 500 |
| DB write fails during upsert | Log warning, continue with in-memory data; don't lose the request |

## 7. Database Changes

**Index addition only** — no schema changes:

```sql
CREATE INDEX IF NOT EXISTS idx_historical_prices_symbol_date 
ON historical_prices(symbol, date);
```

This replaces the current full table scan on the `historical_prices` query.

Existing table remains unchanged:
```sql
CREATE TABLE historical_prices (
    symbol TEXT NOT NULL,
    date DATE NOT NULL,
    close_price REAL NOT NULL
)
```

## 8. What This Does NOT Change

- `historical_prices` table schema (no new columns or tables)
- Frontend chart component or its data shape
- Performance metrics computation (Sharpe, TWR total, volatility, beta — all still daily)
- Correlation matrix calculation
- Any other API routes or endpoints
- The existing `get_price()` single-price method in currency service
- The frontend `fetchPortfolioData` call signature

## 9. Testing Considerations

- **Unit test**: `get_historical_prices()` with mocked Yahoo response — verify parsing and upsert
- **Unit test**: `aggregate_weekly()` with known daily data — verify weekly grouping and last-trading-day selection
- **Unit test**: `get_portfolio_performance()` with full date range — verify `start_date` computation
- **Integration test**: Mock Yahoo Finance HTTP calls, verify end-to-end price fetch + chart data shape
- **Edge case test**: Symbol with no historical data — verify graceful handling, no 500
- **Edge case test**: Symbol with partial data (e.g. 2 years of a 3-year range) — verify gap fill behavior

## 10. Implementation Order

1. Database: add index on `historical_prices(symbol, date)`
2. Currency service: implement `get_historical_prices()` with Yahoo Finance chart API
3. Stats engine: update date range logic, replace price matrix fetch, add `aggregate_weekly()`
4. Remove dead `sync_historical_prices` function
5. Add tests
6. Verify end-to-end with `cargo test` and manual chart check
