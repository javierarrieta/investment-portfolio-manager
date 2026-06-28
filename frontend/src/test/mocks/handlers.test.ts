import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { http, HttpResponse } from 'msw'
import { setupServer } from 'msw/node'
import type { Portfolio, Asset } from '../../types'

const server = setupServer()

describe('MSW Test Setup', () => {
  beforeEach(() => {
    server.listen({ onUnhandledRequest: 'bypass' })
  })

  afterEach(() => {
    server.resetHandlers()
    server.close()
  })

  it('should intercept GET /api/portfolios', async () => {
    const mockPortfolios: Portfolio[] = [
      {
        id: 1,
        name: 'Test Portfolio',
        description: 'A test portfolio',
        currency: 'USD',
        assets: []
      }
    ]

    server.use(
      http.get('/api/portfolios', () => {
        return HttpResponse.json(mockPortfolios)
      })
    )

    const res = await fetch('/api/portfolios')
    const data = await res.json()
    
    expect(res.status).toBe(200)
    expect(data).toHaveLength(1)
    expect(data[0].name).toBe('Test Portfolio')
  })

  it('should intercept GET /api/portfolios/:id', async () => {
    const mockPortfolio: Portfolio = {
      id: 42,
      name: 'Portfolio 42',
      description: 'Test',
      currency: 'USD',
      assets: []
    }

    server.use(
      http.get('/api/portfolios/:id', ({ params }) => {
        const id = Number(params.id)
        return HttpResponse.json({ id, name: `Portfolio ${id}`, description: 'Test', currency: 'USD', assets: [] })
      })
    )

    const res = await fetch('/api/portfolios/42')
    const data = await res.json()
    
    expect(res.status).toBe(200)
    expect(data.id).toBe(42)
  })

  it('should intercept POST /api/portfolios', async () => {
    server.use(
      http.post('/api/portfolios', async ({ request }) => {
        const body = await request.json() as Partial<Portfolio>
        const result: Portfolio = {
          id: 99,
          name: body.name || '',
          description: body.description,
          currency: body.currency || 'USD',
          assets: []
        }
        return HttpResponse.json(result, { status: 201 })
      })
    )

    const res = await fetch('/api/portfolios', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: 'New Portfolio', currency: 'EUR' })
    })
    const data = await res.json()
    
    expect(res.status).toBe(201)
    expect(data.id).toBe(99)
    expect(data.name).toBe('New Portfolio')
  })

  it('should intercept GET /api/assets/:id', async () => {
    const mockAsset: Asset = {
      id: 10,
      portfolio_id: 1,
      symbol: 'MSFT',
      name: 'Microsoft',
      asset_type: 'STOCK',
      currency: 'USD',
      transactions: []
    }

    server.use(
      http.get('/api/assets/:id', () => {
        return HttpResponse.json(mockAsset)
      })
    )

    const res = await fetch('/api/assets/10')
    const data = await res.json()
    
    expect(res.status).toBe(200)
    expect(data.symbol).toBe('MSFT')
  })
})
