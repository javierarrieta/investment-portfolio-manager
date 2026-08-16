import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import App from '../../App'

const openLedger = async () => {
  await waitFor(() => screen.getAllByText('Test Portfolio'))
  fireEvent.click(screen.getByText('Transaction Ledger'))
  await waitFor(() => screen.getAllByText('SPLIT'))
}

describe('App ledger', () => {
  it('renders SPLIT transactions in the transaction ledger', async () => {
    render(<App />)

    await openLedger()

    // SPLIT quantity renders as "numerator : denominator", price/fee/total as "—"
    expect(screen.getByText('2 : 1')).toBeInTheDocument()
    expect(screen.getAllByText('—').length).toBeGreaterThan(0)
  })

  it('renders SPLIT rows with muted styling (not sell styling)', async () => {
    render(<App />)

    await openLedger()

    const splitCell = screen.getAllByText('SPLIT')[0]
    const style = splitCell.getAttribute('style') || ''
    // SPLIT uses the muted secondary color, not the danger color used for sells
    expect(style).toContain('--text-secondary')
  })
})
