# Add Assets via ISIN — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow users to add assets to portfolios by entering an ISIN, which resolves to a symbol via Yahoo Finance search and auto-populates the asset form.

**Architecture:** ISIN lookup is a dedicated backend endpoint that calls Yahoo Finance search. The frontend calls this endpoint on ISIN input, auto-populates form fields, and submits the asset with ISIN stored on the record. The existing symbol-based flow remains unchanged.

**Tech Stack:** Rust (Rocket), SQLx, SQLite, Yahoo Finance API, React + Vite, TypeScript

## Global Constraints

- ISIN format: 12 alphanumeric characters (`/^[A-Z]{2}[A-Z0-9]{10}$/i`)
- ISIN is required for all new assets
- ISIN is stored as `TEXT UNIQUE` on the `assets` table
- ISIN lookup uses Yahoo Finance search endpoint: `https://query1.finance.yahoo.com/v1/finance/search?q={isin}`
- Auto-populated fields are editable after lookup
- ISIN lookup failures show inline error in the form with fallback to symbol entry
- Existing symbol-based asset creation continues to work unchanged
- Follow existing patterns from the multi-currency implementation (migration check on startup, `CurrencyService` for currency detection)

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `backend_rust/src/lib.rs` | Modify | Add `isin` column to `assets` table DDL + startup migration |
| `backend_rust/src/models.rs` | Modify | Add `isin: Option<String>` to `Asset` struct |
| `backend_rust/src/schemas.rs` | Modify | Add `isin` to `AssetCreate` and `AssetOut` |
| `backend_rust/src/api_routes/lookup.rs` | Create | New `GET /api/assets/lookup?isin=...` endpoint |
| `backend_rust/src/api_routes/transactions.rs` | Modify | Accept `isin` in `create_asset`, store on asset |
| `backend_rust/src/api_routes/portfolios.rs` | Modify | Include `isin` in `AssetOut` construction |
| `backend_rust/src/openapi.rs` | Modify | Register lookup endpoint, update schemas |
| `frontend/src/components/PortfolioDetail.tsx` | Modify | Add ISIN field to Add Asset modal, lookup integration |
| `frontend/src/types.ts` | Modify | Add `isin: string \| null` to `Asset` interface |
| `frontend/src/types/api.d.ts` | Modify | Add `isin` to `AssetCreate`, add `AssetLookupResult` type |

---

### Task 1: Add `isin` column to `assets` table (backend schema + migration)

**Files:**
- Modify: `backend_rust/src/lib.rs:56-98`

**Interfaces:**
- Produces: `init_db` function that ensures `isin` column exists on `assets` table

- [ ] **Step 1: Add `isin` column to `CREATE TABLE` DDL**

In `backend_rust/src/lib.rs`, update the `assets` table creation SQL to include `isin TEXT UNIQUE`:

```sql
CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    portfolio_id INTEGER NOT NULL,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_type TEXT NOT NULL,
    sector TEXT,
    currency TEXT NOT NULL DEFAULT 'USD',
    isin TEXT UNIQUE,
    FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
)
```

- [ ] **Step 2: Add startup migration check for existing databases**

After the `init_db` function body (before `Ok(())`), add a migration check that runs `ALTER TABLE assets ADD COLUMN isin TEXT UNIQUE` if the column doesn't exist. Use the same pattern as the multi-currency migration in `docs/implementation_plan.md`.

The migration check should:
1. Query `PRAGMA table_info(assets)` to check if `isin` column exists
2. If not, execute `ALTER TABLE assets ADD COLUMN isin TEXT UNIQUE`
3. Log any errors but don't fail startup

- [ ] **Step 3: Run tests to verify no regressions**

