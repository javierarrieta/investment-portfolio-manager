# Add Assets via ISIN — Design Document

**Date**: 2026-08-02
**Status**: Proposed

## Overview

Add ISIN (International Securities Identification Number) as a primary identifier for adding assets to portfolios. Users can enter an ISIN, and the app resolves it to a symbol via Yahoo Finance's search endpoint, auto-populating the asset form with metadata (name, asset_type, currency). The ISIN is stored persistently on the Asset model. The existing symbol-based flow remains fully functional as a fallback.

## Requirements

1. Users can add assets by entering an ISIN
2. ISIN is resolved to a symbol via Yahoo Finance search API
3. Asset metadata (name, asset_type, currency) is auto-populated from the lookup
4. Auto-populated fields are editable after lookup
5. ISIN is stored on the Asset model as a unique field
6. ISIN format is validated on both client and server (12 alphanumeric characters)
7. ISIN is required for all new assets
8. Existing symbol-based asset creation remains functional as fallback

## Decisions Summary

| Decision | Choice |
|---|---|
| ISIN source | Yahoo Finance |
| Store ISIN on Asset | Yes (required, unique) |
| ISIN-to-symbol resolution | Yahoo Finance search endpoint (`/v1/finance/search`) |
| Error handling | Inline error in form |
| Lookup endpoint | Dedicated `GET /api/assets/lookup?isin=...` |
| Validation | Both client-side and backend |
| ISIN required | Yes |
| Auto-populated fields | Editable |

## Backend Changes

### 1. Schema Migration — Add `isin` column to `assets` table

In `backend_rust/src/lib.rs` (`init_db`), add a migration check on startup:

```sql
ALTER TABLE assets ADD COLUMN isin TEXT UNIQUE
```

This mirrors the multi-currency migration pattern from the implementation plan. For new databases, the `CREATE TABLE IF NOT EXISTS` statement for `assets` should include `isin TEXT UNIQUE` in the column definitions. For existing databases, the startup migration check adds the column via `ALTER TABLE assets ADD COLUMN isin TEXT UNIQUE`.

### 2. Model — Add `isin` field to `Asset`

In `backend_rust/src/models.rs`, add to the `Asset` struct:

```rust
pub isin: Option<String>,
```

`Option<String>` because existing assets won't have ISINs.

### 3. Schemas — Add `isin` to `AssetCreate` and `AssetOut`

In `backend_rust/src/schemas.rs`:

- `AssetCreate`: add `pub isin: String` (required — ISIN is mandatory)
- `AssetOut`: add `pub isin: Option<String>`

### 4. New endpoint: `GET /api/assets/lookup?isin=...`

Create `backend_rust/src/api_routes/lookup.rs`:

**Request**: `GET /api/assets/lookup?isin=US0378331005`

**Validation**:
- ISIN must match `/^[A-Z]{2}[A-Z0-9]{10}$/i` (12 alphanumeric chars)
- Return `400` if format is invalid

**Resolution**:
- Call Yahoo Finance search: `https://query1.finance.yahoo.com/v1/finance/search?q={isin}`
- Parse response to extract `symbol`, `name`, `asset_type`, `exchange`
- Use `CurrencyService::detect_currency()` on the resolved symbol to determine currency
- Return `AssetLookupResult`

**Response** (`200`):
```json
{
  "symbol": "AAPL",
  "name": "Apple Inc.",
  "asset_type": "STOCK",
  "currency": "USD"
}
```

**Error** (`404`):
```json
{ "error": "ISIN not found" }
```

### 5. Modify `create_asset` endpoint

In `backend_rust/src/api_routes/transactions.rs`, `create_asset`:

- Accept `isin` field in `AssetCreate`
- If ISIN is provided:
  - Validate format (server-side)
  - Check uniqueness constraint — return `409` if ISIN already exists on another asset in the same portfolio
  - Store ISIN on the asset
- The existing duplicate-symbol check (`SELECT * FROM assets WHERE portfolio_id = ? AND symbol = ?`) remains
- **Note**: ISIN resolution (lookup) is handled entirely by the frontend before calling `create_asset`. The frontend calls `GET /api/assets/lookup?isin=...` first, then submits the resolved symbol along with the ISIN. The backend does not call the lookup endpoint internally.

