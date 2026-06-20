import pytest
from datetime import datetime, timedelta
from app.tax_engine import TaxLotEngine

# Mock transaction database class for testing
class MockTransaction:
    def __init__(self, type: str, quantity: float, price: float, fee: float, date: datetime, currency: str = "USD"):
        self.type = type
        self.quantity = quantity
        self.price = price
        self.fee = fee
        self.date = date
        self.currency = currency

class MockCurrencyService:
    def __init__(self, rates: dict = None):
        self.rates = rates or {}

    async def get_rate(self, from_curr: str, to_curr: str, date: datetime) -> float:
        if from_curr == to_curr:
            return 1.0
        key = (from_curr, to_curr)
        return self.rates.get(key, 1.0)

@pytest.mark.asyncio
async def test_fifo_matching():
    # Buy 10 AAPL at 100 on Jan 1st (fee = 10)
    # Buy 5 AAPL at 120 on Jan 5th (fee = 5)
    # Sell 12 AAPL at 150 on Jan 10th (fee = 12)
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 10.0, datetime(2026, 1, 1)),
        MockTransaction("BUY", 5.0, 120.0, 5.0, datetime(2026, 1, 5)),
        MockTransaction("SELL", 12.0, 150.0, 12.0, datetime(2026, 1, 10))
    ]
    
    currency_service = MockCurrencyService()
    
    summary = await TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0,
        asset_currency="USD",
        base_currency="USD",
        currency_service=currency_service,
        strategy="FIFO"
    )
    
    assert summary["realized_pnl"] == pytest.approx(536.0)
    assert summary["current_shares"] == pytest.approx(3.0)
    assert summary["total_cost"] == pytest.approx(3 * 121.0)
    assert summary["market_value"] == pytest.approx(3 * 160.0)
    assert summary["unrealized_pnl"] == pytest.approx(3 * (160 - 121))


@pytest.mark.asyncio
async def test_lifo_matching():
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 10.0, datetime(2026, 1, 1)),
        MockTransaction("BUY", 5.0, 120.0, 5.0, datetime(2026, 1, 5)),
        MockTransaction("SELL", 12.0, 150.0, 12.0, datetime(2026, 1, 10))
    ]
    
    currency_service = MockCurrencyService()

    summary = await TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0,
        asset_currency="USD",
        base_currency="USD",
        currency_service=currency_service,
        strategy="LIFO"
    )
    
    assert summary["realized_pnl"] == pytest.approx(476.0)
    assert summary["current_shares"] == pytest.approx(3.0)
    assert summary["total_cost"] == pytest.approx(3 * 101.0)
    assert summary["unrealized_pnl"] == pytest.approx(3 * (160 - 101))


@pytest.mark.asyncio
async def test_hybrid_matching():
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 0.0, datetime(2026, 1, 1)),
        MockTransaction("BUY", 5.0, 120.0, 0.0, datetime(2026, 2, 10)),
        MockTransaction("SELL", 7.0, 150.0, 0.0, datetime(2026, 2, 15))
    ]
    
    currency_service = MockCurrencyService()

    summary = await TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0,
        asset_currency="USD",
        base_currency="USD",
        currency_service=currency_service,
        strategy="HYBRID",
        hybrid_threshold_days=30
    )
    
    assert summary["realized_pnl"] == pytest.approx(250.0)
    assert summary["current_shares"] == pytest.approx(8.0)
    assert summary["total_cost"] == pytest.approx(8 * 100.0)

@pytest.mark.asyncio
async def test_multi_currency_fifo():
    # Buy 10 AAPL (USD) at 100, Sell 5 AAPL (USD) at 150. Portfolio is EUR.
    # Mock CurrencyService to return EURUSD = 0.9
    # Expected realized P&L in EUR = (5 * 150 * 0.9) - (5 * 100 * 0.9) = 225 EUR
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 0.0, datetime(2026, 1, 1), currency="USD"),
        MockTransaction("SELL", 5.0, 150.0, 0.0, datetime(2026, 1, 10), currency="USD")
    ]
    
    # Rate: 1 USD = 0.9 EUR
    currency_service = MockCurrencyService(rates={("USD", "EUR"): 0.9})
    
    summary = await TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0, # USD
        asset_currency="USD",
        base_currency="EUR",
        currency_service=currency_service,
        strategy="FIFO"
    )
    
    # Realized P&L:
    # Buy: 10 * 100 = 1000 USD = 900 EUR
    # Sell: 5 * 150 = 750 USD = 675 EUR
    # Cost of 5 units: 5 * 100 = 500 USD = 450 EUR
    # Realized P&L = 675 - 450 = 225 EUR
    assert summary["realized_pnl"] == pytest.approx(225.0)
    # Current price in EUR: 160 * 0.9 = 144 EUR
    # Current shares: 5
    # Market value: 5 * 144 = 720 EUR
    # Total cost (remaining 5 units): 5 * 100 * 0.9 = 450 EUR
    # Unrealized P&L: 720 - 450 = 270 EUR
    assert summary["market_value"] == pytest.approx(720.0)
    assert summary["unrealized_pnl"] == pytest.approx(270.0)
