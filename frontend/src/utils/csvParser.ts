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
  const usedFields = new Set<CsvTransactionField>();

  for (let i = 0; i < headers.length; i++) {
    const normalized = headers[i].trim().toLowerCase().replace(/[^a-z0-9_]/g, '');
    const matched = FIELD_ALIASES[normalized] ?? null;
    if (matched !== null && !usedFields.has(matched)) {
      mapping[i] = matched;
      usedFields.add(matched);
    } else {
      mapping[i] = null;
    }
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

export function dateToISO(dateStr: string, format: string): string {
  const trimmed = dateStr.trim();
  if (!trimmed) return '';

  if (format === 'YYYY-MM-DD') {
    return new Date(trimmed + 'T00:00:00Z').toISOString();
  }
  if (format === 'YYYY-MM-DD HH:MM') {
    return new Date(trimmed.replace(' ', 'T') + ':00Z').toISOString();
  }
  const slashParts = trimmed.split(/\s/);
  const datePart = slashParts[0];
  const timePart = slashParts[1] ?? '';
  const mm = datePart.split('/');
  if (mm.length !== 3) return new Date(trimmed).toISOString();
  const [a, b, year] = mm.map((p) => parseInt(p, 10));
  let m: number;
  let d: number;
  if (format.startsWith('DD/MM')) {
    m = b;
    d = a;
  } else {
    m = a;
    d = b;
  }
  if (!timePart) {
    return new Date(`${year}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}T00:00:00Z`).toISOString();
  }
  return new Date(`${year}-${String(m).padStart(2, '0')}-${String(d).padStart(2, '0')}T${timePart}:00Z`).toISOString();
}
