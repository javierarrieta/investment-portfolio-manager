# Multi-Currency Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix all currency handling gaps so portfolio/asset currencies are correctly stored and displayed, and transactions use the correct asset currency.

**Architecture:** The backend stores `currency` on both `portfolios` and `assets` tables. The frontend sends `currency` in asset/portfolio creation bodies. The core bug: the backend `create_asset` INSERT omits the `currency` column (always defaulting to USD), and `create_portfolio` does not bind `currency`. Fix requires updating schemas, route handlers, adding PATCH endpoints for currency changes, a currency auto-detection function, and updating frontend labels/forms.

**Tech Stack:** Rust (Rocket + SQLx + SQLite), React + TypeScript + Vite.

## Global Constraints

- No independent `base_currency` — always equals `portfolio.currency`
- Currency auto-detection from symbol suffix (`.DE` → EUR, `.L` → GBP, etc.)
- All existing tests must continue to pass
- Each task ends with a verifiable deliverable (tests pass or lint clean)

---

### Task 1: Add `detect_currency` function to CurrencyService

**Files:**
- Modify: `backend_rust/src/services/currency_service.rs`

**Interfaces:**
- Consumes: nothing
- Produces: `pub fn detect_currency(symbol: &str) -> String`

**Function logic:**

```rust
pub fn detect_currency(symbol: &str) -> String {
    let upper = symbol.to_uppercase();
    if upper.ends_with(".DE") || upper.ends_with(".F") || upper.ends_with(".FR") {
        "EUR".to_string()
    } else if upper.ends_with(".L") {
        "GBP".to_string()
    } else if upper.ends_with(".T") {
        "JPY".to_string()
    } else if upper.ends_with(".HK") {
        "HKD".to_string()
    } else if upper.ends_with(".SX") || upper.ends_with(".SW") {
        "CHF".to_string()
    } else if upper.ends_with(".TO") {
        "CAD".to_string()
    } else if upper.ends_with(".AX") {
        "AUD".to_string()
    } else if upper.ends_with(".K") {
        "KRW".to_string()
    } else if upper.contains("USD") || upper.contains("BTC") || upper.contains("ETH") {
        "USD".to_string()
    } else {
        "USD".to_string()
    }
}
```

**Steps:**
- [ ] **Step 1: Add the function to CurrencyService**

  Add this method to the `impl CurrencyService` block in `currency_service.rs`:

```rust
    pub fn detect_currency(symbol: &str) -> String {
        let upper = symbol.to_uppercase();
        if upper.ends_with(".DE") || upper.ends_with(".F") || upper.ends_with(".FR") {
            "EUR".to_string()
        } else if upper.ends_with(".L") {
            "GBP".to_string()
        } else if upper.ends_with(".T") {
            "JPY".to_string()
        } else if upper.ends_with(".HK") {
            "HKD".to_string()
        } else if upper.ends_with(".SX") || upper.ends_with(".SW") {
            "CHF".to_string()
        } else if upper.ends_with(".TO") {
            "CAD".to_string()
        } else if upper.ends_with(".AX") {
            "AUD".to_string()
        } else if upper.ends_with(".K") {
            "KRW".to_string()
        } else if upper.contains("USD") || upper.contains("BTC") || upper.contains("ETH") {
            "USD".to_string()
        } else {
            "USD".to_string()
        }
    }
```

- [ ] **Step 2: Add unit tests**

  Add these tests inside the existing `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn test_detect_currency_german_stock() {
        assert_eq!(CurrencyService::detect_currency("SAP.DE"), "EUR");
        assert_eq!(CurrencyService::detect_currency("SIE.DE"), "EUR");
    }

    #[test]
    fn test_detect_currency_uk_stock() {
        assert_eq!(CurrencyService::detect_currency("SHEL.L"), "GBP");
        assert_eq!(CurrencyService::detect_currency("BP.L"), "GBP");
    }

    #[test]
    fn test_detect_currency_japanese_stock() {
        assert_eq!(CurrencyService::detect_currency("7203.T"), "JPY");
    }

    #[test]
    fn test_detect_currency_btc() {
        assert_eq!(CurrencyService::detect_currency("BTC-USD"), "USD");
    }

    #[test]
    fn test_detect_currency_us_stock() {
        assert_eq!(CurrencyService::detect_currency("AAPL"), "USD");
        assert_eq!(CurrencyService::detect_currency("MSFT"), "USD");
    }

    #[test]
    fn test_detect_currency_case_insensitive() {
        assert_eq!(CurrencyService::detect_currency("sap.de"), "EUR");
        assert_eq!(CurrencyService::detect_currency("shel.l"), "GBP");
    }
```

