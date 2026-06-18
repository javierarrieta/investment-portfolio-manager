from sqlalchemy import Column, Integer, String, Float, DateTime, ForeignKey, Date, UniqueConstraint
from sqlalchemy.orm import relationship
from .database import Base

class Portfolio(Base):
    __tablename__ = "portfolios"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, nullable=False)
    description = Column(String, nullable=True)
    currency = Column(String, default="USD")
    base_currency = Column(String, default="USD")

    assets = relationship("Asset", back_populates="portfolio", cascade="all, delete-orphan")


class Asset(Base):
    __tablename__ = "assets"

    id = Column(Integer, primary_key=True, index=True)
    portfolio_id = Column(Integer, ForeignKey("portfolios.id", ondelete="CASCADE"), nullable=False)
    symbol = Column(String, nullable=False, index=True)
    name = Column(String, nullable=False)
    asset_type = Column(String, nullable=False)  # STOCK, CRYPTO, ETF, MUTUAL_FUND
    sector = Column(String, nullable=True)
    currency = Column(String, default="USD")

    portfolio = relationship("Portfolio", back_populates="assets")
    transactions = relationship("Transaction", back_populates="asset", cascade="all, delete-orphan")

    __table_args__ = (
        UniqueConstraint("portfolio_id", "symbol", name="_portfolio_symbol_uc"),
    )


class Transaction(Base):
    __tablename__ = "transactions"

    id = Column(Integer, primary_key=True, index=True)
    asset_id = Column(Integer, ForeignKey("assets.id", ondelete="CASCADE"), nullable=False)
    type = Column(String, nullable=False)  # BUY, SELL
    quantity = Column(Float, nullable=False)
    price = Column(Float, nullable=False)
    fee = Column(Float, default=0.0)
    date = Column(DateTime, nullable=False, index=True)  # Transaction timestamp

    asset = relationship("Asset", back_populates="transactions")


class HistoricalPrice(Base):
    __tablename__ = "historical_prices"

    symbol = Column(String, primary_key=True, index=True)
    date = Column(Date, primary_key=True, index=True)
    close_price = Column(Float, nullable=False)
