import { describe, it, expect } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import PortfolioForm from '../../components/PortfolioForm'

describe('PortfolioForm', () => {
  it('submits a new portfolio', async () => {
    const user = userEvent.setup()
    render(<PortfolioForm />)

    await user.type(screen.getByLabelText(/name/i), 'New Portfolio')
    await user.type(screen.getByLabelText(/description/i), 'A new portfolio')

    await user.click(screen.getByRole('button', { name: 'Create' }))

    await waitFor(() => {
      expect(screen.getByText('Portfolio created!')).toBeInTheDocument()
    })
  })
})
