import React from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';
import { DollarSign, ArrowUpRight, ArrowDownRight, Briefcase, TrendingUp } from 'lucide-react';
import { formatCurrency, formatPercent } from '../utils/formatters';

const COLORS = ['#6366f1', '#a855f7', '#f59e0b', '#10b981', '#ec4899'];

export default function Dashboard({ performance, taxSummary, portfolioName }) {
  const [mounted, setMounted] = React.useState(false);
  
  React.useEffect(() => {
    setMounted(true);
  }, []);

  if (!performance || !taxSummary || performance.history.length === 0) {
    return (
      <div className="glass-card" style={{ padding: '48px', textAlign: 'center' }}>
        <Briefcase size={48} style={{ color: 'var(--text-secondary)', marginBottom: '16px' }} />
        <h2>No Active Assets Found</h2>
        <p style={{ color: 'var(--text-secondary)', marginTop: '8px' }}>
          Add assets and transactions to this portfolio to begin tracking historical performance.
        </p>
      </div>
    );
  }

  const { metrics, history } = performance;
  const totalValue = metrics.portfolio_value;
  const realizedPnl = metrics.realized_pnl;
  const unrealizedPnl = metrics.unrealized_pnl;
  const totalGains = realizedPnl + unrealizedPnl;
  
  // Prepare allocation data
  const allocationMap = {};
  taxSummary.assets.forEach(a => {
    const type = a.asset_type || 'STOCK';
    allocationMap[type] = (allocationMap[type] || 0) + a.market_value;
  });

  const allocationData = Object.keys(allocationMap).map(type => ({
    name: type,
    value: allocationMap[type]
  })).filter(item => item.value > 0);

  // Calculate overall performance ROI
  const totalCost = totalValue - unrealizedPnl;
  const totalRoi = totalCost > 0 ? (unrealizedPnl / totalCost) : 0.0;

  return (
    <div>
      {/* Metrics Row */}
      <div className="metrics-grid">
        <div className="glass-card metric-card">
          <div className="metric-label">Net Asset Value</div>
          <div className="metric-value">{formatCurrency(totalValue, taxSummary?.currency || 'USD')}</div>
          <div className="metric-change positive">
            <DollarSign size={14} /> Live Market Prices
          </div>
        </div>

        <div className="glass-card metric-card">
          <div className="metric-label">Total Realized P&L</div>
          <div className="metric-value" style={{ color: realizedPnl >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
            {formatCurrency(realizedPnl, taxSummary?.currency || 'USD')}
          </div>
          <div className={`metric-change ${realizedPnl >= 0 ? 'positive' : 'negative'}`}>
            {realizedPnl >= 0 ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />} Realized Tax Gains
          </div>
        </div>

        <div className="glass-card metric-card">
          <div className="metric-label">Latent P&L (Unrealized)</div>
          <div className="metric-value" style={{ color: unrealizedPnl >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
            {formatCurrency(unrealizedPnl, taxSummary?.currency || 'USD')}
          </div>
          <div className={`metric-change ${unrealizedPnl >= 0 ? 'positive' : 'negative'}`}>
            {unrealizedPnl >= 0 ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />}
            {formatPercent(totalRoi)} Latent ROI
          </div>
        </div>

        <div className="glass-card metric-card">
          <div className="metric-label">Total Portfolio Return</div>
          <div className="metric-value" style={{ color: totalGains >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
            {formatCurrency(totalGains, taxSummary?.currency || 'USD')}
          </div>
          <div className={`metric-change ${totalGains >= 0 ? 'positive' : 'negative'}`}>
            {totalGains >= 0 ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />} Cumulative P&L
          </div>
        </div>
      </div>

      {/* Charts Row */}
      <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '24px', marginBottom: '32px' }}>
        {/* Performance Chart */}
        <div className="glass-card chart-card" style={{ height: '420px' }}>
          <h3 style={{ marginBottom: '16px', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <TrendingUp size={18} style={{ color: 'var(--color-primary)' }} /> Historical Performance (Time-Weighted)
          </h3>
          <div className="chart-container-inner" style={{ height: '320px', minWidth: 0 }}>
            {mounted && (
              <ResponsiveContainer width="100%" height="100%" minWidth={0}>
                <AreaChart data={history}>
                  <defs>
                    <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="var(--color-primary)" stopOpacity={0.3}/>
                      <stop offset="95%" stopColor="var(--color-primary)" stopOpacity={0}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.03)" />
                  <XAxis dataKey="date" stroke="var(--text-muted)" fontSize={12} tickLine={false} />
                  <YAxis 
                    stroke="var(--text-muted)" 
                    fontSize={12} 
                    tickLine={false} 
                    tickFormatter={(val) => `$${val}`}
                  />
                  <Tooltip 
                    contentStyle={{ 
                      background: '#121929', 
                      border: '1px solid var(--border-color)', 
                      borderRadius: '8px',
                      color: 'var(--text-primary)'
                    }}
                    formatter={(value, name) => {
                      if (name === 'value') return [formatCurrency(value, taxSummary?.currency || 'USD'), 'Portfolio Value'];
                      if (name === 'twr') return [formatPercent(value), 'Time-Weighted Return'];
                      return [value, name];
                    }}
                  />
                  <Area type="monotone" dataKey="value" stroke="var(--color-primary)" strokeWidth={2} fillOpacity={1} fill="url(#colorValue)" />
                </AreaChart>
              </ResponsiveContainer>
            )}
          </div>
        </div>

        {/* Asset Allocation */}
        <div className="glass-card chart-card" style={{ height: '420px', display: 'flex', flexDirection: 'column' }}>
          <h3 style={{ marginBottom: '16px' }}>Asset Allocation</h3>
          <div style={{ flexGrow: 1, position: 'relative', display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
            {mounted && allocationData.length > 0 ? (
              <ResponsiveContainer width="100%" height={240} minWidth={0}>
                <PieChart>
                  <Pie
                    data={allocationData}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={80}
                    paddingAngle={4}
                    dataKey="value"
                  >
                    {allocationData.map((entry, index) => (
                      <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip 
                    contentStyle={{ 
                      background: '#121929', 
                      border: '1px solid var(--border-color)', 
                      borderRadius: '8px'
                    }}
                    formatter={(value) => formatCurrency(value, taxSummary?.currency || 'USD')}
                  />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <div style={{ color: 'var(--text-muted)' }}>{!mounted ? 'Loading...' : 'No allocation data'}</div>
            )}
          </div>
          {/* Legend */}
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', justifyContent: 'center', marginTop: '16px' }}>
            {allocationData.map((item, index) => (
              <div key={item.name} style={{ display: 'flex', alignItems: 'center', gap: '6px', fontSize: '0.875rem' }}>
                <span style={{ width: '10px', height: '10px', borderRadius: '50%', backgroundColor: COLORS[index % COLORS.length] }}></span>
                <span style={{ color: 'var(--text-secondary)' }}>{item.name}:</span>
                <span style={{ fontWeight: 600 }}>{((item.value / totalValue) * 100).toFixed(0)}%</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
