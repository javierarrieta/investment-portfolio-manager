import { http, HttpResponse } from 'msw'
import type { Portfolio, Asset } from '../../types'

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
  })
]
