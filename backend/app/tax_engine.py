from datetime import datetime, timedelta
from typing import List, Dict, Any, Tuple
from .schemas import TaxLot, AssetTaxSummary

class TaxLotEngine:
    @staticmethod
    async def calculate_lots(
        symbol: str,
        asset_type: str,
        transactions: List[Any],  # List of Transaction database models
        current_price: float,
        asset_currency: str,      # New
        base_currency: str,       # New
        currency_service: Any,    # New
        strategy: str = "FIFO",  # FIFO, LIFO, HYBRID
        hybrid_threshold_days: int = 30
    ) -> Dict[str, Any]:
        """
        Processes buy and sell transactions chronologically to calculate:
        - Realized P&L
        - Remaining open tax lots (for unrealized/latent P&L calculations)
        - Cost basis metrics
        """
        # Sort transactions chronologically
        sorted_txs = sorted(transactions, key=lambda x: x.date)

        # Convert current_price to base_currency if necessary
        if asset_currency != base_currency:
            # Use the latest transaction date or now for current price conversion
            latest_date = sorted_txs[-1].date if sorted_txs else datetime.now()
            rate = await currency_service.get_rate(asset_currency, base_currency, latest_date)
            current_price_base = current_price * rate
        else:
            current_price_base = current_price

        # Active buy lots list. Each lot is represented as a dict:
        # {
        #   "date": datetime,
        #   "price": float, (raw unit price in base currency)
        #   "qty": float, (remaining qty)
        #   "fee_per_unit": float, (in base currency)
        #   "unit_cost": float  # (qty * price + fee) / qty (in base currency)
        # }
        buy_lots = []
        realized_pnl = 0.0

        for tx in sorted_txs:
            # Get conversion rate for this transaction
            if asset_currency != base_currency:
                tx_rate = await currency_service.get_rate(asset_currency, base_currency, tx.date)
            else:
                tx_rate = 1.0

            if tx.type.upper() == "BUY":
                unit_cost_asset = (tx.quantity * tx.price + tx.fee) / tx.quantity if tx.quantity > 0 else tx.price
                unit_cost_base = unit_cost_asset * tx_rate
                buy_lots.append({
                    "date": tx.date,
                    "price": tx.price * tx_rate,
                    "qty": tx.quantity,
                    "fee_per_unit": (tx.fee / tx.quantity if tx.quantity > 0 else 0.0) * tx_rate,
                    "unit_cost": unit_cost_base
                })
            elif tx.type.upper() == "SELL":
                qty_to_sell = tx.quantity
                sell_unit_proceeds_asset = (tx.quantity * tx.price - tx.fee) / tx.quantity if tx.quantity > 0 else tx.price
                sell_unit_proceeds_base = sell_unit_proceeds_asset * tx_rate

                # Filter lots that were bought BEFORE or AT the sell date and have remaining quantity
                eligible_lots = [lot for lot in buy_lots if lot["date"] <= tx.date and lot["qty"] > 0]

                if not eligible_lots:
                    # In case of short sales or data issues, we skip matching or handle gracefully
                    continue

                # Sort eligible lots based on strategy
                if strategy.upper() == "FIFO":
                    # Oldest first
                    eligible_lots.sort(key=lambda x: x["date"])
                elif strategy.upper() == "LIFO":
                    # Newest first
                    eligible_lots.sort(key=lambda x: x["date"], reverse=True)
                elif strategy.upper() == "HYBRID":
                    # LIFO for short-term acquisitions (<= threshold days), FIFO for long-term
                    short_term = []
                    long_term = []
                    for lot in eligible_lots:
                        age = tx.date - lot["date"]
                        if 0 <= age.days <= hybrid_threshold_days:
                            short_term.append(lot)
                        else:
                            long_term.append(lot)

                    # Newest short term first (LIFO)
                    short_term.sort(key=lambda x: x["date"], reverse=True)
                    # Oldest long term first (FIFO)
                    long_term.sort(key=lambda x: x["date"])

                    # Candidates list
                    eligible_lots = short_term + long_term
                else:
                    # Default to FIFO
                    eligible_lots.sort(key=lambda x: x["date"])

                # Match sell against candidates
                for lot in eligible_lots:
                    if qty_to_sell <= 0:
                        break

                    matched_qty = min(qty_to_sell, lot["qty"])
                    cost_basis = matched_qty * lot["unit_cost"]
                    proceeds = matched_qty * sell_unit_proceeds_base
                    
                    realized_pnl += (proceeds - cost_basis)

                    lot["qty"] -= matched_qty
                    qty_to_sell -= matched_qty

        # Calculate latent profit/loss on remaining open buy lots
        open_lots = []
        total_remaining_qty = 0.0
        total_cost_basis = 0.0

        for lot in buy_lots:
            if lot["qty"] > 0:
                total_remaining_qty += lot["qty"]
                lot_cost = lot["qty"] * lot["unit_cost"]
                total_cost_basis += lot_cost
                
                # Latent metrics for this lot
                lot_market_value = lot["qty"] * current_price_base
                latent_gain_loss = lot_market_value - lot_cost
                latent_roi = (latent_gain_loss / lot_cost) if lot_cost > 0 else 0.0

                open_lots.append(TaxLot(
                    buy_date=lot["date"],
                    buy_price=lot["price"],
                    original_qty=lot["qty"],
                    remaining_qty=lot["qty"],
                    latent_gain_loss=latent_gain_loss,
                    latent_roi=latent_roi
                ))

        market_value = total_remaining_qty * current_price_base
        unrealized_pnl = market_value - total_cost_basis
        unrealized_roi = (unrealized_pnl / total_cost_basis) if total_cost_basis > 0 else 0.0
        average_cost = (total_cost_basis / total_remaining_qty) if total_remaining_qty > 0 else 0.0

        return {
            "symbol": symbol,
            "asset_type": asset_type,
            "current_shares": total_remaining_qty,
            "average_cost": average_cost,
            "current_price": current_price_base,
            "total_cost": total_cost_basis,
            "market_value": market_value,
            "unrealized_pnl": unrealized_pnl,
            "unrealized_roi": unrealized_roi,
            "realized_pnl": realized_pnl,
            "tax_lots": open_lots
        }
