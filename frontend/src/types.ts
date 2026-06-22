export type AssetType = 'STOCK' | 'CRYPTO' | 'ETF' | 'MUTUAL_FUND';
export type TransactionType = 'BUY' | 'SELL';

export interface Transaction {
  id: number;
  asset_id: number;
  type: TransactionType;
  quantity: number;
  price: number;
  fee: number;
  date: string;
}

export interface Asset {
  id: number;
  symbol: string;
  name: string;
  asset_type: AssetType;
  sector?: string;
  portfolio_id: number;
  currency: string;
  transactions: Transaction[];
}

export interface Portfolio {
  id: number;
  name: string;
  description?: string;
  currency: string;
  assets: Asset[];
}

export interface TaxLot {
  buy_date: string;
  buy_price: number;
  original_qty: number;
  remaining_qty: number;
  latent_gain_loss: number;
  latent_roi: number;
}

export interface AssetTaxSummary {
  symbol: string;
  asset_type: string;
  current_shares: number;
  average_cost: number;
  current_price: number;
  total_cost: number;
  market_value: number;
  unrealized_pnl: number;
  unrealized_roi: number;
  realized_pnl: number;
  tax_lots: TaxLot[];
}

export interface HistoryItem {
  date: string;
  value: number;
  daily_return: number;
  twr: number;
  cash_flow: number;
}

export interface PerformanceMetrics {
  volatility: number;
  sharpe_ratio: number;
  beta: number;
  portfolio_value: number;
  beta_adjusted_exposure: number;
  unrealized_pnl?: number;
  realized_pnl?: number;
}

export interface PortfolioPerformance {
  history: HistoryItem[];
  correlation_matrix: Record<string, Record<string, number>>;
  metrics: PerformanceMetrics;
}

export interface TaxSummary {
  assets: AssetTaxSummary[];
  currency: string;
}
