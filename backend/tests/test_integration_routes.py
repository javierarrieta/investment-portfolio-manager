import pytest
from fastapi.testclient import TestClient
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker
from app.main import app
from app.database import Base, engine, get_db
from app.models import Portfolio, Asset, Transaction, HistoricalPrice
from datetime import datetime, date
from app.schemas import PortfolioCreate, AssetCreate, TransactionCreate

# Use the app's actual engine for integration tests to ensure compatibility
# We'll use a separate database or just ensure we clean up.
# For simplicity in this environment, we'll use the app's engine but wrap it in a transaction or just recreate.
TestingSessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

@pytest.fixture(name="db_session")
def fixture_db_session():
    # Clean up and recreate tables on the app's engine
    Base.metadata.drop_all(bind=engine)
    Base.metadata.create_all(bind=engine)
    session = TestingSessionLocal()
    try:
        yield session
    finally:
        session.close()
        Base.metadata.drop_all(bind=engine)

@pytest.fixture(name="client")
def fixture_client(db_session):
    # We need to override the get_db dependency to use our test session
    def _get_test_db():
        try:
            yield db_session
        finally:
            pass
    
    app.dependency_overrides[get_db] = _get_test_db
    with TestClient(app) as c:
        yield c
    app.dependency_overrides.clear()

# --- Portfolio Tests ---

def test_portfolio_lifecycle(client):
    # Create
    payload = {"name": "Test Portfolio", "description": "Desc", "base_currency": "USD"}
    resp = client.post("/api/portfolios/", json=payload)
    assert resp.status_code == 201
    p_id = resp.json()["id"]

    # List
    resp = client.get("/api/portfolios/")
    assert resp.status_code == 200
    assert len(resp.json()) >= 1

    # Get
    resp = client.get(f"/api/portfolios/{p_id}")
    assert resp.status_code == 200
    assert resp.json()["name"] == "Test Portfolio"

    # Delete
    resp = client.delete(f"/api/portfolios/{p_id}")
    assert resp.status_code == 204

    # Verify deleted
    resp = client.get(f"/api/portfolios/{p_id}")
    assert resp.status_code == 404

# --- Asset & Transaction Tests ---

def test_asset_and_transaction_lifecycle(client, db_session):
    # 1. Setup Portfolio
    p_resp = client.post("/api/portfolios/", json={"name": "Asset Test", "base_currency": "USD"})
    p_id = p_resp.json()["id"]

    # 2. Create Asset
    asset_payload = {"symbol": "AAPL", "name": "Apple", "asset_type": "STOCK", "currency": "USD"}
    a_resp = client.post(f"/api/portfolios/{p_id}/assets/", json=asset_payload)
    assert a_resp.status_code == 201
    a_id = a_resp.json()["id"]

    # 3. Create Transaction
    tx_payload = {"type": "BUY", "quantity": 10.0, "price": 150.0, "fee": 5.0, "date": "2023-01-01T00:00:00"}
    t_resp = client.post(f"/api/portfolios/{p_id}/assets/{a_id}/transactions/", json=tx_payload)
    assert t_resp.status_code == 201
    t_id = t_resp.json()["id"]

    # 4. List Transactions
    t_list_resp = client.get(f"/api/portfolios/{p_id}/transactions/")
    assert t_list_resp.status_code == 200
    assert len(t_list_resp.json()) == 1

    # 5. Delete Transaction
    client.delete(f"/api/transactions/{t_id}")
    t_list_resp = client.get(f"/api/portfolios/{p_id}/transactions/")
    assert len(t_list_resp.json()) == 0

    # 6. Delete Asset
    client.delete(f"/api/assets/{a_id}")
    a_list_resp = client.get(f"/api/portfolios/{p_id}/assets/") # Note: Need to check if this endpoint exists. 
    # Wait, transactions.py doesn't have a list assets endpoint. Let's check.
    # It has: create_asset, delete_asset, create_transaction, list_portfolio_transactions, delete_transaction.
    # Okay, let's just verify asset is gone via transaction check.
    t_resp = client.post(f"/api/portfolios/{p_id}/assets/{a_id}/transactions/", json=tx_payload)
    assert t_resp.status_code == 404

# --- Analytics Tests ---

def test_analytics_endpoints(client, db_session):
    # 1. Setup Portfolio, Asset, Tx, and Price
    p_resp = client.post("/api/portfolios/", json={"name": "Analytics Test", "base_currency": "USD"})
    p_id = p_resp.json()["id"]

    a_resp = client.post(f"/api/portfolios/{p_id}/assets/", json={"symbol": "AAPL", "name": "Apple", "asset_type": "STOCK", "currency": "USD"})
    a_id = a_resp.json()["id"]

    client.post(f"/api/portfolios/{p_id}/assets/{a_id}/transactions/", json={"type": "BUY", "quantity": 10.0, "price": 150.0, "fee": 0.0, "date": "2023-01-01T00:00:00"})

    # Manually add historical price to avoid yfinance in tests
    hp = HistoricalPrice(symbol="AAPL", date=date(2023, 1, 1), close_price=150.0)
    db_session.add(hp)
    db_session.commit()

    # 2. Test Tax Summary
    tax_resp = client.get(f"/api/portfolios/{p_id}/tax-summary")
    assert tax_resp.status_code == 200
    tax_data = tax_resp.json()
    assert "assets" in tax_data
    assert len(tax_data["assets"]) == 1

    # 3. Test Performance
    perf_resp = client.get(f"/api/portfolios/{p_id}/performance")
    assert perf_resp.status_code == 200
    perf_data = perf_resp.json()
    assert "metrics" in perf_data
    assert perf_data["metrics"]["portfolio_value"] > 0

def test_not_found_errors(client):
    assert client.get("/api/portfolios/999").status_code == 404
    assert client.get("/api/portfolios/999/performance").status_code == 404
    assert client.get("/api/portfolios/999/tax-summary").status_code == 404
    assert client.delete("/api/portfolios/999").status_code == 404
    assert client.delete("/api/assets/999").status_code == 404
    assert client.delete("/api/transactions/999").status_code == 404
