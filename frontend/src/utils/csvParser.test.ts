import { describe, test, expect } from 'vitest';
import { detectColumns, detectDateFormat } from './csvParser';

describe('detectColumns', () => {
  test('maps exact column names', () => {
    const result = detectColumns(['date', 'symbol', 'type', 'quantity', 'price', 'fee']);
    expect(result[0]).toBe('date');
    expect(result[1]).toBe('symbol');
    expect(result[2]).toBe('type');
    expect(result[3]).toBe('quantity');
    expect(result[4]).toBe('price');
    expect(result[5]).toBe('fee');
  });

  test('maps case-insensitive column names', () => {
    const result = detectColumns(['Date', 'Symbol', 'Type', 'Qty', 'Price', 'Fee']);
    expect(result[0]).toBe('date');
    expect(result[1]).toBe('symbol');
    expect(result[2]).toBe('type');
    expect(result[3]).toBe('quantity');
    expect(result[4]).toBe('price');
    expect(result[5]).toBe('fee');
  });

  test('maps underscores and camelCase', () => {
    const result = detectColumns(['txn_date', 'unit_price', 'ticker']);
    expect(result[0]).toBe('date');
    expect(result[1]).toBe('price');
    expect(result[2]).toBe('symbol');
  });

  test('maps unknown columns to null', () => {
    const result = detectColumns(['date', 'description', 'symbol']);
    expect(result[0]).toBe('date');
    expect(result[1]).toBeNull();
    expect(result[2]).toBe('symbol');
  });

  test('returns empty mapping for empty headers', () => {
    const result = detectColumns([]);
    expect(Object.keys(result)).toHaveLength(0);
  });
});

describe('detectDateFormat', () => {
  test('detects YYYY-MM-DD format', () => {
    const result = detectDateFormat([['2024-01-15', 'AAPL', 'BUY', '100', '150.00', '5.00']]);
    expect(result.format).toBe('YYYY-MM-DD');
    expect(result.ambiguous).toBe(false);
  });

  test('detects MM/DD/YYYY format', () => {
    const result = detectDateFormat([['01/15/2024', 'AAPL', 'BUY', '100', '150.00']]);
    expect(result.format).toBe('MM/DD/YYYY');
    expect(result.ambiguous).toBe(true);
  });

  test('detects DD/MM/YYYY format', () => {
    const result = detectDateFormat([['15/01/2024', 'AAPL', 'BUY', '100', '150.00']]);
    expect(result.format).toBe('DD/MM/YYYY');
    expect(result.ambiguous).toBe(true);
  });

  test('detects YYYY-MM-DD HH:MM format', () => {
    const result = detectDateFormat([['2024-01-15 10:30', 'AAPL', 'BUY', '100', '150.00']]);
    expect(result.format).toBe('YYYY-MM-DD HH:MM');
    expect(result.ambiguous).toBe(false);
  });

  test('defaults to YYYY-MM-DD for non-matching cells', () => {
    const result = detectDateFormat([['hello', 'world', 'test']]);
    expect(result.format).toBe('YYYY-MM-DD');
    expect(result.ambiguous).toBe(false);
  });

  test('returns first matching format when multiple date cells exist', () => {
    const result = detectDateFormat([['2024-01-15', '01/15/2024', 'AAPL']]);
    expect(result.format).toBe('YYYY-MM-DD');
  });
});
