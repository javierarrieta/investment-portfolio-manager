import React, { useState } from 'react';
import { Plus, Trash2, ChevronDown, ChevronUp } from 'lucide-react';
import { formatCurrency, formatPercent } from '../utils/formatters';
import { 
  Portfolio, 
  TaxSummary, 
  Asset, 
  Transaction, 
  TransactionType, 
  AssetType
} from '../types';

function detectCurrencyFromSymbol(symbol: string): string {
  const s = symbol.toUpperCase();
  if (s.endsWith('.DE') || s.endsWith('.F') || s.endsWith('.FR')) return 'EUR';
  if (s.endsWith('.L')) return 'GBP';
  if (s.endsWith('.T')) return 'JPY';
  if (s.endsWith('.HK')) return 'HKD';
  if (s.endsWith('.SX') || s.endsWith('.SW')) return 'CHF';
  if (s.endsWith('.TO')) return 'CAD';
  if (s.endsWith('.AX')) return 'AUD';
  if (s.endsWith('.K')) return 'KRW';
  if (s.includes('USD') || s.includes('BTC') || s.includes('ETH')) return 'USD';
  return 'USD';
}

interface PortfolioDetailProps {
  portfolio: Portfolio;
  taxSummary: TaxSummary;
  onAddAsset: (assetData: Partial<Asset>) => Promise<void>;
  onDeleteAsset: (assetId: number) => Promise<void>;
  onAddTransaction: (assetId: number, txData: Partial<Transaction>) => Promise<void>;
  strategy: string;
  setStrategy: (strategy: string) => void;
  thresholdDays: number;
  setThresholdDays: (days: number) => void;
}