Run: `cd backend_rust && cargo test`
Expected: All existing tests pass

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/lib.rs
git commit -m "feat: add isin column to assets table with migration check"
```

---

### Task 2: Add `isin` field to `Asset` model

**Files:**
- Modify: `backend_rust/src/models.rs:16-24`

**Interfaces:**
- Produces: `Asset` struct with `isin: Option<String>` field

- [ ] **Step 1: Add `isin` field to `Asset` struct**

In `backend_rust/src/models.rs`, add `pub isin: Option<String>,` to the `Asset` struct after the `currency` field:

```rust
#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Asset {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
    pub isin: Option<String>,
}
```

- [ ] **Step 2: Update `Asset` test roundtrip**

In the `test_asset_roundtrip` test, add `isin: Some("US0378331005".to_string())` to the test asset and assert on it.

- [ ] **Step 3: Run tests**

Run: `cd backend_rust && cargo test`
Expected: All tests pass including the updated roundtrip test

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/models.rs
git commit -m "feat: add isin field to Asset model"
```

---

### Task 3: Add `isin` to `AssetCreate` and `AssetOut` schemas

**Files:**
- Modify: `backend_rust/src/schemas.rs:27-46`

**Interfaces:**
- Produces: `AssetCreate` with `isin: String`, `AssetOut` with `isin: Option<String>`

- [ ] **Step 1: Add `isin` to `AssetCreate`**

In `backend_rust/src/schemas.rs`, add `pub isin: String,` to `AssetCreate` after `currency`:

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetCreate {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
    pub isin: String,
}
```

- [ ] **Step 2: Add `isin` to `AssetOut`**

Add `pub isin: Option<String>,` to `AssetOut` after `currency`:

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AssetOut {
    pub id: i32,
    pub portfolio_id: i32,
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub sector: Option<String>,
    pub currency: String,
    pub isin: Option<String>,
    pub transactions: Vec<TransactionOut>,
}
```

- [ ] **Step 3: Update `AssetCreate` test roundtrip**

In `test_asset_create_roundtrip`, add `isin: "US0378331005".to_string()` and assert on it.

- [ ] **Step 4: Update `AssetOut` test roundtrip**

In `test_asset_out_roundtrip`, add `isin: Some("US0378331005".to_string())` and assert on it.

- [ ] **Step 5: Run tests**

Run: `cd backend_rust && cargo test`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add backend_rust/src/schemas.rs
git commit -m "feat: add isin to AssetCreate and AssetOut schemas"
```

---

### Task 4: Create `GET /api/assets/lookup?isin=...` endpoint

**Files:**
- Create: `backend_rust/src/api_routes/lookup.rs`
- Modify: `backend_rust/src/lib.rs:8-12` (add module declaration)
- Modify: `backend_rust/src/openapi.rs` (register endpoint)

**Interfaces:**
- Consumes: `CurrencyService` (for currency detection), `SqlitePool` (not needed for lookup)
- Produces: `GET /api/assets/lookup?isin=...` returning `{ symbol, name, asset_type, currency }`

- [ ] **Step 1: Create `lookup.rs` with ISIN lookup endpoint**

Create `backend_rust/src/api_routes/lookup.rs`:

```rust
use rocket::{State, serde::json::Json, http::Status};
use serde::Serialize;
use crate::services::currency_service::CurrencyService;

#[derive(Serialize)]
pub struct AssetLookupResult {
    pub symbol: String,
    pub name: String,
    pub asset_type: String,
    pub currency: String,
}

#[utoipa::path(
    get,
    path = "/api/assets/lookup",
    params(
        ("isin" = String, Path, description = "ISIN to look up (12 alphanumeric characters)")
    ),
    responses(
        (status = 200, description = "Asset metadata resolved from ISIN", body = AssetLookupResult),
        (status = 400, description = "Invalid ISIN format"),
        (status = 404, description = "ISIN not found")
    )
)]
#[get("/lookup?isin=<isin>")]
pub async fn lookup_isin(
    isin: String,
    currency_service: &State<CurrencyService>,
) -> Result<Json<AssetLookupResult>, Status> {
    if !is_valid_isin(&isin) {
        return Err(Status::BadRequest);
    }

    let url = format!("https://query1.finance.yahoo.com/v1/finance/search?q={}", isin);
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| Status::InternalServerError)?;

    if !response.status().is_success() {
        return Err(Status::InternalServerError);
    }

    let body: serde_json::Value = response.json().await
        .map_err(|_| Status::InternalServerError)?;

    let quotes = body.get("quotes")
        .and_then(|q| q.as_array())
        .and_then(|arr| arr.first())
        .ok_or(Status::NotFound)?;

    let symbol = quotes.get("symbol")
        .and_then(|s| s.as_str())
        .ok_or(Status::NotFound)?
        .to_string();

    let name = quotes.get("name")
        .and_then(|n| n.as_str())
        .unwrap_or(&symbol)
        .to_string();

    let exchange = quotes.get("exchange")
        .and_then(|e| e.as_str())
        .unwrap_or("");

    let asset_type = if exchange.contains("CB") || exchange.contains("CM") {
        "CRYPTO".to_string()
    } else if exchange.contains("INDEX") {
        "ETF".to_string()
    } else {
        "STOCK".to_string()
    };

    let currency = CurrencyService::detect_currency(&symbol);

    Ok(Json(AssetLookupResult {
        symbol,
        name,
        asset_type,
        currency,
    }))
}

