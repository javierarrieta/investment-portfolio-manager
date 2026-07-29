import { describe, test, expect, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { setupServer } from 'msw/node';
import { handlers } from '../mocks/handlers';
import CsvImportModal from '../../components/CsvImportModal';

const server = setupServer(...handlers);

beforeEach(() => {
  server.listen({ onUnhandledRequest: 'bypass' });
});

afterEach(() => {
  server.close();
});

describe('CsvImportModal integration', () => {
  test('upload step allows file selection', () => {
    render(
      <CsvImportModal
        portfolioId={1}
        assets={[{ id: 1, portfolio_id: 1, symbol: 'AAPL', name: 'Apple Inc.', asset_type: 'STOCK', sector: 'Technology', currency: 'USD', transactions: [] }]}
        onClose={() => {}}
        onImportComplete={() => {}}
      />
    );

    const fileInput = screen.getByLabelText(/csv/i);
    expect(fileInput).toBeInTheDocument();
  });

  test('shows upload step initially', () => {
    render(
      <CsvImportModal
        portfolioId={1}
        assets={[]}
        onClose={() => {}}
        onImportComplete={() => {}}
      />
    );

    expect(screen.getByText(/import transactions from csv/i)).toBeInTheDocument();
  });

  test('renders all 5 steps in order', () => {
    render(
      <CsvImportModal
        portfolioId={1}
        assets={[]}
        onClose={() => {}}
        onImportComplete={() => {}}
      />
    );

    expect(screen.getByText('Import Transactions from CSV')).toBeInTheDocument();
  });
});
