# Frontend Testing Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Steps use checkbox (- []) syntax for tracking.

**Goal:** Set up Vitest, React Testing Library, and MSW for frontend testing with OpenAPI-based mocks, and integrate tests into the CI pipeline.

**Architecture:**
- Vitest runs tests in `jsdom` environment.
- MSW intercepts network requests in `setup.ts` and uses handlers typed by `openapi-typescript`.
- `openapi-typescript` generates types from `docs/openapi/openapi.json` before tests run.
- CI runs `npm run generate-types` then `npm test`.

**Tech Stack:**
- Vitest, React Testing Library, MSW, openapi-typescript, Node.js, GitHub Actions.

## Global Constraints
- Use `vitest`, `jsdom`, `@testing-library/react`, `msw`, `openapi-typescript`.
- Mocks must be typed using generated OpenAPI types.
- Tests must run in CI via `npm test` after `npm run generate-types`.

---

### Task 1: Add dependencies and scripts

**Files:**
- Modify: `frontend/package.json`

**Interfaces:**
- Adds `devDependencies`: `vitest`, `jsdom`, `@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event`, `msw`, `@mswjs/http-middleware`, `openapi-typescript`.
- Adds scripts: `test`, `test:coverage`, `generate-types`.

- [ ] **Step 1: Update package.json**

Add the following to `frontend/package.json`:

```json
"devDependencies": {
  "vitest": "^1.6.0",
  "jsdom": "^24.1.0",
  "@testing-library/react": "^14.2.0",
  "@testing-library/jest-dom": "^6.4.0",
  "@testing-library/user-event": "^14.5.0",
  "msw": "^2.4.0",
  "@mswjs/http-middleware": "^0.10.0",
  "openapi-typescript": "^6.7.0",
  "typescript": "^5.4.0"
},
"scripts": {
  "test": "vitest run",
  "test:coverage": "vitest run --coverage",
  "generate-types": "openapi-typescript docs/openapi/openapi.json --output src/types/api.d.ts"
}
```

- [ ] **Step 2: Run install**

Run: `cd frontend && npm install`

---

### Task 2: Configure Vitest

**Files:**
- Modify: `frontend/vite.config.js`

**Interfaces:**
- Consumes: Vite config structure.
- Produces: Vitest config with `jsdom` environment and test setup file.

- [ ] **Step 1: Update vite.config.js**

Modify `frontend/vite.config.js` to include the test configuration:

```javascript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts'
  }
})
```

---

### Task 3: Set up test environment and MSW

**Files:**
- Create: `frontend/src/test/setup.ts`
- Create: `frontend/src/test/mocks/handlers.ts`

**Interfaces:**
- Consumes: MSW server instance.
- Produces: Global test setup with mocked API handlers.

- [ ] **Step 1: Create setup.ts**

Create `frontend/src/test/setup.ts`:

```typescript
import '@testing-library/jest-dom/vitest'
import { afterAll, afterEach, beforeAll } from 'vitest'
import { server } from './mocks/server'

beforeAll(() => server.listen({ onUnhandledRequest: 'bypass' }))
afterEach(() => server.resetHandlers())
afterAll(() => server.close())
```

- [ ] **Step 2: Create mocks/server.ts**

Create `frontend/src/test/mocks/server.ts`:

```typescript
import { setupServer } from 'msw/node'
import { handlers } from './handlers'

export const server = setupServer(...handlers)
```

- [ ] **Step 3: Create mocks/handlers.ts**

Create `frontend/src/test/mocks/handlers.ts` with handlers based on OpenAPI spec:

```typescript
import { http, HttpResponse } from 'msw'
import type { Portfolio, PortfolioOut, PortfolioCreate, AssetOut, AssetCreate } from '@/types/api'

export const handlers = [
  http.get('/api/portfolios', () => {
    const mockPortfolios: PortfolioOut[] = [
      { id: 1, name: 'Test Portfolio', description: 'A test portfolio', base_currency: 'USD' },
      { id: 2, name: 'Retirement Fund', description: 'Retirement savings', base_currency: 'USD' }
    ]
    return HttpResponse.json(mockPortfolios)
  }),
  http.get('/api/portfolios/:id', ({ params }) => {
    const id = Number(params.id)
    const mockPortfolio: PortfolioOut = {
      id,
      name: `Portfolio ${id}`,
      description: `Description for portfolio ${id}`,
      base_currency: 'USD',
      assets: [
        { id: 1, portfolio_id: id, symbol: 'AAPL', name: 'Apple Inc.', asset_type: 'Stock', sector: 'Technology', currency: 'USD', transactions: [] }
      ]
    }
    return HttpResponse.json(mockPortfolio)
  }),
  http.post('/api/portfolios', async ({ request }) => {
    const body = await request.json() as PortfolioCreate
    const newPortfolio: PortfolioOut = {
      id: 3,
      name: body.name,
      description: body.description,
      base_currency: body.base_currency || 'USD',
      assets: []
    }
    return HttpResponse.json(newPortfolio, { status: 201 })
  }),
  http.get('/api/assets/:id', ({ params }) => {
    const id = Number(params.id)
    const mockAsset: AssetOut = {
      id,
      portfolio_id: 1,
      symbol: 'AAPL',
      name: 'Apple Inc.',
      asset_type: 'Stock',
      sector: 'Technology',
      currency: 'USD',
      transactions: []
    }
    return HttpResponse.json(mockAsset)
  })
]
```

---

### Task 4: Create example tests

**Files:**
- Create: `frontend/src/test/example/PortfolioList.test.tsx`
- Create: `frontend/src/test/example/PortfolioForm.test.tsx`

**Interfaces:**
- Consumes: MSW handlers, React components.
- Produces: Verified component behavior.

- [ ] **Step 1: Create PortfolioList.test.tsx**

Create `frontend/src/test/example/PortfolioList.test.tsx`:

```typescript
import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { PortfolioList } from '@/components/PortfolioList'

describe('PortfolioList', () => {
  it('renders a list of portfolios', async () => {
    render(<PortfolioList />)
    
    await waitFor(() => {
      expect(screen.getByText('Test Portfolio')).toBeInTheDocument()
      expect(screen.getByText('Retirement Fund')).toBeInTheDocument()
    })
  })

  it('shows loading state initially', () => {
    render(<PortfolioList />)
    expect(screen.getByText('Loading...')).toBeInTheDocument()
  })
})
```

- [ ] **Step 2: Create PortfolioForm.test.tsx**

Create `frontend/src/test/example/PortfolioForm.test.tsx`:

```typescript
import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { PortfolioForm } from '@/components/PortfolioForm'

describe('PortfolioForm', () => {
  it('submits a new portfolio', async () => {
    const user = userEvent.setup()
    render(<PortfolioForm />)
    
    await user.type(screen.getByLabelText(/name/i), 'New Portfolio')
    await user.type(screen.getByLabelText(/description/i), 'A new portfolio')
    
    await waitFor(() => {
      expect(screen.getByText('Portfolio created!')).toBeInTheDocument()
    })
  })
})
```

---

### Task 5: Update CI pipeline

**Files:**
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: Existing CI structure.
- Produces: CI pipeline with frontend tests job.

- [ ] **Step 1: Add Node.js job to CI**

Add the following job to `.github/workflows/ci.yml`:

```yaml
  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json

      - name: Install dependencies
        run: npm ci
        working-directory: frontend

      - name: Generate API types
        run: npm run generate-types
        working-directory: frontend

      - name: Run tests
        run: npm test
        working-directory: frontend
```

Place this job next to the existing `build-and-test` job.

---

### Task 6: Run tests and verify

**Files:**
- N/A

**Interfaces:**
- Consumes: All previous tasks.
- Produces: Verified test suite.

- [ ] **Step 1: Run tests locally**

Run: `cd frontend && npm test`

Expected: All tests pass.

- [ ] **Step 2: Verify CI**

Push changes to a branch and check GitHub Actions.

Expected: `frontend-tests` job passes.