- [ ] **Step 3: Run tests**

```bash
cd backend_rust && cargo test services::currency_service::tests
```

Expected: ALL tests PASS (existing + new).

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/services/currency_service.rs
git commit -m "feat(currency): add detect_currency heuristic for symbol-to-currency mapping"
```

---

### Task 2: Add `currency` field to `AssetCreate` and `AssetOut` schemas

**Files:**
- Modify: `backend_rust/src/schemas.rs`

**Interfaces:**
- Consumes: existing `AssetCreate`, `AssetOut`
- Produces: `AssetCreate.currency: String`, `AssetOut.currency: String`

**Steps:**
- [ ] **Step 1: Add `currency` to `AssetCreate`**

  Change the struct from:
```rust
pub struct AssetCreate {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
}
```
  To:
```rust
pub struct AssetCreate {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
}
```

- [ ] **Step 2: Add `currency` to `AssetOut`**

  Change the struct from:
```rust
pub struct AssetOut {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub transactions: Vec<TransactionOut>,
}
```
  To:
```rust
pub struct AssetOut {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
    pub transactions: Vec<TransactionOut>,
}
```

- [ ] **Step 3: Update the existing roundtrip test**

  Change `test_asset_out_roundtrip` to include `currency`:
```rust
fn test_asset_out_roundtrip() {
    let a = AssetOut {
        id: 1,
        portfolio_id: 1,
        symbol: "AAPL".to_string(),
        name: "Apple Inc".to_string(),
        asset_type: "STOCK".to_string(),
        sector: Some("Technology".to_string()),
        currency: "USD".to_string(),
        transactions: vec![],
    };
    let json = serde_json::to_string(&a).unwrap();
    let deserialized: AssetOut = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "AAPL");
    assert_eq!(deserialized.currency, "USD");
    assert_eq!(deserialized.sector, Some("Technology".to_string()));
}
```

- [ ] **Step 4: Update `test_asset_create_roundtrip` test**

  Change to include `currency`:
```rust
fn test_asset_create_roundtrip() {
    let a = AssetCreate {
        symbol: "TSLA".to_string(),
        name: "Tesla Inc".to_string(),
        asset_type: "STOCK".to_string(),
        sector: Some("Auto".to_string()),
        currency: "USD".to_string(),
    };
    let json = serde_json::to_string(&a).unwrap();
    let deserialized: AssetCreate = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "TSLA");
    assert_eq!(deserialized.asset_type, "STOCK");
    assert_eq!(deserialized.currency, "USD");
    assert_eq!(deserialized.sector, Some("Auto".to_string()));
}
```

- [ ] **Step 5: Run tests**

```bash
cd backend_rust && cargo test schemas::tests
```

Expected: ALL tests PASS.

- [ ] **Step 6: Commit**

```bash
git add backend_rust/src/schemas.rs
git commit -m "feat(currency): add currency field to AssetCreate and AssetOut schemas"
```

---

### Task 3: Fix `create_asset` route to persist `currency`

**Files:**
- Modify: `backend_rust/src/api_routes/transactions.rs`

**Interfaces:**
- Consumes: updated `AssetCreate` (now has `currency`)
- Produces: asset persisted with `currency` in DB

**Steps:**
- [ ] **Step 1: Fix the INSERT to include `currency`**

  Change the `create_asset` function:

  Replace:
```rust
    let res = sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (portfolio_id, symbol, name, asset_type, sector) 
         VALUES (?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(portfolio_id)
    .bind(asset.symbol.to_uppercase())
    .bind(&asset.name)
    .bind(asset.asset_type.to_uppercase())
    .bind(&asset.sector)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AssetOut {
        id: res.id,
        portfolio_id: res.portfolio_id,
        symbol: res.symbol,
        name: res.name,
        asset_type: res.asset_type,
        sector: res.sector,
        transactions: vec![],
    }))
