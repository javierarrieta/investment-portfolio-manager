import asyncio
import yfinance as yf
from datetime import datetime, timedelta
from typing import Dict, Tuple

class CurrencyService:
    def __init__(self):
        # Cache key is (from_currency, to_currency, date)
        self.cache: Dict[Tuple[str, str, datetime.date], float] = {}

    async def get_rate(self, from_curr: str, to_curr: str, date: datetime) -> float:
        cache_key = (from_curr, to_curr, date.date())
        if cache_key in self.cache:
            return self.cache[cache_key]

        if from_curr == to_curr:
            return 1.0

        symbol = f"{from_curr}{to_curr}=X"

        def fetch_rate():
            ticker = yf.Ticker(symbol)
            start_date = date.strftime('%Y-%m-%d')
            end_date = (date + timedelta(days=1)).strftime('%Y-%m-%d')
            hist = ticker.history(start=start_date, end=end_date)
            if not hist.empty:
                return float(hist.iloc[0]['Close'])
            return None

        loop = asyncio.get_running_loop()
        rate = await loop.run_in_executor(None, fetch_rate)

        if rate is not None:
            self.cache[cache_key] = rate
            return rate
        
        raise Exception(f"Failed to fetch exchange rate for {symbol} on {date.date()}")