fn is_valid_isin(isin: &str) -> bool {
    let isin = isin.to_uppercase();
    if isin.len() != 12 {
        return false;
    }
    let mut chars = isin.chars();
    let country_code = chars.next().unwrap();
    let second = chars.next().unwrap();
    if !country_code.is_ascii_alphabetic() || !second.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_isin() {
        assert!(is_valid_isin("US0378331005"));
        assert!(is_valid_isin("US037833100"));
        assert!(is_valid_isin("DE000BAY0017"));
    }

    #[test]
    fn test_invalid_isin_too_short() {
        assert!(!is_valid_isin("US037833100"));
    }

    #[test]
    fn test_invalid_isin_too_long() {
        assert!(!is_valid_isin("US03783310050"));
    }

    #[test]
    fn test_invalid_isin_non_alphanumeric() {
        assert!(!is_valid_isin("US037833100!"));
    }

    #[test]
    fn test_invalid_isin_no_country_code() {
        assert!(!is_valid_isin("0378331005"));
    }
}
```

- [ ] **Step 2: Add `lookup` module to `lib.rs`**

In `backend_rust/src/lib.rs`, add `pub mod api_routes::lookup;` to the `api_routes` module block:

```rust
pub mod api_routes {
    pub mod portfolios;
    pub mod transactions;
    pub mod analytics;
    pub mod lookup;
}
```

- [ ] **Step 3: Register the lookup route in `build_rocket`**

In `backend_rust/src/lib.rs`, add the lookup route to the `build_rocket` function:

```rust
.mount("/api", routes![
    api_routes::lookup::lookup_isin,
    api_routes::transactions::create_asset,
    // ... existing routes
])
```

- [ ] **Step 4: Register the endpoint in `openapi.rs`**

In `backend_rust/src/openapi.rs`, add `crate::api_routes::lookup::lookup_isin` to the `paths` list and `AssetLookupResult` to the `components` schemas list.

- [ ] **Step 5: Add `reqwest` dependency check**

Verify `reqwest` is already a dependency in `backend_rust/Cargo.toml` (it is, used by `CurrencyService`).

- [ ] **Step 6: Run tests**

Run: `cd backend_rust && cargo test`
Expected: All tests pass including the new ISIN validation tests

- [ ] **Step 7: Commit**

```bash
git add backend_rust/src/api_routes/lookup.rs backend_rust/src/lib.rs backend_rust/src/openapi.rs
git commit -m "feat: add ISIN lookup endpoint"
```

---

### Task 5: Modify `create_asset` to accept and store ISIN

**Files:**
- Modify: `backend_rust/src/api_routes/transactions.rs:17-66`

**Interfaces:**
- Consumes: `AssetCreate` with `isin: String` field (from Task 3)
- Produces: `AssetOut` with `isin` field populated

- [ ] **Step 1: Update `create_asset` to handle ISIN**

In `backend_rust/src/api_routes/transactions.rs`, modify `create_asset`:

1. Add ISIN uniqueness check: query `SELECT * FROM assets WHERE isin = ?` and return `409` if found
2. Pass `isin` through to the INSERT statement
3. Include `isin` in the returned `AssetOut`

The INSERT statement becomes:
```sql
INSERT INTO assets (portfolio_id, symbol, name, asset_type, sector, currency, isin)
VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *
```

And the bind chain adds `.bind(&asset.isin)`.

- [ ] **Step 2: Update `AssetOut` construction in `create_asset`**

Add `isin: res.isin,` to the `AssetOut` struct construction.

- [ ] **Step 3: Run tests**

Run: `cd backend_rust && cargo test`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/api_routes/transactions.rs
git commit -m "feat: store isin on asset creation"
```

