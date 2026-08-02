import { describe, it, expect } from 'vitest'
import { isValidIsin } from '../../utils/isinValidator'

describe('isValidIsin', () => {
  it('returns true for valid ISINs', () => {
    expect(isValidIsin('US0378331005')).toBe(true)
    expect(isValidIsin('DE000BAY0017')).toBe(true)
    expect(isValidIsin('us0378331005')).toBe(true)
  })

  it('returns false for ISINs that are too short', () => {
    expect(isValidIsin('US037833100')).toBe(false)
  })

  it('returns false for ISINs that are too long', () => {
    expect(isValidIsin('US03783310050')).toBe(false)
  })

  it('returns false for ISINs with non-alphanumeric characters', () => {
    expect(isValidIsin('US037833100!')).toBe(false)
  })

  it('returns false for ISINs without a country code', () => {
    expect(isValidIsin('0378331005')).toBe(false)
  })

  it('returns false for empty strings', () => {
    expect(isValidIsin('')).toBe(false)
  })
})