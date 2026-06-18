from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List
from ..database import get_db
from .. import models, schemas

router = APIRouter(
    prefix="/portfolios",
    tags=["portfolios"]
)

@router.post("/", response_model=schemas.PortfolioOut, status_code=status.HTTP_201_CREATED)
def create_portfolio(portfolio: schemas.PortfolioCreate, db: Session = Depends(get_db)):
    db_portfolio = models.Portfolio(**portfolio.dict())
    db.add(db_portfolio)
    db.commit()
    db.refresh(db_portfolio)
    return db_portfolio

@router.get("/", response_model=List[schemas.PortfolioOut])
def list_portfolios(db: Session = Depends(get_db)):
    return db.query(models.Portfolio).all()

@router.get("/{portfolio_id}", response_model=schemas.PortfolioOut)
def get_portfolio(portfolio_id: int, db: Session = Depends(get_db)):
    portfolio = db.query(models.Portfolio).filter(models.Portfolio.id == portfolio_id).first()
    if not portfolio:
        raise HTTPException(status_code=404, detail="Portfolio not found")
    return portfolio

@router.delete("/{portfolio_id}", status_code=status.HTTP_204_NO_CONTENT)
def delete_portfolio(portfolio_id: int, db: Session = Depends(get_db)):
    portfolio = db.query(models.Portfolio).filter(models.Portfolio.id == portfolio_id).first()
    if not portfolio:
        raise HTTPException(status_code=404, detail="Portfolio not found")
    db.delete(portfolio)
    db.commit()
    return {"detail": "Portfolio deleted"}
