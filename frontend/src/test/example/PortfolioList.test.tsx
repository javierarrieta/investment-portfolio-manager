import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import PortfolioList from '../../components/PortfolioList'

describe('PortfolioList', () => {
  it('renders a list of portfolios', async () => {
    render(<PortfolioList />)

    await waitFor(() => {
      expect(screen.getByText('Test Portfolio')).toBeInTheDocument()
      expect(screen.getByText('Retirement Fund')).toBeInTheDocument()
    })
  })

  it('shows loading state initially', async () => {
    render(<PortfolioList />)
    await waitFor(() => {
      expect(screen.getByText('Loading...')).toBeInTheDocument()
    })
  })
})
