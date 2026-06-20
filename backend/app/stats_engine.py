import numpy as np
import pandas as pd
from datetime import date, datetime, timedelta
from typing import List, Dict, Any, Tuple
from sqlalchemy.orm import Session
from .models import HistoricalPrice, Transaction, Asset
import yfinance as yf
from .services.currency_service import CurrencyService

class StatsEngine:
    @staticmethod
    def sync_historical_prices(db: Session, symbols: List[str], start_date: date) -> None:
        """
        Fetches historical prices for the symbols from start_date to today using yfinance,
        and saves them to the SQLite database.
        """
        if not symbols:
            return

        today = date.today()
        # We always want a bit of extra data for benchmarks or return calculations
        start_str = start_date.strftime("%Y-%m-%d")
        end_str = (today + timedelta(days=1)).strftime("%Y-%m-%d")

        # Include SPY as the default market benchmark
        symbols_to_fetch = list(set(symbols + ["SPY"]))

        try:
            # yf.download is very fast for multiple tickers
            data = yf.download(symbols_to_fetch, start=start_str, end=end_str, group_by='ticker')
            
            for symbol in symbols_to_fetch:
                if symbol not in data.columns.levels[0] if isinstance(data.columns, pd.MultiIndex) else [symbol]:
                    # Single ticker returned or missing
                    if len(symbols_to_fetch) == 1:
                        ticker_df = data
                    else:
                        continue
                else:
                    ticker_df = data[symbol]

                if ticker_df.empty:
                    continue

                for timestamp, row in ticker_df.iterrows():
                    close_price = row.get("Close")
                    if pd.isna(close_price) or close_price is None:
                        continue
                    
                    price_date = timestamp.date()
                    
                    # Insert or update in database
                    db_price = db.query(HistoricalPrice).filter_by(symbol=symbol, date=price_date).first()
                    if db_price:
                        db_price.close_price = float(close_price)
                    else:
                        db_price = HistoricalPrice(
                            symbol=symbol,
                            date=price_date,
                            close_price=float(close_price)
                        )
                        db.add(db_price)
            db.commit()
        except Exception as e:
            db.rollback()
            print(f"Error syncing historical prices: {e}")

    @staticmethod
    def get_historical_price_matrix(db: Session, symbols: List[str], start_date: date, end_date: date) -> pd.DataFrame:
        """
        Retrieves historical prices from the local DB cache and returns a Pandas DataFrame
        indexed by date with symbols as columns. Fills missing values with forward-fill.
        """
        symbols_to_query = list(set(symbols + ["SPY"]))
        prices = db.query(HistoricalPrice).filter(
            HistoricalPrice.symbol.in_(symbols_to_query),
            HistoricalPrice.date >= start_date,
            HistoricalPrice.date <= end_date
        ).all()

        if not prices:
            # Return empty DataFrame with dates
            delta = end_date - start_date
            dates = [start_date + timedelta(days=i) for i in range(delta.days + 1)]
            return pd.DataFrame(index=dates, columns=symbols_to_query).fillna(0.0)

        # Convert to records
        records = [{"symbol": p.symbol, "date": p.date, "close_price": p.close_price} for p in prices]
        df = pd.DataFrame(records)

        # Pivot to dates as index, symbols as columns
        df_pivot = df.pivot(index="date", columns="symbol", values="close_price")

        # Generate a complete date index to fill any gaps (weekends, holidays)
        idx = pd.date_range(start=start_date, end=end_date).date
        df_pivot = df_pivot.reindex(idx)
        
        # Forward-fill (previous day price if weekend/holiday) and then backward-fill (if start of series has NaNs)
        df_pivot = df_pivot.ffill().bfill()
        return df_pivot

    @staticmethod
    async def calculate_portfolio_performance(
        db: Session,
        assets: List[Asset],
        transactions: List[Transaction],
        base_currency: str,
        currency_service: CurrencyService
    ) -> Dict[str, Any]:
        """
        Computes historical daily value of the portfolio and cumulative Time-Weighted Return (TWR).
        Also calculates current stats like Volatility, Sharpe Ratio, Beta relative to SPY, and Correlation.
        """
        if not assets or not transactions:
            return {
                "history": [],
                "correlation_matrix": {},
                "metrics": {
                    "volatility": 0.0,
                    "sharpe_ratio": 0.0,
                    "beta": 1.0,
                    "portfolio_value": 0.0,
                    "unrealized_pnl": 0.0,
                    "realized_pnl": 0.0,
                    "beta_adjusted_exposure": 0.0
                }
            }

        # 1. Determine date range
        tx_dates = [tx.date for tx in transactions]
        start_date = min(tx_dates).date()
        end_date = date.today()

        symbols = [a.symbol for a in assets]
        asset_id_to_asset = {a.id: a for a in assets}
        
        # Sync and retrieve prices
        StatsEngine.sync_historical_prices(db, symbols, start_date)
        price_df = StatsEngine.get_historical_price_matrix(db, symbols, start_date, end_date)

        # Create maps for asset symbol to ID
        asset_id_to_symbol = {a.id: a.symbol for a in assets}

        # 2. Build daily asset quantities
        # Index dates
        dates = list(price_df.index)
        qty_dict = {symbol: np.zeros(len(dates)) for symbol in symbols}

        # Sort transactions chronologically
        sorted_txs = sorted(transactions, key=lambda x: x.date)

        for symbol in symbols:
            current_qty = 0.0
            symbol_txs = [t for t in sorted_txs if asset_id_to_symbol.get(t.asset_id) == symbol]
            tx_idx = 0
            for i, d in enumerate(dates):
                while tx_idx < len(symbol_txs) and symbol_txs[tx_idx].date.date() <= d:
                    tx = symbol_txs[tx_idx]
                    if tx.type.upper() == "BUY":
                        current_qty += tx.quantity
                    elif tx.type.upper() == "SELL":
                        current_qty = max(0.0, current_qty - tx.quantity)
                    tx_idx += 1
                qty_dict[symbol][i] = current_qty

        # 3. Calculate daily portfolio values and cash flows
        portfolio_values = np.zeros(len(dates))
        daily_cash_flow = np.zeros(len(dates))  # Purchases (buys) are additions, sales are withdrawals (but let's track net flow)
        
        # Cash flow: buy cost = qty * price + fee (money injected), sell proceeds = qty * price - fee (money taken out)
        for tx in sorted_txs:
            tx_date = tx.date.date()
            if tx_date in dates:
                idx = dates.index(tx_date)
                asset = asset_id_to_asset.get(tx.asset_id)
                if not asset:
                    continue
                
                cost = tx.quantity * tx.price
                fee = tx.fee
                
                if asset.currency != base_currency:
                    rate = await currency_service.get_rate(asset.currency, base_currency, tx.date)
                    cost *= rate
                    fee *= rate
                
                if tx.type.upper() == "BUY":
                    # Cash inflow to the assets
                    daily_cash_flow[idx] += (cost + fee)
                elif tx.type.upper() == "SELL":
                    # Cash outflow from the assets
                    daily_cash_flow[idx] -= (cost - fee)

        for i, d in enumerate(dates):
            val = 0.0
            for asset in assets:
                symbol = asset.symbol
                price = price_df.loc[d, symbol] if symbol in price_df.columns else 0.0
                qty = qty_dict[symbol][i]
                
                if asset.currency != base_currency:
                    rate = await currency_service.get_rate(asset.currency, base_currency, d)
                    price *= rate
                val += qty * price
            portfolio_values[i] = val

        # 4. Calculate Time-Weighted Returns (TWR)
        # Daily return R_t = (V_t - (V_t-1 + CF_t)) / (V_t-1 + CF_t)
        # TWR cumulative = product(1 + R_t) - 1
        daily_returns = np.zeros(len(dates))
        twr_cumulative = np.zeros(len(dates))
        twr_acc = 1.0

        for i in range(len(dates)):
            if i == 0:
                # First day return is 0 or based on first day cash flow
                daily_returns[i] = 0.0
            else:
                prev_val = portfolio_values[i-1]
                curr_val = portfolio_values[i]
                cf = daily_cash_flow[i]
                
                # Base is the value before the current day change, plus any capital added
                # If we inject CF_t on day t, the new baseline is prev_val + CF_t
                base = prev_val + cf
                if base > 0:
                    # R_t is the return on that day's asset movements
                    daily_returns[i] = (curr_val - base) / base
                else:
                    daily_returns[i] = 0.0

            twr_acc *= (1.0 + daily_returns[i])
            twr_cumulative[i] = twr_acc - 1.0

        # Build history list of objects
        history = []
        for i, d in enumerate(dates):
            history.append({
                "date": d.strftime("%Y-%m-%d"),
                "value": float(portfolio_values[i]),
                "daily_return": float(daily_returns[i]),
                "twr": float(twr_cumulative[i]),
                "cash_flow": float(daily_cash_flow[i])
            })

        # 5. Advanced Stats
        # Returns DataFrame for assets and SPY benchmark
        pct_returns = price_df.pct_change().dropna()

        # Correlation Matrix (of active assets)
        corr_matrix = {}
        if len(symbols) > 1:
            try:
                # Select only the symbols in the portfolio
                active_asset_returns = pct_returns[symbols]
                corr_df = active_asset_returns.corr()
                corr_matrix = corr_df.to_dict()
            except Exception as e:
                print(f"Error calculating correlation matrix: {e}")

        # Benchmark SPY returns
        spy_col = "SPY"
        portfolio_return_series = pd.Series(daily_returns, index=dates).reindex(pct_returns.index)
        
        # Calculate Volatility (annualized, 252 days)
        # Using the portfolio returns
        vol = 0.0
        if len(portfolio_return_series) > 1:
            vol = float(portfolio_return_series.std() * np.sqrt(252))

        # Calculate Sharpe Ratio (risk-free rate = 2%)
        risk_free_rate = 0.02
        sharpe = 0.0
        if vol > 0:
            # Annualized return from cumulative return
            total_days = (end_date - start_date).days
            if total_days > 0:
                annualized_return = (1.0 + twr_cumulative[-1]) ** (365.0 / total_days) - 1.0
                sharpe = float((annualized_return - risk_free_rate) / vol)

        # Calculate Beta relative to SPY
        beta = 1.0
        if spy_col in pct_returns.columns and len(portfolio_return_series) > 1:
            try:
                spy_returns = pct_returns[spy_col]
                # Align series
                aligned_df = pd.DataFrame({
                    "portfolio": portfolio_return_series,
                    "spy": spy_returns
                }).dropna()
                
                if len(aligned_df) > 1:
                    covariance = np.cov(aligned_df["portfolio"], aligned_df["spy"])[0][1]
                    spy_variance = np.var(aligned_df["spy"])
                    if spy_variance > 0:
                        beta = float(covariance / spy_variance)
            except Exception as e:
                print(f"Error calculating Beta: {e}")

        current_portfolio_value = portfolio_values[-1] if len(portfolio_values) > 0 else 0.0
        beta_adjusted_exposure = current_portfolio_value * beta

        return {
            "history": history,
            "correlation_matrix": corr_matrix,
            "metrics": {
                "volatility": vol,
                "sharpe_ratio": sharpe,
                "beta": beta,
                "portfolio_value": current_portfolio_value,
                "beta_adjusted_exposure": beta_adjusted_exposure
            }
        }
