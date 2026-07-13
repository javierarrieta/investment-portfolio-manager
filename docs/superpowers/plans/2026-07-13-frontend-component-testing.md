# Frontend Component Testing Plan

> **Goal:** Add component and utility tests to the frontend using the existing vitest + @testing-library/react + MSW infrastructure.

**Why now:** The multi-currency fix introduced several pieces of logic (`detectCurrencyFromSymbol`, currency prop threading, form auto-detection) that have zero test coverage. Component tests prevent regressions and give confidence for future changes.

**Tech Stack:** React 19 + Vite + TypeScript + vitest (already installed)

**Existing infrastructure (ready to use):**
- `@testing-library/react@^16.0.0` — installed in devDependencies
- `@testing-library/jest-dom@^6.4.0` — installed (for matchers like `toBeInTheDocument`)
- `@testing-library/user-event@^14.5.0` — installed (for simulating user interactions)
- `jsdom@^24.1.0` — installed (DOM environment for vitest)
- `msw@^2.4.0` — installed, with working mock server at `src/test/mocks/server.ts`
- `vitest@^4.1.9` — test runner, configured with `"test": "vitest run"`
- `src/test/setup.ts` — exists with MSW server lifecycle hooks
- `src/test/mocks/handlers.ts` — existing MSW handlers for `/api/portfolios`, `/api/assets`

---

## Task 1: Add vitest DOM config + verify test infrastructure

**Files:**
- Modify: `vite.config.ts` (create) or inline config in `package.json`

**What:** Configure vitest to use `jsdom` environment for component tests (separate from the existing handler test which runs in node).

**Steps:**
- [ ] Create `vite.config.ts` in `frontend/` root with vitest config that sets `test.environment = 'jsdom'` for all tests (simpler than per-file config)
- [ ] Add `src/setup.ts` reference to vitest config's `setupFiles` so `@testing-library/jest-dom` matchers are auto-available
- [ ] Run `npm run test` to verify existing `handlers.test.ts` still passes in jsdom

**Verification:**
```bash
cd frontend && npm run test
```

Expected: existing tests pass, no new errors.

---

## Task 2: Test pure utility functions

**Files:**
- New: `frontend/src/utils/formatters.test.ts`

**What:** Unit tests for `formatCurrency` and `formatPercent` — these are pure functions with no DOM dependency, fast to run, high signal.

**Tests to write:**

```typescript
// formatCurrency tests
- formats USD values with $ prefix
- formats EUR values with euro symbol
- formats GBP values with pound symbol
- formats JPY without decimal places (15000 → "¥15,000")
- handles crypto BTC with bitcoin symbol (₿)
- handles crypto ETH with ethereum symbol (Ξ)
- falls back to "CODE value" for unknown currency codes
- defaults to USD when no currencyCode provided

// formatPercent tests
- formats positive percentage with + prefix
- formats negative percentage with - prefix
- formats zero as "+0.00%"
```

**Verification:**
```bash
cd frontend && npx vitest run src/utils/formatters.test.ts
```

---

## Task 3: Test `detectCurrencyFromSymbol`

**Files:**
- New: `frontend/src/components/currency_detection.test.tsx`

**What:** The `detectCurrencyFromSymbol` function in `PortfolioDetail.tsx` is a module-level function (not exported). We need to either:
- (a) Extract it to a utility file (recommended — makes it testable and reusable)
- (b) Copy the logic into the test file for testing

