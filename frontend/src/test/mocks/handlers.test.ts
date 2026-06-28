import { describe, expect, it } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from './server'
import type { Portfolio, Asset } from '../../types'

describe('MSW Test Setup', () => {
  it('should intercept GET /api/portfolios', async () => {
    const res = await fetch('/api/portfolios')
    const data = await res.json() as Portfolio[]

    expect(res.status).toBe(200)
    expect(data).toHaveLength(2)
    expect(data[0].name).toBe('Test Portfolio')
  })

  it('should intercept GET /api/portfolios/:id', async () => {
    const res = await fetch('/api/portfolios/42')
    const data = await res.json() as Portfolio

    expect(res.status).toBe(200)
    expect(data.id).toBe(42)
    expect(data.name).toBe('Portfolio 42')
  })

  it('should intercept POST /api/portfolios', async () => {
    server.resetHandlers()
    server.use(
      http.post('/api/portfolios', async ({ request }) => {
        const body = (await request.json()) as { name?: string; description?: string; currency?: string }
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

    try {
      const res = await fetch('/api/portfolios', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: 'New Portfolio', currency: 'EUR' })
      })
      const data = await res.json() as Portfolio

      expect(res.status).toBe(201)
      expect(data.id).toBe(99)
      expect(data.name).toBe('New Portfolio')
    } finally {
      server.resetHandlers()
    }
  })

  it('should intercept GET /api/assets/:id', async () => {
    const res = await fetch('/api/assets/10')
    const data = await res.json() as Asset

    expect(res.status).toBe(200)
    expect(data.symbol).toBe('AAPL')
  })
})