---

### Task 6: Update `AssetOut` in portfolio routes to include `isin`

**Files:**
- Modify: `backend_rust/src/api_routes/portfolios.rs:47-95`

**Interfaces:**
- Produces: `AssetOut` with `isin` field in `fetch_assets_for_portfolio`

- [ ] **Step 1: Add `isin` to `AssetOut` construction in `fetch_assets_for_portfolio`**

In `backend_rust/src/api_routes/portfolios.rs`, add `isin: a.isin,` to the `AssetOut` struct construction in `fetch_assets_for_portfolio`.

- [ ] **Step 2: Add `isin` to `AssetOut` construction in `list_portfolios`**

In the `list_portfolios` function, add `isin: a.isin,` to the `AssetOut` struct construction.

- [ ] **Step 3: Run tests**

Run: `cd backend_rust && cargo test`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/api_routes/portfolios.rs
git commit -m "feat: include isin in AssetOut responses"
```

---

### Task 7: Update OpenAPI spec

**Files:**
- Modify: `backend_rust/src/openapi.rs`

**Interfaces:**
- Produces: OpenAPI spec with lookup endpoint and `isin` fields in schemas

- [ ] **Step 1: Register `lookup_isin` in paths**

Add `crate::api_routes::lookup::lookup_isin` to the `paths` list in `openapi.rs`.

- [ ] **Step 2: Register `AssetLookupResult` in components**

Add `AssetLookupResult` to the `components` schemas list in `openapi.rs`.

- [ ] **Step 3: Regenerate OpenAPI JSON**

Run the OpenAPI regeneration command per the project's instructions in `AGENTS.md`:

```bash
cd backend_rust
touch portfolio.db
cargo run --release &
sleep 12
curl -s http://127.0.0.1:8000/api-docs/openapi.json > ../docs/openapi/openapi.json
kill $(jobs -p)
```

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/openapi.rs docs/openapi/openapi.json
git commit -m "feat: update OpenAPI spec with ISIN lookup endpoint"
```

---

### Task 8: Update frontend `Asset` type and API types

**Files:**
- Modify: `frontend/src/types.ts:14-23`
- Modify: `frontend/src/types/api.d.ts`

**Interfaces:**
- Produces: `Asset` interface with `isin: string | null`, `AssetCreate` with `isin: string`, `AssetLookupResult` type

- [ ] **Step 1: Add `isin` to `Asset` interface**

In `frontend/src/types.ts`, add `isin: string | null;` to the `Asset` interface:

```typescript
export interface Asset {
  id: number;
  symbol: string;
  name: string;
  asset_type: AssetType;
  sector?: string;
  portfolio_id: number;
  currency: string;
  isin: string | null;
  transactions: Transaction[];
}
```

- [ ] **Step 2: Add `isin` to `AssetCreate` in `api.d.ts`**

In `frontend/src/types/api.d.ts`, add `isin: string;` to the `AssetCreate` schema.

- [ ] **Step 3: Add `AssetLookupResult` type to `api.d.ts`**

Add the `AssetLookupResult` schema and operation to `api.d.ts`:

```typescript
AssetLookupResult: {
  symbol: string;
  name: string;
  asset_type: string;
  currency: string;
};
```

And add the lookup path:
```typescript
"/api/assets/lookup": {
  get: operations["lookup_isin"];
};
```

And the operation definition:
```typescript
lookup_isin: {
  parameters: {
    query: {
      isin: string;
    };
  };
  responses: {
    200: {
      content: {
        "application/json": components["schemas"]["AssetLookupResult"];
      };
    };
    400: {
      content: never;
    };
    404: {
      content: never;
    };
  };
};
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/types.ts frontend/src/types/api.d.ts
git commit -m "feat: add isin to frontend types and API types"
```

---

### Task 9: Update `PortfolioDetail.tsx` — Add ISIN field to Add Asset modal