```
  To:
```rust
    let res = sqlx::query_as::<_, Asset>(
        "INSERT INTO assets (portfolio_id, symbol, name, asset_type, sector, currency) 
         VALUES (?, ?, ?, ?, ?, ?) RETURNING *"
    )
    .bind(portfolio_id)
    .bind(asset.symbol.to_uppercase())
    .bind(&asset.name)
    .bind(asset.asset_type.to_uppercase())
    .bind(&asset.sector)
    .bind(&asset.currency)
    .fetch_one(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Json(AssetOut {
        id: res.id,
        portfolio_id: res.portfolio_id,
        symbol: res.symbol,
        name: res.name,
        asset_type: res.asset_type,
        sector: res.sector,
        currency: res.currency,
        transactions: vec![],
    }))
```

- [ ] **Step 2: Verify it compiles**

```bash
cd backend_rust && cargo check 2>&1
```

Expected: No errors.

- [ ] **Step 3: Run all existing tests**

```bash
cd backend_rust && cargo test
```

Expected: ALL tests PASS.

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/api_routes/transactions.rs
git commit -m "fix(backend): persist asset currency in create_asset INSERT"
```

---

### Task 4: Fix `create_portfolio` to bind currency from request

**Files:**
- Modify: `backend_rust/src/api_routes/portfolios.rs`

**Interfaces:**
- Consumes: existing `PortfolioCreate` (already has `currency` field)
- Produces: portfolio created with correct currency

**Steps:**
- [ ] **Step 1: Verify `create_portfolio` already works**

  Looking at `portfolios.rs:19-28`, the `create_portfolio` function already binds `portfolio.currency`:
```rust
    let res = sqlx::query_as::<_, DbPortfolio>(
        "INSERT INTO portfolios (name, description, currency, base_currency) 
         VALUES (?, ?, ?, ?) RETURNING *"
    )
    .bind(&portfolio.name)
    .bind(&portfolio.description)
    .bind(&portfolio.currency)
    .bind(&portfolio.currency) // Using currency as base_currency default
```

  This is **already correct** — `PortfolioCreate` has `currency` and it is bound to both `currency` and `base_currency`. The frontend bug was that it never sent `currency` in the body (only sent `name` and `description`). The backend is fine.

  No code changes needed for this task.

- [ ] **Step 2: Run tests to confirm**

```bash
cd backend_rust && cargo test
```

Expected: ALL tests PASS.

- [ ] **Step 3: Commit**

```bash
git commit --allow-empty -m "chore: verified create_portfolio correctly binds currency (frontend was the issue)"
```

---

### Task 5: Add PATCH endpoints for updating portfolio and asset currency

**Files:**
- Modify: `backend_rust/src/api_routes/portfolios.rs`
- Modify: `backend_rust/src/api_routes/transactions.rs`
- Modify: `backend_rust/src/main.rs`

**Interfaces:**
- Produces: `PATCH /portfolios/<id>` — updates portfolio currency (and base_currency to match)
- Produces: `PATCH /portfolios/<pid>/assets/<aid>` — updates asset currency

