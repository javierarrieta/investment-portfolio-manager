import { toNumber } from './decimal';

export const formatCurrency = (value: string | number | null | undefined, currencyCode: string = 'USD'): string => {
  const v = toNumber(value);
  const CRYPTO_SYMBOLS: Record<string, string> = {
    'BTC': '₿',
    'ETH': 'Ξ',
  };

  if (CRYPTO_SYMBOLS[currencyCode]) {
    return `${CRYPTO_SYMBOLS[currencyCode]} ${v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}`;
  }

  try {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currencyCode,
    }).format(v);
  } catch {
    return `${currencyCode} ${v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  }
};

export const formatPercent = (value: string | number | null | undefined): string => {
  const v = toNumber(value);
  return `${v >= 0 ? '+' : ''}${(v * 100).toFixed(2)}%`;
};