import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import PortfolioDetail from '../../components/PortfolioDetail'
import type { Portfolio, TaxSummary } from '../../types'

const mockPortfolio: Portfolio = {
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
      transactions: [],
    },
  ],
}

const mockTaxSummary: TaxSummary = {
  assets: [
    {
      symbol: 'AAPL',
      asset_type: 'STOCK',
      current_shares: 100,
      average_cost: 150,
      current_price: 175,
      total_cost: 15000,
      market_value: 17500,
      unrealized_pnl: 2500,
      unrealized_roi: 0.1667,
      realized_pnl: 0,
      tax_lots: [],
    },
  ],
  currency: 'USD',
}

const emptyTaxSummary: TaxSummary = {
  assets: [],
  currency: 'USD',
}

const noop = async () => {}

describe('PortfolioDetail', () => {
  it('shows "Log Transaction" button when assets exist', () => {
    render(
      <PortfolioDetail
        portfolio={mockPortfolio}
        taxSummary={mockTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    expect(screen.getByText('Log Transaction')).toBeVisible()
  })

  it('hides "Log Transaction" button when no assets exist', () => {
    render(
      <PortfolioDetail
        portfolio={{ ...mockPortfolio, assets: [] }}
        taxSummary={emptyTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    expect(screen.queryByText('Log Transaction')).not.toBeInTheDocument()
  })

  it('shows "Add Asset Symbol" button always', () => {
    render(
      <PortfolioDetail
        portfolio={{ ...mockPortfolio, assets: [] }}
        taxSummary={emptyTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    expect(screen.getByText('Add Asset Symbol')).toBeVisible()
  })

  it('opens transaction modal when "Log Transaction" is clicked', async () => {
    const user = userEvent.setup()
    render(
      <PortfolioDetail
        portfolio={mockPortfolio}
        taxSummary={mockTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    await user.click(screen.getByText('Log Transaction'))

    expect(screen.getByText('Log Buy/Sell Transaction')).toBeVisible()
  })

  it('opens asset modal when "Add Asset Symbol" is clicked', async () => {
    const user = userEvent.setup()
    render(
      <PortfolioDetail
        portfolio={{ ...mockPortfolio, assets: [] }}
        taxSummary={emptyTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    await user.click(screen.getByText('Add Asset Symbol'))

    expect(screen.getByText('Register New Asset Symbol')).toBeVisible()
  })

  it('renders tax lot details when asset row is expanded', () => {
    const taxSummaryWithLots: TaxSummary = {
      assets: [
        {
          symbol: 'AAPL',
          asset_type: 'STOCK',
          current_shares: 100,
          average_cost: 150,
          current_price: 175,
          total_cost: 15000,
          market_value: 17500,
          unrealized_pnl: 2500,
          unrealized_roi: 0.1667,
          realized_pnl: 0,
          tax_lots: [
            {
              buy_date: '2024-01-15T00:00:00Z',
              buy_price: 150,
              original_qty: 50,
              remaining_qty: 50,
              latent_gain_loss: 1250,
              latent_roi: 0.1667,
            },
          ],
        },
      ],
      currency: 'USD',
    }

    render(
      <PortfolioDetail
        portfolio={mockPortfolio}
        taxSummary={taxSummaryWithLots}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    const aaplRow = screen.getByText('AAPL').closest('tr')
    expect(aaplRow).toBeDefined()
    const expandButton = aaplRow!.querySelector('button')
    expect(expandButton).toBeDefined()
    fireEvent.click(expandButton!)
    expect(screen.getByText('Tax Lots (Unrealized Profit Details)')).toBeVisible()
  })

  it('shows empty state message when no assets are registered', () => {
    render(
      <PortfolioDetail
        portfolio={{ ...mockPortfolio, assets: [] }}
        taxSummary={emptyTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    expect(
      screen.getByText('No assets registered yet. Click "Add Asset Symbol" to begin.')
    ).toBeVisible()
  })

  it('allows selecting BUY or SELL in transaction modal', async () => {
    const user = userEvent.setup()
    render(
      <PortfolioDetail
        portfolio={mockPortfolio}
        taxSummary={mockTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={noop}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    await user.click(screen.getByText('Log Transaction'))
    expect(screen.getByText('Log Buy/Sell Transaction')).toBeVisible()

    const buyOption = screen.getByText('BUY')
    const typeSelect = buyOption.closest('select') as HTMLSelectElement
    expect(typeSelect).toBeDefined()
    expect(typeSelect.value).toBe('BUY')

    await user.selectOptions(typeSelect, 'SELL')
    expect(typeSelect.value).toBe('SELL')
  })

  it('calls onAddTransaction when transaction form is submitted', async () => {
    const user = userEvent.setup()
    const onAddTransaction = vi.fn().mockResolvedValue(undefined)

    render(
      <PortfolioDetail
        portfolio={mockPortfolio}
        taxSummary={mockTaxSummary}
        onAddAsset={noop}
        onDeleteAsset={noop}
        onAddTransaction={onAddTransaction}
        strategy="FIFO"
        setStrategy={() => {}}
        thresholdDays={30}
        setThresholdDays={() => {}}
      />
    )

    // Open the transaction modal
    await user.click(screen.getAllByText('Log Transaction')[0])
    expect(screen.getByText('Log Buy/Sell Transaction')).toBeVisible()

    // Select asset
    const aaplOption = screen.getByText('AAPL (Apple Inc.)')
    const assetSelect = aaplOption.closest('select') as HTMLSelectElement
    await user.selectOptions(assetSelect, '1')

    // Find the form and submit button within it
    const form = document.querySelector('form')
    expect(form).toBeDefined()
    const submitBtn = form!.querySelector('button[type="submit"]')
    expect(submitBtn).toBeDefined()

    // Set values on controlled inputs and dispatch React-compatible events
    const spinbuttons = screen.getAllByRole('spinbutton') as HTMLInputElement[]
    const quantityInput = spinbuttons[0]
    const priceInput = spinbuttons[1]

    // Simulate React's onChange by setting value and dispatching input event
    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype, 'value'
    )?.set
    nativeInputValueSetter?.call(quantityInput, '10')
    quantityInput.dispatchEvent(new Event('input', { bubbles: true }))
    quantityInput.dispatchEvent(new Event('change', { bubbles: true }))

    nativeInputValueSetter?.call(priceInput, '180')
    priceInput.dispatchEvent(new Event('input', { bubbles: true }))
    priceInput.dispatchEvent(new Event('change', { bubbles: true }))

    await user.click(submitBtn!)

    expect(onAddTransaction).toHaveBeenCalledWith(1, expect.objectContaining({
      type: 'BUY',
      quantity: 10,
      price: 180,
    }))
  })
})