**Steps:**
- [ ] **Step 1: Add `PortfolioUpdate` schema and PATCH route to portfolios**

  Add to `backend_rust/src/schemas.rs` (inside the `use serde::{Serialize, Deserialize};` import, add `use serde::de::IgnoredAny;` is NOT needed — use a simple struct):

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortfolioUpdate {
    pub currency: String,
}
```

  Add this import at the top of `schemas.rs`:
```rust
use rocket::http::Status;
```

  Wait — `Status` is not needed in schemas. Just add the struct:

```rust
// --- Portfolio Update ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortfolioUpdate {
    pub currency: String,
}
```

  Add this AFTER the existing `PortfolioOut` struct definition.

- [ ] **Step 2: Add PATCH handler to `portfolios.rs`**

  Add this import at the top:
```rust
use crate::schemas::PortfolioUpdate;
```

  Add this function BEFORE `delete_portfolio`:

```rust
#[utoipa::path(
    patch,
    path = "/api/portfolios/<id>",
    responses(
        (status = 200, description = "Portfolio updated", body = PortfolioOut),
        (status = 404, description = "Portfolio not found")
    )
)]
#[patch("/<id>", data = "<update>")]
pub async fn update_portfolio(
    id: i32,
    pool: &State<SqlitePool>,
    update: Json<PortfolioUpdate>
) -> Result<Json<PortfolioOut>, Status> {
    let new_currency = &update.currency;

    sqlx::query("UPDATE portfolios SET currency = ?, base_currency = ? WHERE id = ?")
        .bind(new_currency)
        .bind(new_currency) // base_currency always equals currency
        .bind(id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let p = sqlx::query_as::<_, DbPortfolio>("SELECT * FROM portfolios WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    match p {
        Some(p) => Ok(Json(PortfolioOut {
            id: p.id,
            name: p.name,
            description: p.description,
            currency: p.currency,
            assets: vec![],
        })),
        None => Err(Status::NotFound),
    }
}
```

- [ ] **Step 3: Add `AssetUpdate` schema and PATCH handler to transactions**

  Add to `backend_rust/src/schemas.rs`:

```rust
// --- Asset Update ---
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetUpdate {
    pub currency: String,
}
```

  Add this import at the top of `transactions.rs`:
```rust
use crate::schemas::{AssetCreate, AssetOut, TransactionCreate, TransactionOut, AssetUpdate};
```

  Add this function before `delete_asset`:

```rust
#[utoipa::path(
    patch,
    path = "/api/portfolios/<portfolio_id>/assets/<asset_id>",
    responses(
        (status = 200, description = "Asset updated", body = AssetOut),
        (status = 404, description = "Asset not found")
    )
)]
#[patch("/portfolios/<portfolio_id>/assets/<asset_id>", data = "<update>")]
pub async fn update_asset(
    portfolio_id: i32,
    asset_id: i32,
    pool: &State<SqlitePool>,
    update: Json<AssetUpdate>
) -> Result<Json<AssetOut>, Status> {
    sqlx::query("UPDATE assets SET currency = ? WHERE id = ? AND portfolio_id = ?")
        .bind(&update.currency)
        .bind(asset_id)
        .bind(portfolio_id)
        .execute(pool.inner())
        .await
        .map_err(|_| Status::InternalServerError)?;

    let asset = sqlx::query_as::<_, Asset>(
        "SELECT * FROM assets WHERE id = ? AND portfolio_id = ?"
    )
    .bind(asset_id)
    .bind(portfolio_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|_| Status::InternalServerError)?;

    match asset {
        Some(a) => Ok(Json(AssetOut {
            id: a.id,
            portfolio_id: a.portfolio_id,
            symbol: a.symbol,
            name: a.name,
            asset_type: a.asset_type,
            sector: a.sector,
            currency: a.currency,
            transactions: vec![],
        })),
        None => Err(Status::NotFound),
    }
}
```

- [ ] **Step 4: Register new routes in `main.rs`**

  Add `use crate::schemas::{PortfolioUpdate, AssetUpdate};` is NOT needed — schemas are in their own module. But we need to register the route handlers.

  In `main.rs`, update the portfolio routes mount:
```rust
    .mount("/api/portfolios", routes![
        api_routes::portfolios::create_portfolio,
        api_routes::portfolios::list_portfolios,
        api_routes::portfolios::get_portfolio,
        api_routes::portfolios::delete_portfolio,
        api_routes::portfolios::update_portfolio
    ])
```

  And update the api routes mount to include `update_asset`:
```rust
    .mount("/api", routes![
        api_routes::transactions::create_asset,
        api_routes::transactions::update_asset,
        api_routes::transactions::delete_asset,
        api_routes::transactions::create_transaction,
        api_routes::transactions::list_portfolio_transactions,
        api_routes::transactions::delete_transaction
    ])
