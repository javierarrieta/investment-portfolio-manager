from pydantic import BaseModel, Field, ConfigDict
from datetime import datetime, date
from typing import List, Optional

# --- Transaction ---
class TransactionBase(BaseModel):
    type: str = Field(..., description="BUY or SELL")
    quantity: float = Field(..., gt=0, description="Amount bought or sold")
    price: float = Field(..., gt=0, description="Price per unit")
    fee: float = Field(0.0, ge=0, description="Brokerage or transaction fee")
    date: datetime = Field(..., description="Timestamp of transaction")

class TransactionCreate(TransactionBase):
    pass

class TransactionOut(TransactionBase):
    id: int
    asset_id: int

    model_config = ConfigDict(from_attributes=True)

# --- Asset ---
class AssetBase(BaseModel):
    symbol: str = Field(..., description="Symbol of the asset, e.g. AAPL, BTC-USD")
    name: str = Field(..., description="Name of the asset")
    asset_type: str = Field(..., description="STOCK, CRYPTO, ETF, MUTUAL_FUND")
    sector: Optional[str] = Field(None, description="Sector of the asset")

class AssetCreate(AssetBase):
    pass

class AssetOut(AssetBase):
    id: int
    portfolio_id: int
    transactions: List[TransactionOut] = []

    model_config = ConfigDict(from_attributes=True)

# --- Portfolio ---
class PortfolioBase(BaseModel):
    name: str = Field(..., description="Name of the portfolio")
    description: Optional[str] = Field(None, description="Optional description")
    currency: str = Field("USD", description="Base currency, e.g. USD, EUR")

class PortfolioCreate(PortfolioBase):
    pass

class PortfolioOut(PortfolioBase):
    id: int
    assets: List[AssetOut] = []

    model_config = ConfigDict(from_attributes=True)

# --- Tax Lot Out ---
class TaxLot(BaseModel):
    buy_date: datetime
    buy_price: float
    original_qty: float
    remaining_qty: float
    latent_gain_loss: float
    latent_roi: float

class AssetTaxSummary(BaseModel):
    symbol: str
    asset_type: str
    current_shares: float
    average_cost: float
    current_price: float
    total_cost: float
    market_value: float
    unrealized_pnl: float
    unrealized_roi: float
    realized_pnl: float
    tax_lots: List[TaxLot]