export default function PortfolioDetail({ 
  portfolio, 
  taxSummary, 
  onAddAsset, 
  onDeleteAsset, 
  onAddTransaction, 
  strategy, 
  setStrategy, 
  thresholdDays, 
  setThresholdDays
}: PortfolioDetailProps) {
  const [expandedAsset, setExpandedAsset] = useState<string | null>(null);
  const [showAssetModal, setShowAssetModal] = useState(false);
  const [showTxModal, setShowTxModal] = useState(false);
  
  // Forms State
  const [assetForm, setAssetForm] = useState<Partial<Asset>>({ symbol: '', name: '', asset_type: 'STOCK', sector: '', currency: 'USD' });
  const [txForm, setTxForm] = useState({ asset_id: '', type: 'BUY' as TransactionType, quantity: '', price: '', fee: '0.0', date: new Date().toISOString().slice(0, 16) });
  
  const handleAssetSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!assetForm.symbol || !assetForm.name) return;
    await onAddAsset(assetForm);
    setAssetForm({ symbol: '', name: '', asset_type: 'STOCK', sector: '', currency: 'USD' });
    setShowAssetModal(false);
  };

  const handleTxSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!txForm.asset_id || !txForm.quantity || !txForm.price) return;
    await onAddTransaction(Number(txForm.asset_id), {
      type: txForm.type,
      quantity: parseFloat(txForm.quantity),
      price: parseFloat(txForm.price),
      fee: parseFloat(txForm.fee),
      date: new Date(txForm.date).toISOString()
    });
    setTxForm({ asset_id: '', type: 'BUY', quantity: '', price: '', fee: '0.0', date: new Date().toISOString().slice(0, 16) });
    setShowTxModal(false);
  };

  // Group assets for transaction modal dropdown
  const assets = portfolio.assets || [];
  const selectedTxAsset = assets.find(a => a.id === Number(txForm.asset_id));
  const txCurrency = selectedTxAsset?.currency || portfolio.currency || 'USD';

  return (
    <div className="portfolio-detail-container">
      {/* Portfolio Info / Tax Selector */}
      <div className="glass-card" style={{ padding: '24px', marginBottom: '32px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '20px' }}>
          <div>
            <h2 style={{ fontSize: '1.5rem', marginBottom: '8px' }}>{portfolio.name}</h2>
            <p style={{ color: 'var(--text-secondary)' }}>{portfolio.description || 'No description provided.'}</p>
          </div>

          {/* Tax Strategy Selector */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px', background: 'rgba(255,255,255,0.03)', padding: '12px 20px', borderRadius: '12px', border: '1px solid var(--border-color)' }}>
            <div style={{ display: 'flex', flexDirection: 'column' }}>
              <label style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginBottom: '4px', fontWeight: 600 }}>TAX ACCOUNTING METHOD</label>
              <select 
                value={strategy} 
                onChange={(e) => setStrategy(e.target.value)} 
                className="form-control"
                style={{ background: 'transparent', border: 'none', padding: '0', cursor: 'pointer', fontWeight: 600, color: 'var(--color-primary)' }}
              >
                <option value="FIFO">FIFO (First-In, First-Out)</option>
                <option value="LIFO">LIFO (Last-In, First-Out)</option>
                <option value="HYBRID">HYBRID (Short-Term LIFO, Long-Term FIFO)</option>
              </select>
            </div>

            {strategy === 'HYBRID' && (
              <div style={{ display: 'flex', flexDirection: 'column', width: '80px', borderLeft: '1px solid var(--border-color)', paddingLeft: '16px' }}>
                <label style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginBottom: '4px', fontWeight: 600 }}>THRESHOLD</label>
                <input 
                  type="number" 
                  value={thresholdDays} 
                  onChange={(e) => setThresholdDays(parseInt(e.target.value) || 30)} 
                  className="form-control"
                  style={{ background: 'transparent', border: 'none', padding: '0', fontWeight: 600, width: '100%' }}
                />
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Action Buttons Row */}
      <div style={{ display: 'flex', gap: '12px', marginBottom: '24px' }}>
        <button onClick={() => setShowAssetModal(true)} className="btn btn-primary">
          <Plus size={16} /> Add Asset Symbol
        </button>
        {assets.length > 0 && (
          <button onClick={() => setShowTxModal(true)} className="btn btn-secondary">
            <Plus size={16} /> Log Transaction
          </button>
        )}
      </div>

      {/* Assets Table */}
      <div className="glass-card" style={{ padding: '24px', overflowX: 'auto' }}>
        <h3 style={{ marginBottom: '20px' }}>Holdings Breakdown</h3>
        {taxSummary.assets.length === 0 ? (
          <p style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '24px 0' }}>No assets registered yet. Click "Add Asset Symbol" to begin.</p>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th style={{ width: '40px' }}></th>
                <th>Asset / Symbol</th>
                <th>Type</th>
                <th>Holdings</th>
                <th>Avg. Cost</th>
                <th>Price</th>
                <th>Market Value</th>
                <th>Realized P&L</th>
                <th>Unrealized P&L</th>
                <th>Latent ROI</th>
                <th style={{ width: '60px' }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {taxSummary.assets.map((a) => {
                const isExpanded = expandedAsset === a.symbol;
                const assetModel = assets.find(am => am.symbol === a.symbol);
                return (
                  <React.Fragment key={a.symbol}>
                    <tr>
                      <td>
                        <button 
                          onClick={() => setExpandedAsset(isExpanded ? null : a.symbol)}
                          style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-secondary)' }}
                        >
                          {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
                        </button>
                      </td>
                      <td>
                        <div style={{ fontWeight: 600 }}>{a.symbol}</div>
                        <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>{assetModel?.name || ''}</div>
                      </td>
                      <td>
                        <span className={`badge badge-${a.asset_type.toLowerCase() === 'crypto' ? 'crypto' : a.asset_type.toLowerCase() === 'etf' ? 'etf' : 'stock'}`}>
                          {a.asset_type}
                        </span>
                      </td>
                      <td>{a.current_shares.toLocaleString(undefined, { maximumFractionDigits: 6 })}</td>
                      <td>{formatCurrency(a.average_cost, assetModel?.currency || 'USD')}</td>
                      <td>{formatCurrency(a.current_price, assetModel?.currency || 'USD')}</td>
                      <td style={{ fontWeight: 600 }}>{formatCurrency(a.market_value, assetModel?.currency || 'USD')}</td>
                      <td style={{ color: a.realized_pnl >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
                        {formatCurrency(a.realized_pnl, assetModel?.currency || 'USD')}
                      </td>
                      <td style={{ color: a.unrealized_pnl >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
                        {formatCurrency(a.unrealized_pnl, assetModel?.currency || 'USD')}
                      </td>
                      <td style={{ fontWeight: 600, color: a.unrealized_roi >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
                        {formatPercent(a.unrealized_roi)}
                      </td>
                      <td>
                        {assetModel && (
                          <button 
                            onClick={() => onDeleteAsset(assetModel.id)} 
                            style={{ background: 'transparent', border: 'none', cursor: 'pointer', color: 'var(--text-muted)' }}
                            title="Delete Asset"
                          >
                            <Trash2 size={16} />
                          </button>
                        )}
                      </td>
                    </tr>

                    {/* Expanded Tax Lots Row */}
                    {isExpanded && (
                      <tr>
                        <td colSpan={11} style={{ padding: '0 16px 24px 56px', background: 'rgba(255,255,255,0.01)' }}>
                          <div style={{ padding: '16px', background: 'rgba(10, 15, 28, 0.5)', borderRadius: '12px', border: '1px solid var(--border-color)' }}>
                            <h4 style={{ fontSize: '0.875rem', marginBottom: '12px', color: 'var(--color-primary)' }}>Tax Lots (Unrealized Profit Details)</h4>
                            {a.tax_lots.length === 0 ? (
                              <p style={{ fontSize: '0.875rem', color: 'var(--text-muted)' }}>No open tax lots for this asset (fully sold or no transactions).</p>
                            ) : (
                              <table className="data-table" style={{ fontSize: '0.875rem' }}>
                                <thead>
                                  <tr>
                                    <th>Buy Date</th>
                                    <th>Buy Price</th>
                                    <th>Remaining Qty</th>
                                    <th>Original Qty</th>
                                    <th>Latent Profit/Loss</th>
                                    <th>Latent ROI</th>
                                  </tr>
                                </thead>
                                <tbody>
                                  {a.tax_lots.map((lot, index) => (
                                    <tr key={index}>
                                      <td>{new Date(lot.buy_date).toLocaleDateString()}</td>
                                      <td>{formatCurrency(lot.buy_price, assetModel?.currency || 'USD')}</td>
                                      <td>{lot.remaining_qty.toLocaleString(undefined, { maximumFractionDigits: 6 })}</td>
                                      <td>{lot.original_qty.toLocaleString(undefined, { maximumFractionDigits: 6 })}</td>
                                      <td style={{ color: lot.latent_gain_loss >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
                                        {formatCurrency(lot.latent_gain_loss, assetModel?.currency || 'USD')}
                                      </td>
                                      <td style={{ color: lot.latent_roi >= 0 ? 'var(--color-success)' : 'var(--color-danger)' }}>
                                        {formatPercent(lot.latent_roi)}
                                      </td>
                                    </tr>
                                  ))}
                                </tbody>
                              </table>
                            )}
                          </div>
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                );
              })}
            </tbody>
          </table>
        )}
      </div>

      {/* ADD ASSET MODAL */}
      {showAssetModal && (
        <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(4px)', display: 'flex', justifyContent: 'center', alignItems: 'center', zIndex: 100 }}>
          <div className="glass-card" style={{ padding: '32px', width: '450px', background: '#121929' }}>
            <h3 style={{ marginBottom: '20px' }}>Register New Asset Symbol</h3>
            <form onSubmit={handleAssetSubmit}>
              <div className="form-group">
                <label>Symbol (e.g. AAPL, BTC-USD, VOO)</label>
                <input 
                  type="text" 
                  value={assetForm.symbol} 
                  onChange={(e) => {
                    const upper = e.target.value.toUpperCase();
                    const detected = detectCurrencyFromSymbol(upper);
                    setAssetForm(prev => ({ ...prev, symbol: upper, currency: detected }));
                  }}
                  className="form-control"
                  required 
                />
              </div>
              <div className="form-group">
                <label>Name (e.g. Apple Inc., Bitcoin, Vanguard S&P 500)</label>
                <input 
                  type="text" 
                  value={assetForm.name} 
                  onChange={(e) => setAssetForm({ ...assetForm, name: e.target.value })} 
                  className="form-control"
                  required 
                />
              </div>
              <div className="form-group">
                <label>Asset Type</label>
                <select 
                  value={assetForm.asset_type} 
                   onChange={(e) => setAssetForm({ ...assetForm, asset_type: e.target.value as AssetType })} 

                  className="form-control"
                >
                  <option value="STOCK">Stock</option>
                  <option value="CRYPTO">Crypto</option>
                  <option value="ETF">ETF</option>
                  <option value="MUTUAL_FUND">Mutual Fund</option>
                </select>
              </div>
              <div className="form-group">
                <label>Currency</label>
                <select 
                  value={assetForm.currency} 
                  onChange={(e) => setAssetForm({ ...assetForm, currency: e.target.value })} 
                  className="form-control"
                >
                  <option value="USD">USD</option>
                  <option value="EUR">EUR</option>
                  <option value="GBP">GBP</option>
                  <option value="JPY">JPY</option>
                  <option value="BTC">BTC</option>
                  <option value="ETH">ETH</option>
                </select>
              </div>
              <div className="form-group">
                <label>Sector (Optional)</label>
                <input 
                  type="text" 
                  value={assetForm.sector} 
                  onChange={(e) => setAssetForm({ ...assetForm, sector: e.target.value })} 
                  className="form-control" 
                />
              </div>
              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px', marginTop: '24px' }}>
                <button type="button" onClick={() => { setShowAssetModal(false); setAssetForm({ symbol: '', name: '', asset_type: 'STOCK' as AssetType, sector: '', currency: 'USD' }); }} className="btn btn-secondary">Cancel</button>
                <button type="submit" className="btn btn-primary">Add Symbol</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* LOG TRANSACTION MODAL */}
      {showTxModal && (
        <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(4px)', display: 'flex', justifyContent: 'center', alignItems: 'center', zIndex: 100 }}>
          <div className="glass-card" style={{ padding: '32px', width: '450px', background: '#121929' }}>
            <h3 style={{ marginBottom: '20px' }}>Log Buy/Sell Transaction</h3>
            <form onSubmit={handleTxSubmit}>
              <div className="form-group">
                <label>Select Asset</label>
                <select 
                  value={txForm.asset_id} 
                  onChange={(e) => setTxForm({ ...txForm, asset_id: e.target.value })} 
                  className="form-control"
                  required
                >
                  <option value="">-- Choose Asset --</option>
                  {assets.map(a => (
                    <option key={a.id} value={a.id}>{a.symbol} ({a.name})</option>
                  ))}
                </select>
              </div>
              <div className="form-group">
                <label>Transaction Type</label>
                <select 
                  value={txForm.type} 
                  onChange={(e) => setTxForm({ ...txForm, type: e.target.value as TransactionType })} 
                  className="form-control"
                >
                  <option value="BUY">BUY</option>
                  <option value="SELL">SELL</option>
                </select>
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
                <div className="form-group">
                  <label>Quantity</label>
                  <input 
                    type="number" 
                    step="any" 
                    value={txForm.quantity} 
                    onChange={(e) => setTxForm({ ...txForm, quantity: e.target.value })} 
                    className="form-control"
                    required 
                  />
                </div>
                <div className="form-group">
                  <label>Price ({txCurrency})</label>
                  <input 
                    type="number" 
                    step="any" 
                    value={txForm.price} 
                    onChange={(e) => setTxForm({ ...txForm, price: e.target.value })} 
                    className="form-control"
                    required 
                  />
                </div>
              </div>
              <div className="form-group">
                <label>Transaction Fee ({txCurrency})</label>
                <input 
                  type="number" 
                  step="any" 
                  value={txForm.fee} 
                  onChange={(e) => setTxForm({ ...txForm, fee: e.target.value })} 
                  className="form-control" 
                />
              </div>
              <div className="form-group">
                <label>Date & Time</label>
                <input 
                  type="datetime-local" 
                  value={txForm.date} 
                  onChange={(e) => setTxForm({ ...txForm, date: e.target.value })} 
                  className="form-control"
                  required 
                />
              </div>
              <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px', marginTop: '24px' }}>
                <button type="button" onClick={() => setShowTxModal(false)} className="btn btn-secondary">Cancel</button>
                <button type="submit" className="btn btn-primary">Log Transaction</button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
