/**
 * @fileoverview TEST FIXTURE - PortfolioForm component used only in tests.
 * This is a minimal implementation for demonstrating test patterns.
 * Do not use in production.
 */
import React, { useState, useEffect } from 'react'
import { Portfolio } from '../../types'

const API_BASE = '/api'

interface PortfolioFormProps {
  onSuccess?: () => void
}

export default function PortfolioForm({ onSuccess }: PortfolioFormProps) {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [submitted, setSubmitted] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = async () => {
    if (submitting) return
    setSubmitting(true)
    setError(null)
    try {
      const res = await fetch(`${API_BASE}/portfolios`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, description, currency: 'USD' })
      })
      if (!res.ok) throw new Error('Failed to create portfolio')
      await res.json()
      setSubmitted(true)
      if (onSuccess) onSuccess()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSubmitting(false)
    }
  }

  if (submitted) {
    return <div>Portfolio created!</div>
  }

  return (
    <form onSubmit={(e) => { e.preventDefault(); handleSubmit() }} className="portfolio-form">
      <h2>Create Portfolio</h2>
      <div className="form-group">
        <label htmlFor="portfolio-name">Name</label>
        <input
          id="portfolio-name"
          type="text"
          value={name}
          onChange={e => setName(e.target.value)}
          required
        />
      </div>
      <div className="form-group">
        <label htmlFor="portfolio-description">Description</label>
        <textarea
          id="portfolio-description"
          value={description}
          onChange={e => setDescription(e.target.value)}
          rows={3}
        />
      </div>
      {error && <p style={{ color: 'var(--color-danger)' }}>{error}</p>}
      <button type="submit" className="btn btn-primary">Create</button>
    </form>
  )
}
