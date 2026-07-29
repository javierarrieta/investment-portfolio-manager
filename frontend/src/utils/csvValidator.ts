import type { ColumnMapping, CsvValidationRow, CsvImportResult } from '../types/csv';

function isValidDate(dateStr: string): boolean {
  const trimmed = dateStr.trim();
  if (!trimmed) return false;

  const formats = [
    { regex: /^\d{4}-\d{2}-\d{2}$/, parse: (s: string) => new Date(s + 'T00:00:00Z') },
    { regex: /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$/, parse: (s: string) => new Date(s.replace(' ', 'T') + ':00Z') },
    { regex: /^\d{2}\/\d{2}\/\d{4}$/, parse: (s: string) => {
      const parts = s.split('/');
      return new Date(`${parts[2]}-${parts[0]}-${parts[1]}T00:00:00Z`);
    }},
    { regex: /^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}$/, parse: (s: string) => {
      const [datePart, timePart] = s.split(' ');
      const parts = datePart.split('/');
      return new Date(`${parts[2]}-${parts[0]}-${parts[1]}T${timePart}:00Z`);
    }},
  ];

  for (const fmt of formats) {
    if (fmt.regex.test(trimmed)) {
      const d = fmt.parse(trimmed);
      return !isNaN(d.getTime());
    }
  }

  return false;
}

function parseDate(dateStr: string): Date | null {
  const trimmed = dateStr.trim();

  const formats = [
    { regex: /^\d{4}-\d{2}-\d{2}$/, parse: (s: string) => new Date(s + 'T00:00:00Z') },
    { regex: /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}$/, parse: (s: string) => new Date(s.replace(' ', 'T') + ':00Z') },
    { regex: /^\d{2}\/\d{2}\/\d{4}$/, parse: (s: string) => {
      const parts = s.split('/');
      return new Date(`${parts[2]}-${parts[0]}-${parts[1]}T00:00:00Z`);
    }},
    { regex: /^\d{2}\/\d{2}\/\d{4} \d{2}:\d{2}$/, parse: (s: string) => {
      const [datePart, timePart] = s.split(' ');
      const parts = datePart.split('/');
      return new Date(`${parts[2]}-${parts[0]}-${parts[1]}T${timePart}:00Z`);
    }},
  ];

  for (const fmt of formats) {
    if (fmt.regex.test(trimmed)) {
      const d = fmt.parse(trimmed);
      if (!isNaN(d.getTime())) return d;
    }
  }

  return null;
}

function validateDate(
  dateStr: string,
  row: number
): CsvValidationRow[] {
  const errors: CsvValidationRow[] = [];

  if (!dateStr || !dateStr.trim()) {
    errors.push({ row, field: 'date', message: 'Date is required' });
    return errors;
  }

  if (!isValidDate(dateStr)) {
    errors.push({ row, field: 'date', message: `Invalid date format: "${dateStr}"` });
    return errors;
  }

  const parsed = parseDate(dateStr);
  if (parsed) {
    const year = parsed.getUTCFullYear();
    if (year < 1900 || year > 2100) {
      errors.push({ row, field: 'date', message: `Date year ${year} out of range (1900-2100)` });
    }
  }

  return errors;
}

function validateSymbol(
  symbol: string,
  row: number
): CsvValidationRow[] {
  const errors: CsvValidationRow[] = [];

  if (!symbol || !symbol.trim()) {
    errors.push({ row, field: 'symbol', message: 'Symbol is required' });
  }

  return errors;
}

function validateType(
  typeStr: string,
  row: number
): CsvValidationRow[] {
  const errors: CsvValidationRow[] = [];

  if (!typeStr || !typeStr.trim()) {
    errors.push({ row, field: 'type', message: 'Type is required' });
    return errors;
  }

  const upper = typeStr.trim().toUpperCase();
  if (upper !== 'BUY' && upper !== 'SELL') {
    errors.push({ row, field: 'type', message: `Invalid type "${typeStr}", must be BUY or SELL` });
  }

  return errors;
}

function validateQuantity(
  qtyStr: string,
  row: number
): CsvValidationRow[] {
  const errors: CsvValidationRow[] = [];

  if (!qtyStr || !qtyStr.trim()) {
    errors.push({ row, field: 'quantity', message: 'Quantity is required' });
    return errors;
  }

  const val = parseFloat(qtyStr);
  if (isNaN(val)) {
    errors.push({ row, field: 'quantity', message: `Invalid quantity: "${qtyStr}"` });
    return errors;
  }

  if (val <= 0) {
    errors.push({ row, field: 'quantity', message: `Quantity must be positive, got ${val}` });
  }

  return errors;
}

