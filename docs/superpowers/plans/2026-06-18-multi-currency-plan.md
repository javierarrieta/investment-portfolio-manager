### Task 4: Stats Engine Integration

**Files:**
- Modify: `backend/app/stats_engine.py`
- Test: `backend/tests/test_stats_engine.py`

- [ ] **Step 1: Update `StatsEngine.calculate_portfolio_performance` to be async and accept `base_currency` and `currency_service`**

```python
# backend/app/stats_engine.py

class StatsEngine:
    @staticmethod
    async def calculate_portfolio_performance(
        db: Session,
        assets: List[Asset],
        transactions: List[Transaction],
        base_currency: str,
        currency_service: CurrencyService
    ) -> Dict[str, Any]:
        # ... implementation ...
```

- [ ] **Step 2: Implement currency conversion in `calculate_portfolio_performance`**
    - For daily portfolio value:
      ```python
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
      ```
    - For daily cash flows (transactions):
      ```python
      for tx in sorted_txs:
          tx_date = tx.date.date()
          if tx_date in dates:
              idx = dates.index(tx_date)
              # Get asset currency
              asset = db.query(Asset).get(tx.asset_id)
              cost = tx.quantity * tx.price
              
              if asset.currency != base_currency:
                  rate = await currency_service.get_rate(asset.currency, base_currency, tx.date)
                  cost *= rate
                  # fee is usually in asset currency too? Let's assume so.
                  # tx.fee is also converted.
                  # ...
              
              if tx.type.upper() == "BUY":
                  daily_cash_flow[idx] += (cost + tx.fee) # need to convert fee too
              elif tx.type.upper() == "SELL":
                  daily_cash_flow[idx] -= (cost - tx.fee)
      ```

- [ ] **Step 3: Write failing test in `backend/tests/test_stats_engine.py`**

```python
@pytest.mark.asyncio
async def test_calculate_portfolio_performance_multi_currency():
    # 1. Setup in-memory DB with Portfolio (USD), Asset1 (USD), Asset2 (EUR)
    # 2. Add Transactions
    # 3. Mock CurrencyService.get_rate to return 1.1 for EUR->USD
    # 4. Run calculate_portfolio_performance
    # 5. Assert portfolio_value and TWR match expected base-currency values
    pass
```

- [ ] **Step 4: Run test to verify it fails**

Run: `PYTHONPATH=backend pytest backend/tests/test_stats_engine.py -v`
Expected: FAIL (due to missing arguments or unhandled currency)

- [ ] **Step 5: Implement minimal code to make the test pass**

- [ ] **Step 6: Run test to verify it passes**

Run: `PYTHONPATH=backend pytest backend/tests/test_stats_engine.py -v`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add backend/app/stats_engine.py backend/tests/test_stats_engine.py
git commit -m "feat: integrate CurrencyService into StatsEngine"
```
