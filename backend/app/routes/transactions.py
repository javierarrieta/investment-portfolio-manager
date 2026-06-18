from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List
from ..database import get_db
from .. import models, schemas

router = APIRouter(
    tags=["transactions"]
)

# --- Asset CRUD ---
@router.post("/portfolios/{portfolio_id}/assets/", response_model=schemas.AssetOut, status_code=status.HTTP_201_CREATED)
def create_asset(portfolio_id: int, asset: schemas.AssetCreate, db: Session = Depends(get_db)):
    portfolio = db.query(models.Portfolio).filter(models.Portfolio.id == portfolio_id).first()
    if not portfolio:
        raise HTTPException(status_code=404, detail="Portfolio not found")
    
    # Check if asset with symbol already exists in portfolio
    existing = db.query(models.Asset).filter_by(portfolio_id=portfolio_id, symbol=asset.symbol.upper()).first()
    if existing:
        raise HTTPException(status_code=400, detail=f"Asset {asset.symbol} already exists in this portfolio")

    db_asset = models.Asset(
        portfolio_id=portfolio_id,
        symbol=asset.symbol.upper(),
        name=asset.name,
        asset_type=asset.asset_type.upper(),
        sector=asset.sector
    )
    db.add(db_asset)
    db.commit()
    db.refresh(db_asset)
    return db_asset

@router.delete("/assets/{asset_id}", status_code=status.HTTP_204_NO_CONTENT)
def delete_asset(asset_id: int, db: Session = Depends(get_db)):
    asset = db.query(models.Asset).filter(models.Asset.id == asset_id).first()
    if not asset:
        raise HTTPException(status_code=404, detail="Asset not found")
    db.delete(asset)
    db.commit()
    return {"detail": "Asset deleted"}


# --- Transaction CRUD ---
@router.post("/portfolios/{portfolio_id}/assets/{asset_id}/transactions/", response_model=schemas.TransactionOut, status_code=status.HTTP_201_CREATED)
def create_transaction(portfolio_id: int, asset_id: int, transaction: schemas.TransactionCreate, db: Session = Depends(get_db)):
    asset = db.query(models.Asset).filter(models.Asset.id == asset_id, models.Asset.portfolio_id == portfolio_id).first()
    if not asset:
        raise HTTPException(status_code=404, detail="Asset not found in portfolio")

    db_transaction = models.Transaction(
        asset_id=asset_id,
        type=transaction.type.upper(),
        quantity=transaction.quantity,
        price=transaction.price,
        fee=transaction.fee,
        date=transaction.date
    )
    db.add(db_transaction)
    db.commit()
    db.refresh(db_transaction)

    # Trigger async fetch of historical prices to make sure our cache has the new data
    try:
        # We can run sync in-process since it is fast for a single symbol
        from ..stats_engine import StatsEngine
        StatsEngine.sync_historical_prices(db, [asset.symbol], transaction.date.date())
    except Exception as e:
        print(f"Non-blocking error syncing price: {e}")

    return db_transaction

@router.get("/portfolios/{portfolio_id}/transactions/", response_model=List[schemas.TransactionOut])
def list_portfolio_transactions(portfolio_id: int, db: Session = Depends(get_db)):
    portfolio = db.query(models.Portfolio).filter(models.Portfolio.id == portfolio_id).first()
    if not portfolio:
        raise HTTPException(status_code=404, detail="Portfolio not found")
    
    # Get all asset IDs for this portfolio
    asset_ids = [a.id for a in portfolio.assets]
    return db.query(models.Transaction).filter(models.Transaction.asset_id.in_(asset_ids)).order_by(models.Transaction.date.desc()).all()

@router.delete("/transactions/{transaction_id}", status_code=status.HTTP_204_NO_CONTENT)
def delete_transaction(transaction_id: int, db: Session = Depends(get_db)):
    transaction = db.query(models.Transaction).filter(models.Transaction.id == transaction_id).first()
    if not transaction:
        raise HTTPException(status_code=404, detail="Transaction not found")
    db.delete(transaction)
    db.commit()
    return {"detail": "Transaction deleted"}
