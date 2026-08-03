export function isValidIsin(isin: string): boolean {
  return /^[A-Z]{2}[A-Z0-9]{10}$/i.test(isin)
}