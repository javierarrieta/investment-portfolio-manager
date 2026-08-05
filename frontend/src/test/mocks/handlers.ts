import { http, HttpResponse } from 'msw'
import type { Portfolio, Asset } from '../../types'
import type { AssetType } from '../../types'

export const handlers = [
  http.get('/api/portfolios', () => {
    const mockPortfolios: Portfolio[] = [
      {
        id: 1,
        name: 'Test Portfolio',
        description: 'A test portfolio',
        currency: 'USD',
        assets: [
          {
            id: 1,
            portfolio_id: 1,
            symbol: 'AAPL',
            name: 'Apple Inc.',
            asset_type: 'STOCK',
            sector: 'Technology',
            currency: 'USD',
            transactions: []
          }
        ]
      },
      {
        id: 2,
        name: 'Retirement Fund',
        description: 'Retirement savings',
        currency: 'USD',
        assets: []
      }
    ]
    return HttpResponse.json(mockPortfolios)
  }),
  http.get('/api/portfolios/:id', ({ params }) => {
    const id = Number(params.id)
    const mockPortfolio: Portfolio = {
      id,
      name: `Portfolio ${id}`,
      description: `Description for portfolio ${id}`,
      currency: 'USD',
      assets: [
        {
          id: 1,
          portfolio_id: id,
          symbol: 'AAPL',
          name: 'Apple Inc.',
          asset_type: 'STOCK',
          sector: 'Technology',
          currency: 'USD',
          transactions: []
        }
      ]
    }
    return HttpResponse.json(mockPortfolio)
  }),
  http.post('/api/portfolios', async ({ request }) => {
    const body = (await request.json()) as Partial<Portfolio>
    const newPortfolio: Portfolio = {
      id: 3,
      name: body.name || 'New Portfolio',
      description: body.description,
      currency: body.currency || 'USD',
      assets: []
    }
    return HttpResponse.json(newPortfolio, { status: 201 })
  }),
  http.get('/api/assets/:id', ({ params }) => {
    const id = Number(params.id)
    const mockAsset: Asset = {
      id,
      portfolio_id: 1,
      symbol: 'AAPL',
      name: 'Apple Inc.',
      asset_type: 'STOCK',
      sector: 'Technology',
      currency: 'USD',
      transactions: []
    }
    return HttpResponse.json(mockAsset)
  }),
  http.post('/api/portfolios/:portfolioId/assets', async ({ params, request }) => {
    const { portfolioId } = params
    const body = (await request.json()) as { symbol: string; name: string; asset_type: string; currency: string }
    const newAsset: Asset = {
      id: 999,
      portfolio_id: Number(portfolioId),
      symbol: body.symbol.toUpperCase(),
      name: body.name,
      asset_type: body.asset_type.toUpperCase() as AssetType,
      sector: undefined,
      currency: body.currency || 'USD',
      transactions: []
    }
    return HttpResponse.json(newAsset, { status: 201 })
  }),
  http.post('/api/portfolios/:portfolioId/assets/:assetId/transactions', async ({ params, request }) => {
    const { assetId } = params
    const body = (await request.json()) as { type: string; quantity: number; price: number; fee: number; date: string }
    return HttpResponse.json({
      id: 1000,
      asset_id: Number(assetId),
      type: body.type.toUpperCase(),
      quantity: String(body.quantity),
      price: String(body.price),
      fee: String(body.fee),
      date: body.date
    }, { status: 201 })
  }),
  http.delete('/api/portfolios/:id', () => {
    return HttpResponse.json(null, { status: 204 })
  }),
  http.all('*', () => {
    return HttpResponse.json({ error: 'not implemented' }, { status: 501 })
  })
]
