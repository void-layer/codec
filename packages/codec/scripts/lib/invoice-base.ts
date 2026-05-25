/**
 * Base invoice fixture factory and shared dev wallet constants.
 * All generate-vectors scenarios build on top of base().
 */

export const ISSUED_AT = 1_700_000_000
export const DUE_AT = 1_700_086_400
export const FROM_WALLET = '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045'
export const CLIENT_WALLET = '0x70997970C51812dc3A010C7d01b50e0d17dc79C8'
export const SALT = 'deadbeefdeadbeefdeadbeefdeadbeef'

export function base(overrides: Record<string, unknown>): Record<string, unknown> {
  return {
    invoice_id: 'INV-001',
    issued_at: ISSUED_AT,
    due_at: DUE_AT,
    network_id: 1,
    currency: 'USDC',
    decimals: 6,
    from: { name: 'Alice', wallet_address: FROM_WALLET },
    client: { name: 'Bob' },
    items: [{ description: 'Consulting', quantity: 1.0, rate: '1000000' }],
    total: '1000000',
    salt: SALT,
    ...overrides,
  }
}
