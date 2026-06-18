from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from sqlalchemy import text
from .database import engine, Base
from .routes import portfolios, transactions, analytics

# Create the database tables
Base.metadata.create_all(bind=engine)

app = FastAPI(
    title="Investment Portfolio Manager API",
    description="REST API for managing portfolios, assets, FIFO/LIFO tax calculations, and advanced stats.",
    version="1.0.0"
)

@app.on_event("startup")
def run_migrations():
    with engine.connect() as conn:
        # Migrate Portfolios
        try:
            conn.execute(text("ALTER TABLE portfolios ADD COLUMN base_currency VARCHAR DEFAULT 'USD'"))
            conn.commit()
        except Exception: # Column exists
            pass
        # Migrate Assets
        try:
            conn.execute(text("ALTER TABLE assets ADD COLUMN currency VARCHAR DEFAULT 'USD'"))
            conn.commit()
        except Exception: # Column exists
            pass

# Enable CORS for frontend integration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # For local development; restrict in production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include API routers
app.include_router(portfolios.router, prefix="/api")
app.include_router(transactions.router, prefix="/api")
app.include_router(analytics.router, prefix="/api")

@app.get("/")
def read_root():
    return {
        "message": "Welcome to the Investment Portfolio Manager API",
        "docs": "/docs",
        "openapi": "/openapi.json"
    }
