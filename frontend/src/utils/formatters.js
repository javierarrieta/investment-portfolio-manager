export const formatCurrency = (value, currencyCode = 'USD') => {
  const CRYPTO_SYMBOLS = {
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
  } catch (e) {
    return `${currencyCode} ${value.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  }
};

export const formatPercent = (value) => {
  return `${value >= 0 ? '+' : ''}${(value * 100).toFixed(2)}%`;
};
