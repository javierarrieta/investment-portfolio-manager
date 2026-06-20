import pytest
import numpy as np
import pandas as pd
from datetime import date, datetime
from unittest.mock import AsyncMock, MagicMock, patch
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from app.models import Asset, Transaction, Portfolio, HistoricalPrice, Base
from app.stats_engine import StatsEngine
from app.services.currency_service import CurrencyService

@pytest.mark.asyncio
async def test_calculate_portfolio_performance_multi_currency():
    # 1. Setup in-memory DB
    engine = create_engine("sqlite:///:memory:", connect_args={"check_same_thread": False})
    TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
    Base.metadata.create_all(bind=engine)
    db = TestingSessionLocal()

    try:
        # 2. Setup Portfolio (USD)
        portfolio = Portfolio(id=1, name="Test Portfolio", base_currency="USD")
        db.add(portfolio)
        db.commit()

        # 3. Setup Asset 1: AAPL in USD
        asset1 = Asset(id=1, symbol="AAPL", name="Apple", asset_type="STOCK", currency="USD", portfolio_id=1)
        db.add(asset1)
        
        # 4. Setup Asset 2: SAP in EUR
        asset2 = Asset(id=2, symbol="SAP", name="SAP", asset_type="STOCK", currency="EUR", portfolio_id=1)
        db.add(asset2)
        db.commit()

        # 5. Setup Transactions
        # AAPL: Buy 10 @ 150 USD on 2023-01-01
        tx1 = Transaction(
            id=1, asset_id=1, type="BUY", quantity=10.0, price=150.0, fee=0.0, date=datetime(2023, 1, 1)
        )
        # SAP: Buy 10 @ 100 EUR on 2023-01-01
        tx2 = Transaction(
            id=2, asset_id=2, type="BUY", quantity=10.0, price=100.0, fee=0.0, date=datetime(2023, 1, 1)
        )
        db.add_all([tx1, tx2])
        db.commit()

        # 6. Mock HistoricalPrice
        # We need prices for AAPL and SAP for 2023-01-01
        p1 = HistoricalPrice(symbol="AAPL", date=date(2023, 1, 1), close_price=150.0)
        p2 = HistoricalPrice(symbol="SAP", date=date(2023, 1, 1), close_price=100.0)
        db.add_all([p1, p2])
        db.commit()

        # 7. Mock CurrencyService
        currency_service = AsyncMock(spec=CurrencyService)
        # EUR to USD rate is 1.1
        async def mock_get_rate(from_curr, to_curr, dt):
            if from_curr == "EUR" and to_curr == "USD":
                return 1.1
            return 1.0
        currency_service.get_rate.side_effect = mock_get_rate

        # 8. Call StatsEngine.calculate_portfolio_performance
        # We patch sync_historical_prices to avoid it fetching real data from yfinance
        with patch("app.stats_engine.StatsEngine.sync_historical_prices") as mock_sync:
            results = await StatsEngine.calculate_portfolio_performance(
                db=db,
                assets=[asset1, asset2],
                transactions=[tx1, tx2],
                base_currency="USD",
                currency_service=currency_service
            )

        # 9. Assertions
        # Total value in USD:
        # AAPL: 10 * 150 USD = 1500 USD
        # SAP: 10 * 100 EUR * 1.1 = 1100 USD
        # Total = 2600 USD
        expected_value = 2600.0
        actual_value = results["metrics"]["portfolio_value"]
        
        print(f"\nDEBUG: history={results['history']}")
        print(f"DEBUG: actual_value={actual_value}")
        
        assert actual_value == pytest.approx(expected_value, rel=1e-2)

    finally:
        db.close()

