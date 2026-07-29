# CSV Transaction Import Design

**Date**: 2026-07-29
**Status**: Proposed

## Overview
Add a CSV import feature that allows users to upload a transaction CSV file and have transactions automatically parsed, validated, and added to a portfolio. The import supports flexible column mapping, auto-detection of column names and date formats, auto-creation of missing assets, and strict pre-import validation with error reporting.

## Requirements

- Support flexible column mapping (user maps CSV columns to fields)
- Frontend-only parsing (no new backend multipart endpoint)
- Strict validation before any data is persisted
- Auto-create assets for unknown symbols (default currency: USD)
- Auto-detect column names (case-insensitive) and date formats
- Skip duplicate transactions (same date + asset + type + quantity + price)
- Fee is optional (defaults to 0)

## Column Mapping

### Fields to Map

| CSV Column (auto-detected) | Transaction Field | Required | Format |
|---|---|---|---|
| `date`, `datetime`, `txn_date` | `date` | Yes | Any common date format |
| `symbol`, `ticker`, `asset` | `symbol` (for asset lookup/creation) | Yes | String matching portfolio asset |
| `type`, `side`, `txn_type` | `type` (BUY/SELL) | Yes | Case-insensitive |
| `qty`, `quantity`, `shares`, `amount` | `quantity` | Yes | Positive number |
| `price`, `unit_price`, `cost` | `price` | Yes | Positive number |
| `fee`, `commission`, `cost_base` | `fee` | No | Number, defaults to 0 |

### Auto-Detection
- Column names are matched case-insensitively
- Date format auto-detection tries: `YYYY-MM-DD`, `MM/DD/YYYY`, `DD/MM/YYYY`, `YYYY-MM-DD HH:MM`, `MM/DD/YYYY HH:MM`
- Ambiguous date formats (e.g., `01/02/2025`) are flagged for user resolution
- Unmapped columns are flagged for user to assign or skip

## User Flow

1. **Upload**: User clicks "Import CSV" button in PortfolioDetail -> file picker accepts `.csv` files
2. **Preview & Map**: First 5 rows displayed; columns auto-mapped by name; dropdowns provided for unmapped/ambiguous columns; date format detected and shown, with option to override
3. **Validate**: All rows validated before any API calls. Report includes: invalid dates, missing required fields, invalid types/quantities/prices, unknown symbols (will be auto-created)
4. **Confirm & Import**: Summary shows X new transactions, Y duplicates skipped, Z errors. On confirmation, valid transactions are POSTed individually to existing endpoint `POST /api/portfolios/<id>/assets/<asset_id>/transactions`
5. **Results**: Report of successes and failures; transactions added to the ledger view

## Error Handling

### Validation (pre-import)
- Entire file validated before any data is persisted
- All errors reported in a single summary:
  - Missing required columns
  - Unparseable dates
  - Invalid types (not BUY/SELL)
  - Non-positive quantities or prices
  - Unknown symbols (listed but will be auto-created)
- User can fix and re-upload, or confirm to proceed with valid rows

### Runtime (during import)
- Duplicates detected by matching (date + asset_id + type + quantity + price + fee) and skipped
- Individual row failures (e.g., asset not found after creation) reported but do not halt the entire import
- Results summary shown at the end

## Backend Changes

### No new endpoints needed
- Leverages existing `POST /api/portfolios/<portfolio_id>/assets/<asset_id>/transactions` for transactions
- Leverages existing `POST /api/portfolios/<portfolio_id>/assets` for auto-creating assets

### Changes to `create_asset` endpoint behavior
- The existing `create_asset` endpoint already supports creating assets with a symbol. When auto-creating assets during CSV import:
  - Symbol is uppercased
  - `name` defaults to the symbol string
  - `asset_type` defaults to `"STOCK"`
  - `currency` defaults to `"USD"`
  - `sector` is left empty

## Frontend Changes

### New Component: `CsvImportModal` (`frontend/src/components/CsvImportModal.tsx`)
- File upload step with drag-and-drop or file picker
- Column mapping step with auto-detection and dropdown overrides
- Validation results step showing all errors
- Confirm import step with summary
- Results step showing successes and failures

### Updated Component: `PortfolioDetail`
- Add "Import CSV" button alongside existing "Add Transaction" button
- `CsvImportModal` rendered as a modal overlay

### New TypeScript Types
- `CsvImportResult` - tracks import outcome per row
- `ColumnMapping` - maps CSV column index to transaction field

### Dependencies
- `papaparse` for CSV parsing (add to frontend dependencies)

## Validation Rules

| Field | Rule |
|---|---|
| `date` | Must be parseable as a valid date; must be between 1900-01-01 and 2100-01-01 |
| `symbol` | Non-empty string; matched to existing asset or triggers auto-creation |
| `type` | Must be BUY or SELL (case-insensitive) |
| `quantity` | Must be a positive number (> 0) |
| `price` | Must be a positive number (> 0) |
| `fee` | Must be a non-negative number (>= 0); defaults to 0 if missing |
| Dedup | Skip if identical transaction already exists (date + asset_id + type + quantity + price + fee) |

## Testing

### Frontend Tests (Vitest + MSW)
- CSV parsing with various column configurations
- Column auto-detection logic
- Date format auto-detection
- Validation logic (invalid dates, missing fields, bad values)
- Deduplication detection
- Import flow integration (upload -> map -> validate -> confirm -> results)

### Backend Tests (no new endpoints, but validate assumptions)
- Verify existing `create_asset` handles auto-creation correctly (symbol upper-cased, defaults)
- Verify existing `create_transaction` handles edge cases

## Success Criteria
1. User can upload a CSV file with any reasonable column naming
2. Columns are auto-detected and mapped correctly
3. Validation catches all errors before any data is persisted
4. Missing assets are created automatically
5. Duplicates are skipped and reported
6. Import results are clearly displayed