### 6. OpenAPI spec update

In `backend_rust/src/openapi.rs`:
- Register the new `lookup` endpoint
- Update schemas to include `isin`

### 7. Update `AssetOut` in `fetch_assets_for_portfolio`

In `backend_rust/src/api_routes/portfolios.rs`, include `isin` in the `AssetOut` construction.

## Frontend Changes

### 1. Modify `PortfolioDetail.tsx` — Add Asset modal

The "Register New Asset Symbol" modal is updated:

- **ISIN field** is added as the **first/primary input** at the top of the form
- **Symbol field** remains but becomes secondary — auto-populated from ISIN lookup or manually entered
- On ISIN field blur or enter key:
  1. Client-side format validation (12 alphanumeric chars)
  2. If valid, call `GET /api/assets/lookup?isin={value}`
  3. On success: auto-populate symbol, name, asset_type, currency fields (all editable)
  4. On failure: show inline error below ISIN field — "ISIN not found. You can enter a symbol directly."
- ISIN field is required (both client-side format validation and server-side)
- The existing "Add Symbol" button submits the form as before

### 2. ISIN validation utility

Add a client-side validation function:
```typescript
function isValidIsin(isin: string): boolean {
  return /^[A-Z]{2}[A-Z0-9]{10}$/i.test(isin);
}
```

### 3. Update API types

In `frontend/src/types/api.d.ts`:
- Update `AssetCreate` schema to include `isin: string`
- Add `AssetLookupResult` type

### 4. Update `Asset` type in `frontend/src/types.ts`

Add `isin: string | null` to the `Asset` interface.

## Data Flow

```
User enters ISIN in Add Asset modal
  → Frontend validates ISIN format (12 alphanumeric)
  → If invalid: show inline error
  → If valid: GET /api/assets/lookup?isin={isin}
    → Backend validates format again
    → Backend calls Yahoo Finance search endpoint
    → Backend parses response → { symbol, name, asset_type, currency }
    → Backend returns result
  → On success: Frontend auto-populates form fields (editable)
  → On failure: Frontend shows inline error below ISIN field
  → User can edit any auto-populated field
  → User submits form → POST /api/portfolios/<id>/assets with { symbol, name, asset_type, currency, isin, ... }
    → Backend validates ISIN uniqueness
    → Backend stores asset with ISIN
  → Frontend refreshes holdings view
```

## Error Handling

| Scenario | User-Facing Error | Location |
|---|---|---|
| Invalid ISIN format | "ISIN must be 12 alphanumeric characters" | Below ISIN input |
| ISIN not found | "ISIN not found. You can enter a symbol directly." | Below ISIN input |
| Yahoo Finance API failure | "Could not resolve ISIN. Please try again or enter a symbol." | Below ISIN input |
| Duplicate ISIN on create | "An asset with this ISIN already exists" | Form-level error |
| Duplicate symbol on create | "Asset with this symbol already exists in this portfolio" | Existing behavior |

## Testing

### Backend Tests
- Unit test: ISIN format validation
- Unit test: Yahoo Finance search response parsing
- Unit test: Duplicate ISIN rejection (`409`)
- Unit test: ISIN lookup with invalid format returns `400`
- Unit test: ISIN lookup with unknown ISIN returns `404`

### Frontend Tests
- Test ISIN format validation (valid and invalid inputs)
- Test auto-population on successful lookup
- Test inline error display on failed lookup
- Test that auto-populated fields remain editable

### Integration Tests
- End-to-end: Add asset via ISIN, verify it appears in holdings with correct metadata
- End-to-end: Add asset via symbol (existing flow), verify ISIN is null
- End-to-end: Attempt to add duplicate ISIN, verify `409` error

## Success Criteria

1. User can add an asset by entering a valid ISIN
2. ISIN is resolved to symbol and metadata auto-populates the form
3. Auto-populated fields are editable after lookup
4. ISIN is stored on the Asset and displayed in the holdings view (e.g., as a tooltip or secondary label next to the symbol)
5. Invalid ISIN format is caught client-side before API call
6. ISIN lookup failures show inline error with fallback to symbol entry
7. Duplicate ISINs are rejected with a clear error
8. Existing symbol-based asset creation continues to work unchanged
