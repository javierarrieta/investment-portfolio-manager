export const formatCurrency = (value: number, currencyCode: string = 'USD'): string => {
  const CRYPTO_SYMBOLS: Record<string, string> = {
    'BTC': '₿',
    'ETH': 'Ξ',
  };

  if (CRYPTO_SYMBOLS[currencyCode]) {
    return `${CRYPTO_SYMBOLS[currencyCode]} ${value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 })}`;
  }

  try {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: currencyCode,
    }).format(value);
  } catch {
    return `${currencyCode} ${value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  }
};

export const formatPercent = (value: number): string => {
  return `${value >= 0 ? '+' : ''}${(value * 100).toFixed(2)}%`;
};