**Files:**
- Modify: `frontend/src/components/PortfolioDetail.tsx:55-71,292-366`

**Interfaces:**
- Consumes: `GET /api/assets/lookup?isin=...` endpoint
- Produces: ISIN input field in Add Asset modal with auto-populate and inline error

- [ ] **Step 1: Add ISIN form state**

Add `isin` to the `assetForm` state initialization:

```typescript
const [assetForm, setAssetForm] = useState<Partial<Asset>>({
  symbol: '', name: '', asset_type: 'STOCK', sector: '', currency: 'USD', isin: ''
});
```

Add `lookupError` state for inline error display:

```typescript
const [lookupError, setLookupError] = useState<string | null>(null);
const [isLookingUp, setIsLookingUp] = useState(false);
```

- [ ] **Step 2: Add ISIN validation function**

Add a client-side ISIN validation function (near the existing `detectCurrencyFromSymbol`):

```typescript
function isValidIsin(isin: string): boolean {
  return /^[A-Z]{2}[A-Z0-9]{10}$/i.test(isin);
}
```

- [ ] **Step 3: Add ISIN lookup handler**

Add a function that calls the lookup endpoint when ISIN is entered:

```typescript
const handleIsinLookup = async (isin: string) => {
  if (!isValidIsin(isin)) {
    setLookupError('ISIN must be 12 alphanumeric characters');
    return;
  }
  setIsLookingUp(true);
  setLookupError(null);
  try {
    const res = await fetch(`/api/assets/lookup?isin=${isin.toUpperCase()}`);
    if (!res.ok) {
      if (res.status === 404) {
        setLookupError('ISIN not found. You can enter a symbol directly.');
      } else {
        setLookupError('Could not resolve ISIN. Please try again or enter a symbol.');
      }
      return;
    }
    const data = await res.json();
    setAssetForm(prev => ({
      ...prev,
      symbol: data.symbol,
      name: data.name,
      asset_type: data.asset_type as AssetType,
      currency: data.currency,
      isin: isin.toUpperCase(),
    }));
  } catch {
    setLookupError('Could not resolve ISIN. Please try again or enter a symbol.');
  } finally {
    setIsLookingUp(false);
  }
};
```

- [ ] **Step 4: Add ISIN input field to the modal form**

Add the ISIN input field as the first field in the "Register New Asset Symbol" form, above the symbol field:

```tsx
<div className="form-group">
  <label>ISIN (e.g. US0378331005)</label>
  <input
    type="text"
    value={assetForm.isin || ''}
    onChange={(e) => {
      const val = e.target.value.toUpperCase();
      setAssetForm(prev => ({ ...prev, isin: val }));
      setLookupError(null);
      if (val.length === 12) {
        handleIsinLookup(val);
      }
    }}
    onBlur={() => {
      const val = assetForm.isin || '';
      if (val.length === 12) {
        handleIsinLookup(val);
      }
    }}
    className="form-control"
    placeholder="e.g. US0378331005"
    maxLength={12}
    required
  />
  {isLookingUp && <span style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>Looking up ISIN...</span>}
  {lookupError && <span style={{ fontSize: '0.75rem', color: 'var(--color-danger)' }}>{lookupError}</span>}
</div>
```

- [ ] **Step 5: Make auto-populated fields editable**

The symbol, name, asset_type, and currency fields should remain editable after auto-population. No changes needed — they're already standard form fields that the user can modify.

- [ ] **Step 6: Reset ISIN state on modal close**

Update the cancel button handler and the form reset after submit to include `isin: ''` and clear `lookupError`.

- [ ] **Step 7: Commit**

```bash
git add frontend/src/components/PortfolioDetail.tsx
git commit -m "feat: add ISIN field to Add Asset modal with lookup"
```

---

### Task 10: Backend tests for ISIN lookup

**Files:**
- Modify: `backend_rust/src/api_routes/lookup.rs` (add integration-style tests)

**Interfaces:**
- Produces: Tests for ISIN validation, lookup endpoint, error cases

- [ ] **Step 1: Add test for invalid ISIN format returns 400**

Add a test in `lookup.rs` that verifies a malformed ISIN (e.g., "ABC") returns `400 Bad Request`.

