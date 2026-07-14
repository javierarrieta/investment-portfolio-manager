import React from 'react';
import { ShieldAlert, Info, TrendingUp, Flame } from 'lucide-react';
import { PortfolioPerformance } from '../types';

export default function AnalyticsView({ performance, currency = 'USD' }: { performance: PortfolioPerformance | null; currency?: string }) {
  if (!performance || !performance.metrics || performance.history.length === 0) {
    return (
      <div className="glass-card" style={{ padding: '48px', textAlign: 'center' }}>
        <Flame size={48} style={{ color: 'var(--text-secondary)', marginBottom: '16px' }} />
        <h2>Advanced Analytics Stale</h2>
        <p style={{ color: 'var(--text-secondary)', marginTop: '8px' }}>
          Add multiple assets with price histories to view correlations and risk statistics.
        </p>
      </div>
    );
  }

  const { metrics, correlation_matrix } = performance;

  const formatPercent = (val: number) => {
    return `${(val * 100).toFixed(2)}%`;
  };

  const formatCurrency = (val: number) => {
    const code = currency && currency.length > 0 ? currency : 'USD';
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: code }).format(val);
  };

  // Get color for correlation cells
  const getCellColor = (val: number | undefined | null) => {
    if (val === undefined || val === null) return 'rgba(255,255,255,0.05)';
    // Scale color from red (-1) to dark slate (0) to blue/purple (+1)
    if (val > 0) {
      return `rgba(99, 102, 241, ${val})`; // Indigo tint based on correlation
    } else {
      return `rgba(239, 68, 68, ${Math.abs(val)})`; // Crimson tint based on negative correlation
    }
  };

  // Get symbols for the matrix
  const symbols = Object.keys(correlation_matrix || {});

  return (
    <div className="analytics-container">
      {/* Risk Metrics Cards */}
      <div className="metrics-grid">
        <div className="glass-card metric-card">
          <div className="metric-label" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            Annualized Volatility <span title="Measure of standard deviation of portfolio daily returns multiplied by sqrt(252). Indicates general portfolio price variability."><Info size={14} /></span>
          </div>
          <div className="metric-value">{formatPercent(metrics.volatility || 0.0)}</div>
          <div className="metric-change" style={{ color: (metrics.volatility || 0.0) < 0.25 ? 'var(--color-success)' : 'var(--color-warning)' }}>
            {(metrics.volatility || 0) < 0.25 ? 'Moderate Risk' : 'High Risk Profile'}
          </div>
        </div>

        <div className="glass-card metric-card">
          <div className="metric-label" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            Sharpe Ratio <span title="Excess return per unit of volatility. Risk-free rate assumed at 2%. Values > 1.0 are considered good, > 2.0 very good."><Info size={14} /></span>
          </div>
          <div className="metric-value">{(metrics.sharpe_ratio || 0.0).toFixed(2)}</div>
          <div className="metric-change" style={{ color: (metrics.sharpe_ratio || 0) >= 1.0 ? 'var(--color-success)' : 'var(--text-muted)' }}>
            {(metrics.sharpe_ratio || 0) >= 1.0 ? 'Strong Risk-Adjusted Returns' : 'Sub-optimal Return/Risk'}
          </div>
        </div>

        <div className="glass-card metric-card">
          <div className="metric-label" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            Portfolio Beta <span title="Sensitivity of the portfolio to market movements (SPY benchmark). Beta > 1 is more volatile than market; Beta < 1 is less volatile."><Info size={14} /></span>
          </div>
          <div className="metric-value">{(metrics.beta || 1.0).toFixed(2)}</div>
          <div className="metric-change" style={{ color: Math.abs((metrics.beta || 1) - 1.0) < 0.2 ? 'var(--text-primary)' : 'var(--color-secondary)' }}>
            {(metrics.beta || 1) > 1.1 ? 'Market Amplifier' : (metrics.beta || 1) < 0.9 ? 'Defensive Structure' : 'Market Tracker'}
          </div>
        </div>

        <div className="glass-card metric-card">
          <div className="metric-label" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            Beta-Adjusted Net Exposure <span title="Equivalent exposure of the portfolio compared to SPY. Formula: Portfolio Value * Portfolio Beta."><Info size={14} /></span>
          </div>
          <div className="metric-value">{formatCurrency(metrics.beta_adjusted_exposure || 0.0)}</div>
          <div className="metric-change positive">
            Benchmark Equivalent Risk
          </div>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1.2fr 1fr', gap: '24px' }}>
        {/* Correlation Heatmap */}
        <div className="glass-card" style={{ padding: '24px' }}>
          <h3 style={{ marginBottom: '8px' }}>Asset Correlation Matrix</h3>
          <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem', marginBottom: '24px' }}>
            Measures the relative price return movements of assets. High correlation (+1) reduces diversification benefits; low/negative correlation provides safety.
          </p>
          
          {symbols.length === 0 ? (
            <p style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '24px 0' }}>No correlation data available. Add at least two assets to view the heatmap.</p>
          ) : (
            <div style={{ overflowX: 'auto' }}>
              <table style={{ borderCollapse: 'collapse', width: '100%' }}>
                <thead>
                  <tr>
                    <th style={{ padding: '8px', borderBottom: '1px solid var(--border-color)' }}></th>
                    {symbols.map(s => (
                      <th key={s} style={{ padding: '8px', borderBottom: '1px solid var(--border-color)', fontWeight: 600, fontSize: '0.875rem', textAlign: 'center' }}>
                        {s}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {symbols.map(s1 => (
                    <tr key={s1}>
                      <td style={{ padding: '12px 8px', fontWeight: 600, fontSize: '0.875rem', borderBottom: '1px solid rgba(255,255,255,0.03)' }}>{s1}</td>
                      {symbols.map(s2 => {
                        const val = correlation_matrix[s1][s2];
                        return (
                          <td 
                            key={s2} 
                            style={{ 
                              padding: '12px 8px',
                              textAlign: 'center',
                              borderBottom: '1px solid rgba(255,255,255,0.03)',
                              backgroundColor: getCellColor(val),
                              transition: 'background-color 0.2s'
                            }}
                          >
                            <span 
                              style={{ 
                                fontWeight: 700, 
                                fontSize: '0.875rem',
                                color: '#ffffff',
                                textShadow: '0 1px 2px rgba(0,0,0,0.5)'
                              }}
                            >
                              {val !== undefined ? val.toFixed(2) : '-'}
                            </span>
                          </td>
                        );
                      })}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Portfolio Optimization Rules */}
        <div className="glass-card" style={{ padding: '24px' }}>
          <h3 style={{ marginBottom: '16px' }}>Risk Assessment & Notes</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div style={{ display: 'flex', gap: '12px', background: 'rgba(255,255,255,0.02)', padding: '16px', borderRadius: '12px', border: '1px solid var(--border-color)' }}>
              <ShieldAlert size={20} style={{ color: 'var(--color-warning)', flexShrink: 0 }} />
              <div>
                <h4 style={{ fontSize: '0.875rem', fontWeight: 600, marginBottom: '4px' }}>Diversification Score</h4>
                <p style={{ fontSize: '0.875rem', color: 'var(--text-secondary)', lineHeight: '1.4' }}>
                  Assets like Crypto tend to have low correlation with general stocks/funds, which can increase risk-adjusted efficiency (Sharpe Ratio). Monitor any correlation above 0.70 closely.
                </p>
              </div>
            </div>
            <div style={{ display: 'flex', gap: '12px', background: 'rgba(255,255,255,0.02)', padding: '16px', borderRadius: '12px', border: '1px solid var(--border-color)' }}>
              <TrendingUp size={20} style={{ color: 'var(--color-primary)', flexShrink: 0 }} />
              <div>
                <h4 style={{ fontSize: '0.875rem', fontWeight: 600, marginBottom: '4px' }}>Beta Adjustment Guidance</h4>
                <p style={{ fontSize: '0.875rem', color: 'var(--text-secondary)', lineHeight: '1.4' }}>
                  If your Beta-Adjusted Net Exposure is higher than your absolute portfolio value, your portfolio has high-risk sensitivity and will move faster than standard stock market benchmarks (e.g. S&P 500).
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
