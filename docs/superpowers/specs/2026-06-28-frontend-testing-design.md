# Frontend Testing Integration Design

## Overview
This document outlines the design for integrating frontend unit and integration tests into the investment-portfolio-manager project, including CI integration.

## Dependencies & Configuration
### Dependencies
- `vitest`, `jsdom` for the test runner and environment.
- `@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event` for component testing.
- `msw` for network interception.
- `openapi-typescript` as a dev dependency for type generation.

### Configuration
1. **Vite Config**: Updated `vite.config.ts` with a `test` block for Vitest (specifying `globals: true` and `environment: 'jsdom'`).
2. **Setup File**: Created `frontend/src/test/setup.ts` to initialize the MSW server and extend Jest-DOM matchers.

## Mocking & Test Structure
### Mocking Strategy
1. **Type Generation**: A `generate-types` script runs `openapi-typescript` against `docs/openapi/openapi.json`, creating a `frontend/src/types/api.d.ts` file.
2. **MSW Handlers**: Created `frontend/src/test/mocks/handlers.ts` where API responses are defined, typed using the generated OpenAPI types.
3. **Server Integration**: The MSW server is started before all tests and reset after each test to ensure isolation.

### Test Structure
- **Unit Tests**: Collocated with components (e.g., `ComponentName.test.tsx`) for logic-heavy components.
- **Integration Tests**: Located in `frontend/src/test/integration/`, focusing on full user flows (e.g., "adding a portfolio asset") and verifying the interaction between components and the mocked API.

## CI Integration
A **Frontend Tests** job is added to `.github/workflows/ci.yml` that runs in parallel to the backend job:
- **Setup**: Install Node.js and cache dependencies with `npm ci`.
- **Type Generation**: Run `npm run generate-types` to create the typed API mocks before testing.
- **Test Execution**: Run `npm run test` to execute the Vitest suite.