```

- [ ] **Step 5: Verify compilation**

```bash
cd backend_rust && cargo check 2>&1
```

Expected: No errors.

- [ ] **Step 6: Run all tests**

```bash
cd backend_rust && cargo test
```

Expected: ALL tests PASS.

- [ ] **Step 7: Commit**

```bash
git add backend_rust/src/schemas.rs backend_rust/src/api_routes/portfolios.rs backend_rust/src/api_routes/transactions.rs backend_rust/src/main.rs
git commit -m "feat(currency): add PATCH endpoints for updating portfolio and asset currency"
```

---

### Task 6: Frontend — Portfolio creation form with currency selector

**Files:**
- Modify: `frontend/src/App.tsx`

**Interfaces:**
- Consumes: existing `handleCreatePortfolio`
- Produces: portfolio creation sends `currency`, defaults to "USD"

**Steps:**
- [ ] **Step 1: Add currency state and selector to new portfolio modal**

  Add state near the existing portfolio modal state (around line 48):
```typescript
  const [newPortfolioCurrency, setNewPortfolioCurrency] = useState('USD');
```

- [ ] **Step 2: Update `handleCreatePortfolio` to send currency**

  Change the body from:
```typescript
        body: JSON.stringify({ name: newPortfolioName, description: newPortfolioDesc })
```
  To:
```typescript
        body: JSON.stringify({ name: newPortfolioName, description: newPortfolioDesc, currency: newPortfolioCurrency })
```

- [ ] **Step 3: Add currency selector to the New Portfolio Modal**

  Add this block inside the form, between the description textarea and the buttons div:
```tsx
              <div className="form-group">
                <label>Currency</label>
                <select
                  value={newPortfolioCurrency}
                  onChange={(e) => setNewPortfolioCurrency(e.target.value)}
                  className="form-control"
                >
                  <option value="USD">USD</option>
                  <option value="EUR">EUR</option>
                  <option value="GBP">GBP</option>
                  <option value="JPY">JPY</option>
                  <option value="CAD">CAD</option>
                  <option value="AUD">AUD</option>
                  <option value="CHF">CHF</option>
                  <option value="HKD">HKD</option>
                </select>
              </div>
```

- [ ] **Step 4: Reset currency state on modal close**

  Add to the Cancel button click handler — update the existing `setShowNewPortfolioModal(false)` call to also reset:
```typescript
                <button type="button" onClick={() => { setShowNewPortfolioModal(false); setNewPortfolioCurrency('USD'); }} className="btn btn-secondary">Cancel</button>
```

- [ ] **Step 5: Verify lint passes**

```bash
cd frontend && npm run lint
```

Expected: No errors or warnings.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/App.tsx
git commit -m "feat(frontend): add currency selector to portfolio creation form"
```

---

### Task 7: Frontend — Fix transaction labels and auto-detect asset currency

**Files:**
- Modify: `frontend/src/components/PortfolioDetail.tsx`

**Interfaces:**
- Consumes: `assetForm.currency` in add-asset modal, `asset.currency` in transaction form
- Produces: labels show correct currency, asset creation includes currency, cancel resets to USD

**Steps:**
- [ ] **Step 1: Fix transaction form labels to use asset currency**

  The transaction form needs to know the selected asset's currency. Add a helper to get the selected asset:

  Add near the top of the component body, after `const assets = portfolio.assets || [];`:
```typescript
  const selectedTxAsset = assets.find(a => a.id === Number(txForm.asset_id));
  const txCurrency = selectedTxAsset?.currency || portfolio.currency || 'USD';
```

  Then replace the hardcoded "USD" labels:

  Change `<label>Price (USD)</label>` to:
```tsx
                <label>Price ({txCurrency})</label>
```

  Change `<label>Transaction Fee (USD)</label>` to:
```tsx
                <label>Transaction Fee ({txCurrency})</label>
```

- [ ] **Step 2: Auto-detect currency from symbol in add-asset form**

  Add an `useEffect` that updates the currency when the symbol changes:

```typescript
  useEffect(() => {
    if (assetForm.symbol) {
      // Auto-detect currency from symbol (backend also does this)
      const detected = detectCurrencyFromSymbol(assetForm.symbol);
      if (detected && detected !== assetForm.currency) {
        setAssetForm(prev => ({ ...prev, currency: detected }));
      }
    }
  }, [assetForm.symbol]);
```

  Add the helper function at the top of the file (after imports):

