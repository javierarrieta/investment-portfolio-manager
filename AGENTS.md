# Agent Instructions

## Overview
Investment Portfolio Manager: Full-stack app for portfolio management and tax lot (FIFO/LIFO/Hybrid) calculations.

## Architecture
- **Backend**: FastAPI in `backend/`. Uses SQLite (`backend/portfolio.db`) via SQLAlchemy. Dependencies are managed in `backend/.venv/`.
- **Frontend**: React + Vite in `frontend/`.

## Commands

### Run Everything
From root:
```bash
./start.sh
```

### Backend (requires `backend/.venv` activation)
- **Start API**: `uvicorn app.main:app --host 127.0.0.1 --port 8000` (run from `backend/`)
- **Test**: `PYTHONPATH=backend pytest backend/tests/test_tax_engine.py` (run from root)

### Frontend
- **Dev Server**: `npm run dev` (run from `frontend/`)
- **Lint**: `npm run lint` (run from `frontend/`)

## Key Context
- **API Prefix**: All routes are under `/api`.
- **Tax Logic**: `backend/app/tax_engine.py` contains the core `TaxLotEngine` supporting `FIFO`, `LIFO`, and `HYBRID` (using `hybrid_threshold_days`) strategies.
- **Database**: Persistent storage is in `backend/portfolio.db`.
- **Metrics Engine**: `backend/app/stats_engine.py` computes TWR, Volatility, Sharpe Ratio, and Portfolio Beta (vs SPY).
- **macOS Tip**: Use `127.0.0.1` instead of `localhost` for the API to avoid IPv6 resolution issues.

## Roadmap / Current Focus
- **Multi-Currency Support**: Implementing asset currency tracking and exchange rate conversion for tax and stats (see `docs/implementation_plan.md`).