- [ ] **Step 2: Add test for ISIN not found returns 404**

Add a test that verifies a valid-format but non-existent ISIN returns `404 Not Found`.

- [ ] **Step 3: Run all backend tests**

Run: `cd backend_rust && cargo test`
Expected: All tests pass including new lookup tests

- [ ] **Step 4: Commit**

```bash
git add backend_rust/src/api_routes/lookup.rs
git commit -m "test: add ISIN lookup endpoint tests"
```

---

### Task 11: Frontend tests for ISIN validation and lookup

**Files:**
- Create: `frontend/src/test/components/IsinLookup.test.tsx`

**Interfaces:**
- Produces: Tests for ISIN format validation, auto-population, error display

- [ ] **Step 1: Write test for ISIN format validation**

Test that `isValidIsin` returns true for valid ISINs (e.g., "US0378331005") and false for invalid ones (e.g., "ABC", "123456789012", "us0378331005").

- [ ] **Step 2: Write test for auto-population on successful lookup**

Mock the `/api/assets/lookup` endpoint and test that entering a valid ISIN auto-populates symbol, name, asset_type, and currency fields.

- [ ] **Step 3: Write test for inline error on failed lookup**

Mock the `/api/assets/lookup` endpoint to return 404 and test that an error message is displayed below the ISIN field.

- [ ] **Step 4: Write test that auto-populated fields remain editable**

Test that after ISIN lookup auto-populates fields, the user can still edit symbol, name, asset_type, and currency.

- [ ] **Step 5: Run frontend tests**

Run: `cd frontend && npm test`
Expected: All new tests pass

- [ ] **Step 6: Commit**

```bash
git add frontend/src/test/components/IsinLookup.test.tsx
git commit -m "test: add ISIN lookup frontend tests"
```

---

### Task 12: Integration test — Add asset via ISIN end-to-end

**Files:**
- Modify: `frontend/src/test/components/PortfolioDetail.test.tsx`

**Interfaces:**
- Produces: End-to-end test verifying ISIN-based asset creation flows correctly

- [ ] **Step 1: Write integration test for ISIN-based asset creation**

Test the full flow:
1. User enters valid ISIN in Add Asset modal
2. ISIN lookup succeeds and auto-populates form
3. User edits auto-populated fields if needed
4. User submits form
5. Asset appears in holdings with ISIN stored

- [ ] **Step 2: Run integration tests**

Run: `cd frontend && npm test`
Expected: Integration test passes

- [ ] **Step 3: Commit**

```bash
git add frontend/src/test/components/PortfolioDetail.test.tsx
git commit -m "test: add ISIN asset creation integration test"
```

---

### Task 13: Regenerate OpenAPI spec and verify

**Files:**
- Modify: `docs/openapi/openapi.json`

**Interfaces:**
- Produces: Updated OpenAPI spec with ISIN lookup endpoint and `isin` fields

- [ ] **Step 1: Regenerate OpenAPI spec**

```bash
cd backend_rust
touch portfolio.db
cargo run --release &
sleep 12
curl -s http://127.0.0.1:8000/api-docs/openapi.json > ../docs/openapi/openapi.json
kill $(jobs -p)
```

- [ ] **Step 2: Verify the spec includes the new endpoint**

Check that `docs/openapi/openapi.json` contains:
- `/api/assets/lookup` path
- `isin` field in `Asset` schema
- `AssetLookupResult` schema

- [ ] **Step 3: Commit**

```bash
git add docs/openapi/openapi.json
git commit -m "chore: regenerate OpenAPI spec with ISIN lookup"
```

---

### Task 14: Run full test suite and verify

**Files:** None (verification only)

- [ ] **Step 1: Run backend tests**

```bash
cd backend_rust && cargo test
```

Expected: All tests pass

- [ ] **Step 2: Run frontend tests**

```bash
cd frontend && npm test
```

Expected: All tests pass

- [ ] **Step 3: Run frontend lint**

```bash
cd frontend && npm run lint
```

Expected: No lint errors

- [ ] **Step 4: Commit any fixes**

If any test or lint failures, fix and commit.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final verification of ISIN add-funds feature"
```
