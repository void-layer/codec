import { describe, it, expect } from 'vitest'
import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
  encodeInvoiceWire,
  decodeInvoiceWire,
} from './index.js'

const MINIMAL_INVOICE = {
  invoice_id: 'INV-001',
  issued_at: 1_700_000_000,
  due_at: 1_700_086_400,
  network_id: 8453,
  currency: 'USDC',
  decimals: 6,
  from: {
    name: 'Alice',
    wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045',
  },
  client: { name: 'Bob' },
  items: [
    { description: 'Consulting', quantity: 1.0, rate: '1000000' },
  ],
  total: '1000000',
  salt: 'deadbeefdeadbeefdeadbeefdeadbeef',
}

// A larger invoice whose body Brotli can beneficially compress. Minimal
// invoices are too small for Brotli (it expands payloads <~180 B per the
// T-P2-0a spike), so the COMPRESSED_FLAG path needs a sizeable, repetitive
// payload to exercise.
const LONG_DESC =
  'Professional consulting services rendered including architecture review, ' +
  'code review, deployment support and incident response, billed monthly. '
const LARGE_INVOICE = {
  ...MINIMAL_INVOICE,
  invoice_id: 'INV-LARGE-001',
  items: [
    { description: LONG_DESC.repeat(3), quantity: 1.0, rate: '1000000' },
    { description: LONG_DESC.repeat(3), quantity: 2.0, rate: '2000000' },
    { description: LONG_DESC.repeat(3), quantity: 3.0, rate: '3000000' },
  ],
  total: '14000000',
}

describe('encodeInvoiceCanonical + decodeInvoiceCanonical (WASM pass-through)', () => {
  it('returns Uint8Array with magic byte 0x56', () => {
    const bytes = encodeInvoiceCanonical(MINIMAL_INVOICE)
    expect(bytes).toBeInstanceOf(Uint8Array)
    expect(bytes[0]).toBe(0x56)
  })

  it('roundtrips through canonical encode → decode', () => {
    const bytes = encodeInvoiceCanonical(MINIMAL_INVOICE)
    const decoded = decodeInvoiceCanonical(bytes)
    expect(decoded.invoice_id).toBe('INV-001')
    expect(decoded.currency).toBe('USDC')
    expect(decoded.total).toBe('1000000')
  })
})

describe('encodeInvoiceWire', () => {
  it('sets COMPRESSED_FLAG (0x80) on version byte when compression is beneficial', async () => {
    const wire = await encodeInvoiceWire(LARGE_INVOICE)
    expect(wire).toBeInstanceOf(Uint8Array)
    // magic byte preserved
    expect(wire[0]).toBe(0x56)
    // version byte must have 0x80 set (Brotli compressed)
    expect(wire[1]! & 0x80).toBe(0x80)
    // compressed wire must be smaller than the canonical bytes
    expect(wire.length).toBeLessThan(encodeInvoiceCanonical(LARGE_INVOICE).length)
  })

  it('falls back to uncompressed (flag clear) when Brotli would expand the payload', async () => {
    // A minimal invoice is too small for Brotli to help — the shim must emit
    // the uncompressed canonical bytes with COMPRESSED_FLAG clear.
    const wire = await encodeInvoiceWire(MINIMAL_INVOICE)
    expect(wire[1]! & 0x80).toBe(0)
    // and it must still roundtrip through decodeInvoiceWire
    const decoded = await decodeInvoiceWire(wire)
    expect(decoded.invoice_id).toBe('INV-001')
  })
})

describe('decodeInvoiceWire', () => {
  it('roundtrips through wire encode → decode', async () => {
    const wire = await encodeInvoiceWire(MINIMAL_INVOICE)
    const decoded = await decodeInvoiceWire(wire)
    expect(decoded.invoice_id).toBe('INV-001')
    expect(decoded.currency).toBe('USDC')
    expect(decoded.total).toBe('1000000')
    expect(decoded.decimals).toBe(6)
  })

  it('accepts uncompressed canonical bytes (flag clear path)', async () => {
    // decodeInvoiceWire must handle uncompressed input (flag not set)
    const canonical = encodeInvoiceCanonical(MINIMAL_INVOICE)
    // Verify flag is NOT set on canonical output
    expect(canonical[1]! & 0x80).toBe(0)
    // decodeInvoiceWire should pass through to canonical decode
    const decoded = await decodeInvoiceWire(canonical)
    expect(decoded.invoice_id).toBe('INV-001')
  })
})
