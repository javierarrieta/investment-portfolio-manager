# Agent Instructions

## Overview
Investment Portfolio Manager: Full-stack app for portfolio management and tax lot (FIFO/LIFO/Hybrid) calculations.

## Architecture
- **Backend**: Rust (Rocket) in `backend_rust/`. Uses SQLite (`backend/portfolio.db`) via SQLx.
- **Frontend**: React + Vite in `frontend/`.

## Commands

### Run Everything
From root:
```bash
./start.sh
```

### Backend
- **Start API**: `cargo run` (run from `backend_rust/`)
- **Test**: `cargo test` (run from `backend_rust/`)
- **Update OpenAPI Spec**: When API routes or schemas change, regenerate and save:
  ```bash
  cd backend_rust
  touch portfolio.db
  cargo run --release &
  sleep 12
  curl -s http://127.0.0.1:8000/api-docs/openapi.json > ../docs/openapi/openapi.json
  kill $(jobs -p)
  ```

### Frontend
- **Dev Server**: `npm run dev` (run from `frontend/`)
- **Lint**: `npm run lint` (run from `frontend/`)

## Key Context
- **API Prefix**: All routes are under `/api`.
- **Tax Logic**: `backend_rust/src/engines/tax_engine.rs` contains the core `TaxLotEngine` supporting `FIFO`, `LIFO`, and `HYBRID` (using `hybrid_threshold_days`) strategies.
- **Database**: Persistent storage is in `backend/portfolio.db`.
- **Metrics Engine**: `backend_rust/src/engines/stats_engine.rs` computes TWR, Volatility, Sharpe Ratio, and Portfolio Beta (vs SPY).
- **macOS Tip**: Use `127.0.0.1` instead of `localhost` for the API to avoid IPv6 resolution issues.

## Branching Workflow

- **Always create a new branch** for each feature, fix, or change. Never work on `main`.
- Name branches descriptively using the `type/scope` convention:
  - `feat/...` — new features or functionality
  - `fix/...` — bug fixes
  - `chore/...` — configuration, tooling, docs
  - `refactor/...` — code refactoring with no behavior change
- After finishing work, commit your changes. The user will create the PR and merge manually.

## Roadmap / Current Focus
- **Multi-Currency Support**: Implementing asset currency tracking and exchange rate conversion for tax and stats (see `docs/implementation_plan.md`).