function validatePrice(
  priceStr: string,
  row: number
): CsvValidationRow[] {
  const errors: CsvValidationRow[] = [];

  if (!priceStr || !priceStr.trim()) {
    errors.push({ row, field: 'price', message: 'Price is required' });
    return errors;
  }

  const val = parseFloat(priceStr);
  if (isNaN(val)) {
    errors.push({ row, field: 'price', message: `Invalid price: "${priceStr}"` });
    return errors;
  }

  if (val <= 0) {
    errors.push({ row, field: 'price', message: `Price must be positive, got ${val}` });
  }

  return errors;
}

function validateFee(
  feeStr: string | undefined,
  row: number
): CsvValidationRow[] {
  const errors: CsvValidationRow[] = [];

  if (!feeStr || !feeStr.trim()) {
    return errors;
  }

  const val = parseFloat(feeStr);
  if (isNaN(val)) {
    errors.push({ row, field: 'fee', message: `Invalid fee: "${feeStr}"` });
    return errors;
  }

  if (val < 0) {
    errors.push({ row, field: 'fee', message: `Fee must be non-negative, got ${val}` });
  }

  return errors;
}

export interface ValidatedRow {
  row: number;
  date: string;
  symbol: string;
  type: 'BUY' | 'SELL';
  quantity: number;
  price: number;
  fee: number;
  errors: CsvValidationRow[];
}

export interface ValidationReport {
  valid: boolean;
  totalRows: number;
  validRows: number;
  errorRows: CsvValidationRow[];
  unknownSymbols: string[];
  validatedRows: ValidatedRow[];
}

export function validateCsvRow(
  rowData: Record<string, string>,
  columnMapping: ColumnMapping,
  portfolioSymbols: string[]
): ValidatedRow {
  const getMapped = (field: string): string | undefined => {
    for (const [idx, mappedField] of Object.entries(columnMapping)) {
      if (mappedField === field) {
        const keys = Object.keys(rowData);
        const key = keys[parseInt(idx, 10)];
        if (key !== undefined) return rowData[key];
      }
    }
    return undefined;
  };

  const dateRaw = getMapped('date') ?? '';
  const symbolRaw = getMapped('symbol') ?? '';
  const typeRaw = getMapped('type') ?? '';
  const qtyRaw = getMapped('quantity') ?? '';
  const priceRaw = getMapped('price') ?? '';
  const feeRaw = getMapped('fee');

  const row = parseInt(Object.keys(rowData)[0] ?? '0', 10);
  const allErrors: CsvValidationRow[] = [];

  allErrors.push(...validateDate(dateRaw, row));
  allErrors.push(...validateSymbol(symbolRaw, row));
  allErrors.push(...validateType(typeRaw, row));
  allErrors.push(...validateQuantity(qtyRaw, row));
  allErrors.push(...validatePrice(priceRaw, row));
  allErrors.push(...validateFee(feeRaw, row));

  return {
    row,
    date: dateRaw,
    symbol: symbolRaw.trim(),
    type: (typeRaw.trim().toUpperCase() as 'BUY' | 'SELL') || 'BUY',
    quantity: parseFloat(qtyRaw) || 0,
    price: parseFloat(priceRaw) || 0,
    fee: parseFloat(feeRaw ?? '0') || 0,
    errors: allErrors,
  };
}

export function validateCsvFile(
  rows: Record<string, string>[],
  columnMapping: ColumnMapping,
  portfolioSymbols: string[]
): ValidationReport {
  const allErrors: CsvValidationRow[] = [];
  const validatedRows: ValidatedRow[] = [];
  const unknownSymbols = new Set<string>();
  let validCount = 0;

  for (const rowData of rows) {
    const validated = validateCsvRow(rowData, columnMapping, portfolioSymbols);

    if (validated.errors.length > 0) {
      allErrors.push(...validated.errors);
    } else {
      validCount++;
    }

    const symbolUpper = validated.symbol.toUpperCase();
    if (symbolUpper && !portfolioSymbols.map((s) => s.toUpperCase()).includes(symbolUpper)) {
      unknownSymbols.add(symbolUpper);
    }

    validatedRows.push(validated);
  }

  return {
    valid: allErrors.length === 0,
    totalRows: rows.length,
    validRows: validCount,
    errorRows: allErrors,
    unknownSymbols: Array.from(unknownSymbols),
    validatedRows,
  };
}
