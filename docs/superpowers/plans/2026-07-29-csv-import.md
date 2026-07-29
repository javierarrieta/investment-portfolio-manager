# CSV Transaction Import Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow users to upload a CSV file of transactions, map columns, validate all rows, and import them into the portfolio with duplicates skipped and missing assets auto-created.

**Architecture:** Frontend-only CSV parsing using `papaparse`. Column names auto-detected (case-insensitive). Dates auto-detected from common formats. Validation runs against all rows before any API calls. Transactions posted to existing endpoints; missing assets created on the fly.

**Tech Stack:** `papaparse` (frontend CSV parsing), existing React/Vite frontend, existing Rocket/Rust backend (no new endpoints).

## Global Constraints
- No new backend endpoints required
- Leverage existing `POST /api/portfolios/<id>/assets` for auto-creating assets
- Leverage existing `POST /api/portfolios/<id>/assets/<asset_id>/transactions` for transactions
- Fee defaults to 0 if missing from CSV
- CSV parsing stays client-side (no multipart upload)
- TDD: frontend tests with Vitest + MSW

---

### Task 1: Add papaparse and TypeScript types for CSV import

**Files:**
- Modify: `frontend/package.json` (add `papaparse` dependency)
- Create: `frontend/src/types/csv.ts`

- [ ] **Step 1**: Install `papaparse` — `npm install papaparse` in `frontend/`
- [ ] **Step 2**: Create `frontend/src/types/csv.ts` with interfaces: `ColumnMapping` (maps CSV column index to `date` | `symbol` | `type` | `quantity` | `price` | `fee`), `CsvImportResult` (array of per-row results), `CsvValidationRow` (row number, field, error message)
- [ ] **Step 3**: Run `npm run lint` in `frontend/` to verify no type errors

### Task 2: Build column auto-detection and mapping utility

**Files:**
- Create: `frontend/src/utils/csvParser.ts`

- [ ] **Step 1**: Write `detectColumns(headers: string[]): ColumnMapping` — iterate through headers, match case-insensitively against known aliases: `date` ← `date\|datetime\|txn_date`; `symbol` ← `symbol\|ticker\|asset`; `type` ← `type\|side\|txn_type`; `quantity` ← `qty\|quantity\|shares\|amount`; `price` ← `price\|unit_price\|cost`; `fee` ← `fee\|commission\|cost_base`; unmatched columns get `null`; ties broken by first match
- [ ] **Step 2**: Write `detectDateFormat(sampleRows: string[][]): {format: string, ambiguous: boolean}` — extract the first cell matching a date pattern, try parsing with `YYYY-MM-DD`, `MM/DD/YYYY`, `DD/MM/YYYY`, `YYYY-MM-DD HH:MM`, `MM/DD/YYYY HH:MM`; return the format that successfully parses the most rows; mark `ambiguous: true` for formats like `MM/DD/YYYY` vs `DD/MM/YYYY` where both parse valid dates
- [ ] **Step 3**: Write unit tests in `frontend/src/utils/csvParser.test.ts` testing: exact column name matches, case-insensitive matches, unknown columns mapped to null, date format detection for each format, ambiguous date detection
- [ ] **Step 4**: Run tests — `npx vitest run frontend/src/utils/csvParser.test.ts` — verify all pass

### Task 3: Build CSV validation logic

**Files:**
- Create: `frontend/src/utils/csvValidator.ts`

- [ ] **Step 1**: Write `validateCsvRow(row: Record<string, string>, columnMapping: ColumnMapping, portfolioAssets: string[]): ValidationRowResult` — validate: date parseable and between 1900-01-01 and 2100-01-01; symbol non-empty; type is BUY/SELL (case-insensitive); quantity > 0; price > 0; fee >= 0 or missing (defaults to 0); return object with `valid: boolean`, `errors: string[]`, `normalizedRow: TransactionRow` (with parsed values)
- [ ] **Step 2**: Write `validateCsvFile(rows: any[], columnMapping: ColumnMapping, portfolioAssets: string[]): ValidationReport` — iterate all rows, collect validation errors per row, collect unknown symbols that don't match any portfolio asset; return `{ valid: boolean, totalRows: number, validRows: number, errorRows: ValidationRowResult[], unknownSymbols: string[] }`
- [ ] **Step 3**: Write unit tests in `frontend/src/utils/csvValidator.test.ts` testing: valid row passes, invalid date format rejected, missing symbol flagged, type not BUY/SELL rejected, negative quantity rejected, fee missing defaults to 0, unknown symbols collected
- [ ] **Step 4**: Run tests and verify all pass

