import React, { useState, useCallback } from 'react';
import { X, Upload, ChevronRight, ChevronLeft, CheckCircle, XCircle, AlertTriangle } from 'lucide-react';
import Papa from 'papaparse';
import { Asset } from '../types';
import { detectColumns, detectDateFormat, dateToISO } from '../utils/csvParser';
import { validateCsvFile } from '../utils/csvValidator';
import type { ColumnMapping, CsvImportResult, CsvTransactionField } from '../types/csv';
import type { ValidationReport } from '../utils/csvValidator';

interface CsvImportModalProps {
  portfolioId: number;
  assets: Asset[];
  onClose: () => void;
  onImportComplete: () => void;
}

type ImportStep = 'upload' | 'map' | 'validate' | 'confirm' | 'results';

export default function CsvImportModal({ portfolioId, assets, onClose, onImportComplete }: CsvImportModalProps) {
  const [step, setStep] = useState<ImportStep>('upload');
  const [csvRows, setCsvRows] = useState<Record<string, string>[]>([]);
  const [headers, setHeaders] = useState<string[]>([]);
  const [columnMapping, setColumnMapping] = useState<ColumnMapping>({});
  const [dateFormat, setDateFormat] = useState<string>('YYYY-MM-DD');
  const [dateAmbiguous, setDateAmbiguous] = useState(false);
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null);
  const [importResults, setImportResults] = useState<CsvImportResult[]>([]);
  const [overrideFormat, setOverrideFormat] = useState<string>('');
  const [isImporting, setIsImporting] = useState(false);

  const portfolioSymbols = assets.map((a) => a.symbol);

  const handleFileUpload = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    Papa.parse(file, {
      header: true,
      skipEmptyLines: true,
      complete: (results: Papa.ParseResult<Record<string, string>>) => {
        const data = results.data as Record<string, string>[];
        const parsedHeaders = results.meta.fields ?? [];
        setHeaders(parsedHeaders);
        setCsvRows(data);
        setStep('map');

        const cols = detectColumns(parsedHeaders);
        setColumnMapping(cols);

        const dateResult = detectDateFormat(data.slice(0, 5).map((row) => parsedHeaders.map((h: string) => (row[h] ?? ''))));
        setDateFormat(dateResult.format);
        setDateAmbiguous(dateResult.ambiguous);
      },
    });
  }, []);

  const handleColumnChange = useCallback((colIdx: number, field: CsvTransactionField) => {
    setColumnMapping((prev: ColumnMapping) => ({ ...prev, [colIdx]: field }));
  }, []);

  const handleValidate = useCallback(() => {
    const report = validateCsvFile(csvRows, columnMapping, portfolioSymbols);
    setValidationReport(report);
    setStep('validate');
  }, [csvRows, columnMapping, portfolioSymbols]);

  const handleConfirmImport = useCallback(async () => {
    if (!validationReport) return;

    setIsImporting(true);
    const results: CsvImportResult[] = [];
    const importedKeys = new Set<string>();

    for (const row of validationReport.validatedRows) {
      if (row.errors.length > 0) {
        results.push({ row: row.row, success: false, error: row.errors.map((e) => e.message).join('; ') });
        continue;
      }

      let asset = assets.find((a) => a.symbol.toUpperCase() === row.symbol.toUpperCase());

      if (!asset) {
        try {
          const res = await fetch(`/api/portfolios/${portfolioId}/assets`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              symbol: row.symbol.toUpperCase(),
              name: row.symbol,
              asset_type: 'STOCK',
              currency: 'USD',
            }),
          });
          if (!res.ok) {
            const existing = assets.find((a) => a.symbol.toUpperCase() === row.symbol.toUpperCase());
            if (existing) {
              asset = existing;
            } else {
              try {
                const portfolioRes = await fetch(`/api/portfolios/${portfolioId}`);
                if (portfolioRes.ok) {
                  const portfolio = (await portfolioRes.json()) as { assets: Asset[] };
                  const found = portfolio.assets.find((a) => a.symbol.toUpperCase() === row.symbol.toUpperCase());
                  if (found) {
                    asset = found;
                  }
                }
              } catch {
                // ignore
              }
            }
            if (!asset) {
              results.push({ row: row.row, success: false, error: `Failed to create asset for ${row.symbol}` });
              continue;
            }
          } else {
            const assetData = await res.json();
            asset = assetData as Asset;
          }
        } catch {
          results.push({ row: row.row, success: false, error: `Failed to create asset for ${row.symbol}` });
          continue;
        }
      }

      const effectiveFormat = overrideFormat || dateFormat;
      const isoDate = dateToISO(row.date, effectiveFormat);

      const dedupKey = `${isoDate}-${asset.id}-${row.type}-${row.quantity}-${row.price}-${row.fee}`;
      if (importedKeys.has(dedupKey)) {
        results.push({ row: row.row, success: false, error: 'Duplicate transaction skipped' });
        continue;
      }
      importedKeys.add(dedupKey);

      const txPayload = {
        type: row.type,
        quantity: row.quantity,
        price: row.price,
        fee: row.fee,
        date: isoDate,
      };

      try {
        const res = await fetch(`/api/portfolios/${portfolioId}/assets/${asset.id}/transactions`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(txPayload),
        });

        if (res.ok) {
          results.push({ row: row.row, success: true });
        } else {
          const err = await res.json();
          results.push({ row: row.row, success: false, error: err.message ?? 'Transaction creation failed' });
        }
      } catch {
        results.push({ row: row.row, success: false, error: 'Network error' });
      }
    }

    setImportResults(results);
    setStep('results');
    setIsImporting(false);

    const hasSuccess = results.some((r) => r.success);
    if (hasSuccess) {
      onImportComplete();
    }
  }, [validationReport, assets, portfolioId, onImportComplete]);

  const mappedColumns = Object.entries(columnMapping)
    .filter(([, field]) => field !== null)
    .sort(([a], [b]) => parseInt(a, 10) - parseInt(b, 10));

  const unmappedColumns = Object.entries(columnMapping)
    .filter(([, field]) => field === null);

  const totalRows = csvRows.length;
  const validCount = validationReport?.validRows ?? 0;

  return (
    <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(4px)', display: 'flex', justifyContent: 'center', alignItems: 'center', zIndex: 100 }}>
      <div className="glass-card" style={{ padding: '32px', width: '600px', maxHeight: '80vh', overflowY: 'auto', background: '#121929' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
          <h3 style={{ margin: 0 }}>Import Transactions from CSV</h3>
          <button onClick={onClose} style={{ background: 'none', border: 'none', color: 'var(--text-secondary)', cursor: 'pointer', padding: '4px' }}>
            <X size={20} />
          </button>
        </div>

        {/* Step indicators */}
        <div style={{ display: 'flex', gap: '8px', marginBottom: '24px' }}>
          {(['upload', 'map', 'validate', 'confirm', 'results'] as ImportStep[]).map((s) => (
            <div key={s} style={{ flex: 1, height: '4px', borderRadius: '2px', background: step === s ? 'var(--color-primary)' : step > s ? 'var(--color-success)' : 'var(--border-color)' }} />
          ))}
        </div>

        {/* Step 1: Upload */}
        {step === 'upload' && (
          <div>
            <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
              Upload a CSV file of transactions. The first row will be treated as headers.
            </p>
            <div
              style={{ border: '2px dashed var(--border-color)', borderRadius: '12px', padding: '48px', textAlign: 'center', cursor: 'pointer' }}
              onClick={() => document.getElementById('csv-file-input')?.click()}
            >
              <Upload size={32} style={{ marginBottom: '12px', color: 'var(--text-secondary)' }} />
              <p style={{ color: 'var(--text-secondary)' }}>Click to select or drag and drop a CSV file</p>
              <input id="csv-file-input" type="file" accept=".csv" onChange={handleFileUpload} style={{ display: 'none' }} />
            </div>
            {csvRows.length > 0 && (
              <div style={{ marginTop: '16px' }}>
                <p>{csvRows.length} rows loaded with {headers.length} columns</p>
                <button onClick={() => setStep('map')} className="btn btn-primary" style={{ marginTop: '12px' }}>
                  Continue to Column Mapping <ChevronRight size={16} />
                </button>
              </div>
            )}
          </div>
        )}

        {/* Step 2: Map */}
        {step === 'map' && (
          <div>
            <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
              Review column mappings. Unmapped columns are available in the dropdowns.
            </p>
            {mappedColumns.length > 0 && (
              <div style={{ marginBottom: '16px' }}>
                <h4 style={{ marginBottom: '8px' }}>Mapped Columns</h4>
                <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                  <thead>
                    <tr>
                      <th style={{ textAlign: 'left', padding: '8px', borderBottom: '1px solid var(--border-color)' }}>CSV Column</th>
                      <th style={{ textAlign: 'left', padding: '8px', borderBottom: '1px solid var(--border-color)' }}>Maps To</th>
                    </tr>
                  </thead>
                  <tbody>
                    {mappedColumns.map(([idx, field]) => (
                      <tr key={idx}>
                        <td style={{ padding: '8px', borderBottom: '1px solid var(--border-color)' }}>{headers[parseInt(idx, 10)]}</td>
                        <td style={{ padding: '8px', borderBottom: '1px solid var(--border-color)', color: 'var(--color-primary)', textTransform: 'uppercase' }}>{field}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            {unmappedColumns.length > 0 && (
              <div style={{ marginBottom: '16px' }}>
                <h4 style={{ marginBottom: '8px' }}>Unmapped Columns</h4>
                {unmappedColumns.map(([idx]) => (
                  <div key={idx} style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '8px' }}>
                    <span style={{ color: 'var(--text-secondary)' }}>{headers[parseInt(idx, 10)]}</span>
                    <select
                      value=""
                      onChange={(e) => handleColumnChange(parseInt(idx, 10), e.target.value as CsvTransactionField)}
                      className="form-control"
                      style={{ width: 'auto' }}
                    >
                      <option value="">Skip this column</option>
                      <option value="date">Date</option>
                      <option value="symbol">Symbol</option>
                      <option value="type">Type (BUY/SELL)</option>
                      <option value="quantity">Quantity</option>
                      <option value="price">Price</option>
                      <option value="fee">Fee</option>
                    </select>
                  </div>
                ))}
              </div>
            )}
            {dateAmbiguous && (
              <div style={{ marginBottom: '16px', padding: '12px', background: 'rgba(255,193,7,0.1)', border: '1px solid rgba(255,193,7,0.3)', borderRadius: '8px' }}>
                <p style={{ margin: 0, color: '#ffc107' }}>
                  <AlertTriangle size={16} style={{ verticalAlign: 'middle', marginRight: '6px' }} />
                  Date format <strong>{dateFormat}</strong> is ambiguous. Please specify the format:
                </p>
                <input
                  type="text"
                  value={overrideFormat}
                  onChange={(e) => setOverrideFormat(e.target.value)}
                  placeholder="e.g. MM/DD/YYYY"
                  className="form-control"
                  style={{ marginTop: '8px' }}
                />
              </div>
            )}
            <div style={{ display: 'flex', gap: '12px', marginTop: '16px' }}>
              <button onClick={() => setStep('upload')} className="btn btn-secondary">
                <ChevronLeft size={16} /> Back
              </button>
              <button onClick={handleValidate} className="btn btn-primary">
                Validate Data <ChevronRight size={16} />
              </button>
            </div>
          </div>
        )}

        {/* Step 3: Validate */}
        {step === 'validate' && validationReport && (
          <div>
            <h4 style={{ marginBottom: '12px' }}>Validation Report</h4>
            <div style={{ display: 'flex', gap: '16px', marginBottom: '16px' }}>
              <div style={{ padding: '12px', background: 'rgba(0,180,100,0.1)', border: '1px solid rgba(0,180,100,0.3)', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: 'var(--color-success)' }}>{validationReport.validRows}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>Valid Rows</div>
              </div>
              <div style={{ padding: '12px', background: 'rgba(255,59,48,0.1)', border: '1px solid rgba(255,59,48,0.3)', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: 'var(--color-error)' }}>{validationReport.totalRows - validationReport.validRows}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>Rows with Errors</div>
              </div>
              <div style={{ padding: '12px', background: 'rgba(0,123,255,0.1)', border: '1px solid rgba(0,123,255,0.3)', borderRadius: '8px', textAlign: 'center' }}>
                <div style={{ fontSize: '1.5rem', fontWeight: 'bold', color: 'var(--color-info)' }}>{validationReport.unknownSymbols.length}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>New Assets to Create</div>
              </div>
            </div>

            {validationReport.unknownSymbols.length > 0 && (
              <div style={{ marginBottom: '12px', padding: '8px 12px', background: 'rgba(0,123,255,0.1)', borderRadius: '8px' }}>
                <p style={{ margin: 0, fontSize: '0.875rem' }}>
                  <strong>New symbols will be auto-created as STOCK/USD:</strong>{' '}
                  {validationReport.unknownSymbols.join(', ')}
                </p>
              </div>
            )}

            {validationReport.errorRows.length > 0 && (
              <div style={{ maxHeight: '200px', overflowY: 'auto', marginBottom: '16px' }}>
                <h5 style={{ marginBottom: '8px', color: 'var(--color-error)' }}>Errors by Row</h5>
                {validationReport.errorRows.map((err, i) => (
                  <div key={i} style={{ padding: '4px 0', fontSize: '0.875rem', borderBottom: '1px solid var(--border-color)' }}>
                    <strong>Row {err.row}:</strong> {err.message}
                  </div>
                ))}
              </div>
            )}

            <div style={{ display: 'flex', gap: '12px' }}>
              <button onClick={() => setStep('map')} className="btn btn-secondary">
                <ChevronLeft size={16} /> Back to Mapping
              </button>
              <button onClick={() => setStep('confirm')} className="btn btn-primary">
                Import {validationReport.validRows} Transactions {validationReport.totalRows - validationReport.validRows > 0 && <span style={{ color: 'var(--color-warning)', fontSize: '0.75rem' }}>({validationReport.totalRows - validationReport.validRows} row(s) with errors will be skipped)</span>}
              </button>
            </div>
          </div>
        )}

        {/* Step 4: Confirm */}
        {step === 'confirm' && (
          <div>
            <h4 style={{ marginBottom: '16px' }}>Confirm Import</h4>
            <div style={{ padding: '16px', background: 'rgba(255,255,255,0.03)', borderRadius: '8px', marginBottom: '16px' }}>
              <p><strong>Total rows:</strong> {totalRows}</p>
              <p><strong>New transactions:</strong> {validCount}</p>
              <p><strong>New assets to create:</strong> {validationReport?.unknownSymbols.length ?? 0}</p>
              {validationReport?.unknownSymbols && validationReport.unknownSymbols.length > 0 && (
                <p><strong>Auto-created assets:</strong> {validationReport.unknownSymbols.join(', ')}</p>
              )}
            </div>
            <div style={{ display: 'flex', gap: '12px' }}>
              <button onClick={() => setStep('validate')} className="btn btn-secondary">Back</button>
              <button onClick={handleConfirmImport} className="btn btn-primary" disabled={isImporting}>
                {isImporting ? 'Importing...' : 'Confirm and Import'}
              </button>
            </div>
          </div>
        )}

        {/* Step 5: Results */}
        {step === 'results' && (
          <div>
            <h4 style={{ marginBottom: '16px' }}>Import Results</h4>
            <div style={{ padding: '16px', background: 'rgba(0,180,100,0.1)', border: '1px solid rgba(0,180,100,0.3)', borderRadius: '8px', marginBottom: '16px' }}>
              <p style={{ color: 'var(--color-success)', margin: 0 }}>
                <CheckCircle size={16} style={{ verticalAlign: 'middle', marginRight: '6px' }} />
                {importResults.filter((r) => r.success).length} transactions imported successfully
              </p>
              {importResults.filter((r) => !r.success).length > 0 && (
                <p style={{ color: 'var(--color-error)', margin: '8px 0 0 0' }}>
                  <XCircle size={16} style={{ verticalAlign: 'middle', marginRight: '6px' }} />
                  {importResults.filter((r) => !r.success).length} rows failed
                </p>
              )}
            </div>

            {importResults.filter((r) => !r.success).length > 0 && (
              <div style={{ maxHeight: '200px', overflowY: 'auto', marginBottom: '16px' }}>
                {importResults.filter((r) => !r.success).map((r, i) => (
                  <div key={i} style={{ padding: '4px 0', fontSize: '0.875rem', borderBottom: '1px solid var(--border-color)' }}>
                    <strong>Row {r.row}:</strong> {r.error}
                  </div>
                ))}
              </div>
            )}

            <button onClick={() => { onImportComplete(); onClose(); }} className="btn btn-primary">Done</button>
          </div>
        )}
      </div>
    </div>
  );
}
