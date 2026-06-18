import pytest
from datetime import datetime, timedelta
from app.tax_engine import TaxLotEngine

# Mock transaction database class for testing
class MockTransaction:
    def __init__(self, type: str, quantity: float, price: float, fee: float, date: datetime):
        self.type = type
        self.quantity = quantity
        self.price = price
        self.fee = fee
        self.date = date

def test_fifo_matching():
    # Buy 10 AAPL at 100 on Jan 1st (fee = 10)
    # Buy 5 AAPL at 120 on Jan 5th (fee = 5)
    # Sell 12 AAPL at 150 on Jan 10th (fee = 12)
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 10.0, datetime(2026, 1, 1)),
        MockTransaction("BUY", 5.0, 120.0, 5.0, datetime(2026, 1, 5)),
        MockTransaction("SELL", 12.0, 150.0, 12.0, datetime(2026, 1, 10))
    ]
    
    # FIFO Calculation:
    # First lot: 10 units. Cost basis including fee = (10 * 100 + 10) / 10 = 101.
    # Second lot: 5 units. Cost basis including fee = (5 * 120 + 5) / 5 = 121.
    # Sell: 12 units at 150. Net proceeds per unit = (12 * 150 - 12) / 12 = 149.
    # FIFO matches:
    # - 10 units from Lot 1 (cost = 101, proceeds = 149, gain = 10 * 48 = 480)
    # - 2 units from Lot 2 (cost = 121, proceeds = 149, gain = 2 * 28 = 56)
    # Total Realized P&L = 480 + 56 = 536.
    # Remaining: 3 units of Lot 2 (cost basis = 121, market price = 160).
    
    summary = TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0,
        strategy="FIFO"
    )
    
    assert summary["realized_pnl"] == pytest.approx(536.0)
    assert summary["current_shares"] == pytest.approx(3.0)
    assert summary["total_cost"] == pytest.approx(3 * 121.0)
    assert summary["market_value"] == pytest.approx(3 * 160.0)
    assert summary["unrealized_pnl"] == pytest.approx(3 * (160 - 121))


def test_lifo_matching():
    # Buy 10 AAPL at 100 on Jan 1st (fee = 10)
    # Buy 5 AAPL at 120 on Jan 5th (fee = 5)
    # Sell 12 AAPL at 150 on Jan 10th (fee = 12)
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 10.0, datetime(2026, 1, 1)),
        MockTransaction("BUY", 5.0, 120.0, 5.0, datetime(2026, 1, 5)),
        MockTransaction("SELL", 12.0, 150.0, 12.0, datetime(2026, 1, 10))
    ]
    
    # LIFO Calculation:
    # First lot (Jan 5th): 5 units. Cost = 121. Proceeds = 149. Gain = 5 * 28 = 140.
    # Second lot (Jan 1st): 7 units. Cost = 101. Proceeds = 149. Gain = 7 * 48 = 336.
    # Total Realized P&L = 140 + 336 = 476.
    # Remaining: 3 units of Lot 1 (cost basis = 101).
    
    summary = TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0,
        strategy="LIFO"
    )
    
    assert summary["realized_pnl"] == pytest.approx(476.0)
    assert summary["current_shares"] == pytest.approx(3.0)
    assert summary["total_cost"] == pytest.approx(3 * 101.0)
    assert summary["unrealized_pnl"] == pytest.approx(3 * (160 - 101))


def test_hybrid_matching():
    # Buy 10 AAPL at 100 on Jan 1st (fee = 0)  - Long term if sold on Feb 15th (>30 days)
    # Buy 5 AAPL at 120 on Feb 10th (fee = 0)  - Short term if sold on Feb 15th (5 days)
    # Sell 7 AAPL at 150 on Feb 15th (fee = 0)
    txs = [
        MockTransaction("BUY", 10.0, 100.0, 0.0, datetime(2026, 1, 1)),
        MockTransaction("BUY", 5.0, 120.0, 0.0, datetime(2026, 2, 10)),
        MockTransaction("SELL", 7.0, 150.0, 0.0, datetime(2026, 2, 15))
    ]
    
    # Hybrid strategy (30 days threshold):
    # Feb 15th sell.
    # - Lot 2 (Feb 10th) is short term (5 days age <= 30). Matches first (LIFO).
    # - Lot 1 (Jan 1st) is long term (45 days age > 30). Matches next (FIFO).
    # We sell 7 units:
    # - 5 units from Lot 2 (cost = 120, proceeds = 150, gain = 5 * 30 = 150)
    # - 2 units from Lot 1 (cost = 100, proceeds = 150, gain = 2 * 50 = 100)
    # Total Realized P&L = 150 + 100 = 250.
    # Remaining: 8 units of Lot 1 (cost = 100).
    
    summary = TaxLotEngine.calculate_lots(
        symbol="AAPL",
        asset_type="STOCK",
        transactions=txs,
        current_price=160.0,
        strategy="HYBRID",
        hybrid_threshold_days=30
    )
    
    assert summary["realized_pnl"] == pytest.approx(250.0)
    assert summary["current_shares"] == pytest.approx(8.0)
    assert summary["total_cost"] == pytest.approx(8 * 100.0)
