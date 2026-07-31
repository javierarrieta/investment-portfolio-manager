import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import Dashboard from '../../components/Dashboard'
import type { PortfolioPerformance, TaxSummary } from '../../types'

const makePerformance = (portfolioValue: number, unrealizedPnl: number, realizedPnl: number = 0): PortfolioPerformance => ({
  history: [
    { date: '2024-01-01', value: portfolioValue, daily_return: 0, twr: 0, cash_flow: 0 },
  ],
  correlation_matrix: {},
  metrics: {
    volatility: 0,
    sharpe_ratio: 0,
    beta: 1,
    portfolio_value: portfolioValue,
    beta_adjusted_exposure: portfolioValue,
    unrealized_pnl: unrealizedPnl,
    realized_pnl: realizedPnl,
  },
})

const makeTaxSummary = (marketValue: number, currency: string = 'USD'): TaxSummary => ({
  assets: [
    {
      symbol: 'AAPL',
      asset_type: 'STOCK',
      current_shares: 10,
      average_cost: 150,
      current_price: 175,
      total_cost: 1500,
      market_value: marketValue,
      unrealized_pnl: marketValue - 1500,
      unrealized_roi: 0.1667,
      realized_pnl: 0,
      tax_lots: [],
    },
  ],
  currency,
  total_portfolio_value: marketValue,
  total_realized_pnl: 0,
  total_unrealized_pnl: marketValue - 1500,
})

describe('Dashboard', () => {
  it('shows 0% allocation instead of Infinity when totalValue is zero', () => {
    const performance = makePerformance(0, 0)
    const taxSummary = makeTaxSummary(1750)
    taxSummary.total_portfolio_value = 0

    render(<Dashboard performance={performance} taxSummary={taxSummary} />)

    expect(screen.queryByText('Infinity')).not.toBeInTheDocument()
    const legendItems = screen.getAllByText(/%$/)
    expect(legendItems.length).toBeGreaterThan(0)
    expect(legendItems[0].textContent).toBe('0%')
  })

  it('shows correct allocation percentage when totalValue is non-zero', () => {
    const performance = makePerformance(10000, 1000)
    const taxSummary = makeTaxSummary(5000, 'USD')
    taxSummary.total_portfolio_value = 10000

    render(<Dashboard performance={performance} taxSummary={taxSummary} />)

    const legendItems = screen.getAllByText(/%$/)
    expect(legendItems.length).toBeGreaterThan(0)
    expect(legendItems[0].textContent).toBe('50%')
  })

  it('shows zero values for all metrics when portfolio_value is zero', () => {
    const performance = makePerformance(0, 0)
    const taxSummary = makeTaxSummary(0)

    render(<Dashboard performance={performance} taxSummary={taxSummary} />)

    const zeroValues = screen.getAllByText('$0.00')
    expect(zeroValues.length).toBeGreaterThanOrEqual(1)
  })
})