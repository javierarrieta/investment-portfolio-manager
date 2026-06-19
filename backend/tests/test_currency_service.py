import pytest
from datetime import datetime
from app.services.currency_service import CurrencyService

@pytest.mark.asyncio
async def test_get_rate_success():
    service = CurrencyService()
    # Mocking the Yahoo Finance fetch in the implementation
    rate = await service.get_rate("EUR", "USD", datetime(2024, 1, 1))
    assert rate > 0