```typescript
function detectCurrencyFromSymbol(symbol: string): string {
  const s = symbol.toUpperCase();
  if (s.endsWith('.DE') || s.endsWith('.F') || s.endsWith('.FR')) return 'EUR';
  if (s.endsWith('.L')) return 'GBP';
  if (s.endsWith('.T')) return 'JPY';
  if (s.endsWith('.HK')) return 'HKD';
  if (s.endsWith('.SX') || s.endsWith('.SW')) return 'CHF';
  if (s.endsWith('.TO')) return 'CAD';
  if (s.endsWith('.AX')) return 'AUD';
  if (s.endsWith('.K')) return 'KRW';
  if (s.includes('USD') || s.includes('BTC') || s.includes('ETH')) return 'USD';
  return 'USD';
}
```

- [ ] **Step 3: Reset asset form currency on cancel**

  Change the Cancel button in the add-asset modal to reset currency:
```tsx
                <button type="button" onClick={() => { setShowAssetModal(false); setAssetForm({ symbol: '', name: '', asset_type: 'STOCK' as AssetType, sector: '', currency: 'USD' }); }} className="btn btn-secondary">Cancel</button>
```

- [ ] **Step 4: Verify lint passes**

```bash
cd frontend && npm run lint
```

Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/components/PortfolioDetail.tsx
git commit -m "feat(frontend): use asset currency in transaction labels, auto-detect from symbol"
```

---

### Task 8: Frontend — Fix AnalyticsView to use portfolio currency

**Files:**
- Modify: `frontend/src/components/AnalyticsView.tsx`

**Interfaces:**
- Consumes: receives `portfolioCurrency` prop
- Produces: all currency labels formatted in portfolio currency

**Steps:**
- [ ] **Step 1: Accept currency prop and use it in formatter**

  Change the component signature to accept a `currency` prop:
```typescript
export default function AnalyticsView({ performance, currency = 'USD' }: { performance: PortfolioPerformance | null; currency?: string }) {
```

  Change the local `formatCurrency` to use the prop:
```typescript
  const formatCurrency = (val: number) => {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(val);
  };
```

- [ ] **Step 2: Pass portfolio currency from App.tsx**

  In `App.tsx`, change the AnalyticsView render:
```tsx
            {activeTab === 'analytics' && (
              <AnalyticsView performance={performance} currency={currentPortfolio?.currency || 'USD'} />
            )}
```

- [ ] **Step 3: Verify lint passes**

```bash
cd frontend && npm run lint
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/components/AnalyticsView.tsx frontend/src/App.tsx
git commit -m "feat(frontend): pass portfolio currency to AnalyticsView for correct formatting"
```

---

### Task 9: Run full verification

**Steps:**
- [ ] **Step 1: Run all backend tests**

```bash
cd backend_rust && cargo test 2>&1
```

Expected: ALL tests PASS.

- [ ] **Step 2: Run frontend lint**

```bash
cd frontend && npm run lint 2>&1
```

Expected: No errors or warnings.

- [ ] **Step 3: Verify backend builds**

```bash
cd backend_rust && cargo check 2>&1
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git commit --allow-empty -m "chore: full verification of multi-currency fixes"
```

---

## Plan Self-Review

**1. Spec coverage:**
- Asset currency persisted in DB: Task 2 (schema), Task 3 (INSERT fix)
- Portfolio currency from frontend: Task 4 (verified backend OK), Task 6 (frontend fix)
- Transaction labels show asset currency: Task 7
- Analytics uses portfolio currency: Task 8
- Changeable portfolio currency later: Task 5 (PATCH endpoint)
- Changeable asset currency later: Task 5 (PATCH endpoint)
- Auto-detect from symbol: Task 1 (backend), Task 7 (frontend)
- No independent base_currency: Tasks 4, 5 (always sets base_currency = currency)

**2. Placeholder scan:** No TBD, TODO, or "similar to" references found.

**3. Type consistency:** `AssetOut.currency` added in Task 2, used in Task 3. `PortfolioUpdate.currency` defined in Task 5. All consistent.

**4. Dependencies:** Task 2 must precede Task 3. Task 5 depends on Tasks 2+4 schemas. Frontend tasks are independent of backend task ordering.