### Task 4: Create `CsvImportModal` component

**Files:**
- Create: `frontend/src/components/CsvImportModal.tsx`

- [ ] **Step 1**: Create `CsvImportModal` component with props: `portfolioId: number`, `assets: Asset[]`, `onClose: () => void`, `onImportComplete: () => void`. State: `step` (upload | map | validate | confirm | results), `csvRows`, `columnMapping`, `dateFormat`, `validationReport`, `importResults`
- [ ] **Step 2**: Upload step — file input accepting `.csv`, on file selected call `papaparse.parse(file, {header: true, preview: 5})` for preview, store full parsed rows; show error if file is not CSV or is empty
- [ ] **Step 3**: Map step — display table of first 1 row with column headers; show auto-detected mapping for each column; for columns mapped to `null` or ambiguous, render a dropdown (`field -> value`) allowing user to assign or skip; show detected date format with override input; column mapping changes update state
- [ ] **Step 4**: Validate step — when user clicks "Validate", call `validateCsvFile(rows, columnMapping, portfolioAssets.map(a => a.symbol))`; display summary: total rows, valid count, error count, list of unknown symbols (will be auto-created as STOCK/USD); show per-row errors in expandable section; disable "Confirm" if validation has blocking errors (missing required columns, all rows invalid)
- [ ] **Step 5**: Confirm step — show summary counts: new transactions, duplicates to skip, warnings; "Import" button triggers `importCsv()` function
- [ ] **Step 6**: Results step — after import completes, show per-row results (success/error) in a table; "Done" button calls `onImportComplete`
- [ ] **Step 7**: Write unit tests in `frontend/src/components/CsvImportModal.test.tsx` — test each step renders correctly, test file upload triggers papaparse, test column mapping updates on dropdown change

### Task 5: Wire up `CsvImportModal` into `PortfolioDetail`

**Files:**
- Modify: `frontend/src/components/PortfolioDetail.tsx`

- [ ] **Step 1**: Add `CsvImportModal` to imports
- [ ] **Step 2**: Add `showImportModal` state (boolean)
- [ ] **Step 3**: Add "Import CSV" button in the PortfolioDetail toolbar (next to "Add Transaction" button)
- [ ] **Step 4**: Render `CsvImportModal` conditionally when `showImportModal` is true, passing `portfolio.id`, `portfolio.assets`, `onClose` (sets state false), `onImportComplete` (sets state false + triggers transaction refetch)

### Task 6: Integrate into `App.tsx` transaction flow

**Files:**
- Modify: `frontend/src/App.tsx`

- [ ] **Step 1**: Verify `PortfolioDetail` receives `portfolio` prop with `assets` and `id` fields (already passed via `onAddTransaction` callback)
- [ ] **Step 2**: After CSV import completes and `onImportComplete` fires, verify the transaction list refreshes by calling the existing `fetchTransactions` function in `App.tsx`
- [ ] **Step 3**: Run `npm run dev` in `frontend/` and manually test: create a test portfolio, add an asset, upload a CSV with a mix of valid/invalid rows, verify results

### Task 7: Write integration tests for the import flow

**Files:**
- Create: `frontend/src/components/CsvImportModal.integration.test.tsx`

- [ ] **Step 1**: Set up MSW handlers for `POST /api/portfolios/<id>/assets` (returns 201 with new asset) and `POST /api/portfolios/<id>/assets/<assetId>/transactions` (returns 201 with new transaction)
- [ ] **Step 2**: Write integration test: upload CSV with 3 valid rows, auto-map columns, validate, confirm → 3 transactions created, ledger refreshed
- [ ] **Step 3**: Write integration test: upload CSV with duplicate rows → duplicates skipped, results report them
- [ ] **Step 4**: Write integration test: upload CSV with validation errors → errors shown, no API calls made
- [ ] **Step 5**: Write integration test: upload CSV with unknown symbol → asset auto-created, transaction created
- [ ] **Step 6**: Run `npx vitest run` — verify all integration tests pass

