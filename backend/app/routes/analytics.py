from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy.orm import Session
from datetime import date, timedelta
from typing import List, Dict, Any
import yfinance as yf
from ..database import get_db
from .. import models
from ..services.currency_service import CurrencyService
from ..tax_engine import TaxLotEngine
from ..stats_engine import StatsEngine

router = APIRouter(
    prefix="/portfolios",
    tags=["analytics"]
)


def get_current_prices(db: Session, symbols: List[str]) -> Dict[str, float]:
    """
    Resolves the current price of a list of symbols, using database cache
    or fetching from yfinance if stale.
    """
    prices = {}
    today = date.today()
    
    # Always guarantee SPY is cached
    symbols_to_check = list(set(symbols + ["SPY"]))
    
    for symbol in symbols_to_check:
        db_price = db.query(models.HistoricalPrice).filter_by(symbol=symbol).order_by(models.HistoricalPrice.date.desc()).first()
        # If cache is valid (price fetched today or yesterday)
        if db_price and db_price.date >= today - timedelta(days=1):
            prices[symbol] = db_price.close_price
        else:
            try:
                ticker = yf.Ticker(symbol)
                current_price = None
                try:
                    current_price = ticker.fast_info.get('lastPrice', None)
                except:
                    pass
                
                if current_price is None or pd.isna(current_price):
                    hist = ticker.history(period="1d")
                    if not hist.empty:
                        current_price = hist['Close'].iloc[-1]
                
                if current_price is not None:
                    current_price = float(current_price)
                    prices[symbol] = current_price
                    
                    # Update cache
                    existing = db.query(models.HistoricalPrice).filter_by(symbol=symbol, date=today).first()
                    if existing:
                        existing.close_price = current_price
                    else:
                        db_price = models.HistoricalPrice(symbol=symbol, date=today, close_price=current_price)
                        db.add(db_price)
                    db.commit()
                else:
                    prices[symbol] = db_price.close_price if db_price else 0.0
            except Exception as e:
                print(f"Error fetching current price for {symbol}: {e}")
                prices[symbol] = db_price.close_price if db_price else 0.0
    return prices


@router.get("/{portfolio_id}/tax-summary", response_model=Dict[str, Any])
async def get_portfolio_tax_summary(
    portfolio_id: int,
    strategy: str = Query("FIFO", description="Tax matching strategy: FIFO, LIFO, or HYBRID"),
    threshold_days: int = Query(30, description="Short-term holding threshold in days for HYBRID strategy"),
    db: Session = Depends(get_db)
):
    portfolio = db.query(models.Portfolio).filter(models.Portfolio.id == portfolio_id).first()
    if not portfolio:
        raise HTTPException(status_code=404, detail="Portfolio not found")

    if not portfolio.assets:
        return {"assets": [], "total_portfolio_value": 0.0, "total_realized_pnl": 0.0, "total_unrealized_pnl": 0.0}

    symbols = [a.symbol for a in portfolio.assets]
    import pandas as pd # Import pandas locally just in case it is used by pd.isna
    current_prices = get_current_prices(db, symbols)

    currency_service = CurrencyService()
    asset_summaries = []
    total_value = 0.0
    total_realized = 0.0
    total_unrealized = 0.0

    for asset in portfolio.assets:
        price = current_prices.get(asset.symbol, 0.0)
        summary = await TaxLotEngine.calculate_lots(
            symbol=asset.symbol,
            asset_type=asset.asset_type,
            transactions=asset.transactions,
            current_price=price,
            asset_currency=asset.currency,
            base_currency=portfolio.base_currency,
            currency_service=currency_service,
            strategy=strategy,
            hybrid_threshold_days=threshold_days
        )
        asset_summaries.append(summary)
        total_value += summary["market_value"]
        total_realized += summary["realized_pnl"]
        total_unrealized += summary["unrealized_pnl"]

    return {
        "assets": asset_summaries,
        "total_portfolio_value": total_value,
        "total_realized_pnl": total_realized,
        "total_unrealized_pnl": total_unrealized,
        "strategy": strategy,
        "threshold_days": threshold_days
    }


@router.get("/{portfolio_id}/performance", response_model=Dict[str, Any])
async def get_portfolio_performance(portfolio_id: int, db: Session = Depends(get_db)):
    portfolio = db.query(models.Portfolio).filter(models.Portfolio.id == portfolio_id).first()
    if not portfolio:
        raise HTTPException(status_code=404, detail="Portfolio not found")

    # Fetch transactions and assets
    assets = portfolio.assets
    asset_ids = [a.id for a in assets]
    transactions = db.query(models.Transaction).filter(models.Transaction.asset_id.in_(asset_ids)).all()

    import pandas as pd # Ensure pandas is imported

    currency_service = CurrencyService()
    perf_data = await StatsEngine.calculate_portfolio_performance(
        db, 
        assets, 
        transactions, 
        base_currency=portfolio.base_currency, 
        currency_service=currency_service
    )
    
    # Add tax realized/unrealized P&L to metrics using default FIFO for high-level dashboard
    symbols = [a.symbol for a in assets]
    prices = get_current_prices(db, symbols)
    
    realized_total = 0.0
    unrealized_total = 0.0
    for asset in assets:
        price = prices.get(asset.symbol, 0.0)
        summary = await TaxLotEngine.calculate_lots(
            symbol=asset.symbol,
            asset_type=asset.asset_type,
            transactions=asset.transactions,
            current_price=price,
            asset_currency=asset.currency,
            base_currency=portfolio.base_currency,
            currency_service=currency_service,
            strategy="FIFO"
        )
        realized_total += summary["realized_pnl"]
        unrealized_total += summary["unrealized_pnl"]

    perf_data["metrics"]["realized_pnl"] = realized_total
    perf_data["metrics"]["unrealized_pnl"] = unrealized_total

    return perf_data