**Recommended approach (a):** Extract to `frontend/src/utils/currency.ts`:
```typescript
export function detectCurrencyFromSymbol(symbol: string): string {
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

**Tests:**
```typescript
- detects EUR from .DE suffix
- detects EUR from .F suffix
- detects EUR from .FR suffix
- detects GBP from .L suffix
- detects JPY from .T suffix
- detects HKD from .HK suffix
- detects CHF from .SX suffix
- detects CHF from .SW suffix
- detects CAD from .TO suffix
- detects AUD from .AX suffix
- detects KRW from .K suffix
- detects USD from BTC-USD
- detects USD from ETH-USD
- detects USD from AAPL (unknown suffix → fallback)
- case insensitive (sap.de → EUR, shel.L → GBP)
```

**Verification:**
```bash
cd frontend && npx vitest run src/utils/currency.test.ts
```

---

## Task 4: Test AnalyticsView component

**Files:**
- New: `frontend/src/components/AnalyticsView.test.tsx`

**What:** Component tests for the analytics view — verifies currency prop threading and rendering.

**Tests:**
```typescript
- renders "Advanced Analytics Stale" when performance is null
- renders "Advanced Analytics Stale" when metrics are missing
- renders "Advanced Analytics Stale" when history is empty
- renders volatility metric when performance is provided
- renders Sharpe Ratio metric when performance is provided
- renders Beta metric when performance is provided
- renders Beta-Adjusted Net Exposure when performance is provided
- uses "USD" as default currency in formatCurrency
- uses "EUR" currency prop in formatCurrency (verifies currency prop threading)
- uses "GBP" currency prop in formatCurrency
- renders correlation matrix when symbols exist
- renders empty state "No correlation data" when no symbols
- renders risk assessment sections
- shows "Moderate Risk" for volatility < 0.25
- shows "High Risk Profile" for volatility >= 0.25
- shows "Strong Risk-Adjusted Returns" for sharpe >= 1.0
```

**Verification:**
```bash
cd frontend && npx vitest run src/components/AnalyticsView.test.tsx
```

---

## Task 5: Test PortfolioDetail component

**Files:**
- New: `frontend/src/components/PortfolioDetail.test.tsx`

**What:** Component tests for the portfolio detail view — the most complex component with forms, modals, and user interactions.

**Setup:** Mock all callback props (`onAddAsset`, `onDeleteAsset`, `onAddTransaction`) as vi.fn(). Use MSW for any fetch calls.

**Tests:**

**Rendering:**
```typescript
- renders portfolio name and description
- renders "No assets registered" when taxSummary is empty
- renders asset table with holdings data
- renders tax lots section when asset is expanded
```

**Add Asset Modal:**
```typescript
- shows "Add Asset Symbol" button
- opens asset modal when button is clicked
- shows symbol input field
- shows name input field
- shows asset type dropdown with STOCK/CRYPTO/ETF/MUTUAL_FUND
- shows currency dropdown with USD/EUR/GBP/JPY/BTC/ETH
- shows sector input field
- auto-detects currency from symbol (.DE → EUR)
- auto-detects currency from symbol (.L → GBP)
- auto-detects currency from symbol (BTC-USD → USD)
- calls onAddAsset when form is submitted with valid data
- does not call onAddAsset when symbol is empty
- closes modal and resets form on Cancel
- closes modal and resets form on successful submit
```

**Log Transaction Modal:**
```typescript
- shows "Log Transaction" button when assets exist
- does NOT show "Log Transaction" button when no assets
- opens transaction modal when button is clicked
- shows asset dropdown with available assets
- shows BUY/SELL type selector
- shows quantity, price, fee, date inputs
- shows price label with asset currency (e.g. "Price (EUR)" for SAP.DE)
- shows price label with portfolio currency as fallback
- calls onAddTransaction when form is submitted
- closes modal on Cancel
```

**Tax Strategy:**
```typescript
- shows FIFO/LIFO/HYBRID selector
- calls setStrategy when option changes
- shows threshold input when HYBRID selected
- hides threshold input when FIFO selected
- calls setThresholdDays when threshold changes
```

**Delete Asset:**
```typescript
- shows delete button for each asset
- calls onDeleteAsset when delete button is clicked
```

**Verification:**
```bash
cd frontend && npx vitest run src/components/PortfolioDetail.test.tsx
```

---

## Task 6: Test Dashboard component

**Files:**
- New: `frontend/src/components/Dashboard.test.tsx`

**What:** Tests for the dashboard with charts. Recharts renders SVG which works in jsdom (no canvas needed).

**Tests:**
```typescript
- renders "No Active Assets Found" when performance/taxSummary is null
- renders "No Active Assets Found" when history is empty
- renders Net Asset Value metric
- renders Total Realized P&L metric
- renders Latent P&L (Unrealized) metric
- renders Total Portfolio Return metric
- renders historical performance chart (AreaChart)
- renders Asset Allocation chart (PieChart)
- renders allocation legend items
- uses taxSummary.currency for formatCurrency
- shows positive class for positive realized P&L
- shows negative class for negative realized P&L
```

**Verification:**
```bash
cd frontend && npx vitest run src/components/Dashboard.test.tsx
```

---

## Task 7: Test App component (integration-level)

**Files:**
- New: `frontend/src/App.test.tsx`

**What:** Higher-level tests for the App component using MSW handlers. Verifies that the full app renders and interacts correctly with mocked API responses.

**Note:** This is the heaviest test file. Consider implementing it last or splitting into multiple focused tests. The App component has many state interactions and fetch calls.

**Tests:**
```typescript
- renders "Investment Portfolio Manager" when no portfolios
- renders portfolio list from MSW mock data
- shows "New Portfolio" button
- shows "Load Demonstration Fund" when no portfolios
- renders PortfolioDetail tab when portfolio is selected
- renders AnalyticsView tab with portfolio currency
- renders dashboard tab
- renders ledger tab with transaction table
- creates portfolio via MSW mock
- handles portfolio selection
- shows loading state
- shows error message on fetch failure
```

**Verification:**
```bash
cd frontend && npx vitest run src/App.test.tsx
```

---

## Task 8: Add coverage script + verify

**Files:**
- Modify: `package.json` (already has `"test:coverage": "vitest run --coverage"`)
- Verify: no additional deps needed (vitest has built-in coverage via `@vitest/coverage-v8`)

**Steps:**
- [ ] Run `npm run test:coverage` to check baseline coverage
- [ ] Verify all tests pass with `npm run test`
- [ ] Verify lint still passes with `npm run lint`
- [ ] Commit all new test files

**Verification:**
```bash
cd frontend && npm run test && npm run lint
```

Expected: all tests pass, lint clean.

---

## File Summary

| File | Type | Priority |
|---|---|---|
| `vite.config.ts` | Config (create) | Task 1 |
| `src/utils/currency.ts` | Utility (create) | Task 3 |
| `src/utils/currency.test.ts` | Unit test | Task 3 |
| `src/utils/formatters.test.ts` | Unit test | Task 2 |
| `src/components/AnalyticsView.test.tsx` | Component test | Task 4 |
| `src/components/PortfolioDetail.test.tsx` | Component test | Task 5 |
| `src/components/Dashboard.test.tsx` | Component test | Task 6 |
| `src/App.test.tsx` | Integration test | Task 7 |

**Estimated test count: ~60-70 tests across 6 test files.**

---

## Dependencies

- Task 1 must be done first (config)
- Task 2 and 3 are independent unit tests (can be done in parallel)
- Tasks 4, 5, 6, 7 depend on Task 1 (DOM environment)
- Task 3 should happen before Task 5 (PortfolioDetail imports the function)
- Task 8 is final verification

## Risks

- **Recharts in jsdom:** Recharts uses SVG which works in jsdom. If tooltip components cause issues, mock them or use `data-testid` queries instead of DOM text matching.
- **App.tsx complexity:** The App component has many hooks, fetch calls, and state. Testing it fully requires careful MSW handler setup. Consider starting with just render tests and adding interaction tests incrementally.
- **CSS variable dependencies:** Components reference CSS custom properties (e.g. `var(--text-secondary)`). jsdom doesn't compute CSS, so queries by text content or `data-testid` are preferred over CSS-dependent assertions.
