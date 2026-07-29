export type CsvTransactionField = 'date' | 'symbol' | 'type' | 'quantity' | 'price' | 'fee' | null;

export interface ColumnMapping {
  [columnIndex: number]: CsvTransactionField;
}

export interface CsvValidationRow {
  row: number;
  field: string;
  message: string;
}

export interface CsvImportResult {
  row: number;
  success: boolean;
  error?: string;
  assetId?: number;
}
