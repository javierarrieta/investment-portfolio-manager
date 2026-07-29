import type { ColumnMapping, CsvTransactionField } from '../types/csv';

const FIELD_ALIASES: Record<string, CsvTransactionField> = {
  date: 'date',
  datetime: 'date',
  txn_date: 'date',
  symbol: 'symbol',
  ticker: 'symbol',
  asset: 'symbol',
  type: 'type',
  side: 'type',
  txn_type: 'type',
  qty: 'quantity',
  quantity: 'quantity',
  shares: 'quantity',
  amount: 'quantity',
  price: 'price',
  unit_price: 'price',
  cost: 'price',
  fee: 'fee',
  commission: 'fee',
  cost_base: 'fee',
};

export function detectColumns(headers: string[]): ColumnMapping {
  const mapping: ColumnMapping = {};

  for (let i = 0; i < headers.length; i++) {
    const normalized = headers[i].trim().toLowerCase().replace(/[^a-z0-9_]/g, '');
    mapping[i] = FIELD_ALIASES[normalized] ?? null;
  }

  return mapping;
}

const DATE_FORMATS = [
  { format: 'YYYY-MM-DD', regex: /^\d{4}-\d{2}-\d{2}$/ },
  { format: 'YYYY-MM-DD HH:MM', regex: /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$/ },
  { format: 'MM/DD/YYYY', regex: /^\d{2}\/\d{2}\/\d{4}$/ },
  { format: 'DD/MM/YYYY', regex: /^\d{2}\/\d{2}\/\d{4}$/ },
  { format: 'MM/DD/YYYY HH:MM', regex: /^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}$/ },
  { format: 'DD/MM/YYYY HH:MM', regex: /^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}$/ },
] as const;

export function detectDateFormat(
  sampleRows: string[][]
): { format: string; ambiguous: boolean } {
  for (const row of sampleRows) {
    for (const cell of row) {
      const trimmed = cell.trim();
      for (const df of DATE_FORMATS) {
        if (df.regex.test(trimmed)) {
          if (df.format === 'MM/DD/YYYY' || df.format === 'DD/MM/YYYY' ||
              df.format === 'MM/DD/YYYY HH:MM' || df.format === 'DD/MM/YYYY HH:MM') {
            const parts = trimmed.split(/[/\s]/).filter(p => /^\d{2}$/.test(p));
            if (parts.length >= 2) {
              const first = parseInt(parts[0], 10);
              if (first > 12) {
                return { format: df.format.startsWith('DD') ? df.format : 'DD/MM/YYYY', ambiguous: true };
              }
              return { format: df.format.startsWith('MM') ? df.format : 'MM/DD/YYYY', ambiguous: true };
            }
          }
          const isAmbiguous =
            df.format === 'MM/DD/YYYY' || df.format === 'DD/MM/YYYY' ||
            df.format === 'MM/DD/YYYY HH:MM' || df.format === 'DD/MM/YYYY HH:MM';
          return { format: df.format, ambiguous: isAmbiguous };
        }
      }
    }
  }

  return { format: 'YYYY-MM-DD', ambiguous: false };
}
