import React, { useState, useEffect } from 'react'
import type { Portfolio } from '../../types'

export default function PortfolioList() {
  const [portfolios, setPortfolios] = useState<Portfolio[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let cancelled = false
    fetch('/api/portfolios')
      .then(res => {
        if (!res.ok) throw new Error('Failed to fetch portfolios')
        return res.json()
      })
      .then(data => {
        if (!cancelled) {
          setPortfolios(data)
          setLoading(false)
        }
      })
      .catch(() => {
        if (!cancelled) setLoading(false)
      })
    return () => { cancelled = true }
  }, [])

  if (loading) {
    return (
      <div data-testid="loading">
        <h2>Loading...</h2>
      </div>
    )
  }

  return (
    <div className="portfolio-list">
      <h2>My Portfolios</h2>
      {portfolios.length === 0 ? (
        <p>No portfolios found</p>
      ) : (
        <ul>
          {portfolios.map(p => (
            <li key={p.id}>{p.name}</li>
          ))}
        </ul>
      )}
    </div>
  )
}
