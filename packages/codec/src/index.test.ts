import { describe, it, expect } from 'vitest'
import type { Invoice } from '@void-layer/types'
import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
  encodeInvoiceWire,
  decodeInvoiceWire,
  receiptHash,
} from './index.js'

interface DecodedInvoice {
  invoice_id: string
  currency: string
  total: string
  decimals: number
}

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
} satisfies Invoice

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
} satisfies Invoice

describe('encodeInvoiceCanonical + decodeInvoiceCanonical (WASM pass-through)', () => {
  it('returns Uint8Array with magic byte 0x56', () => {
    const bytes = encodeInvoiceCanonical(MINIMAL_INVOICE)
    expect(bytes).toBeInstanceOf(Uint8Array)
    expect(bytes[0]).toBe(0x56)
  })

  it('roundtrips through canonical encode → decode', () => {
    const bytes = encodeInvoiceCanonical(MINIMAL_INVOICE)
    const decoded = decodeInvoiceCanonical(bytes) as DecodedInvoice
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
    const decoded = (await decodeInvoiceWire(wire)) as DecodedInvoice
    expect(decoded.invoice_id).toBe('INV-001')
  })
})

describe('decodeInvoiceWire', () => {
  it('roundtrips through wire encode → decode', async () => {
    const wire = await encodeInvoiceWire(MINIMAL_INVOICE)
    const decoded = (await decodeInvoiceWire(wire)) as DecodedInvoice
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
    const decoded = (await decodeInvoiceWire(canonical)) as DecodedInvoice
    expect(decoded.invoice_id).toBe('INV-001')
  })
})

describe('decodeInvoiceWire decompression-bomb guard', () => {
  it('rejects a wire payload that decompresses past MAX_DECOMPRESSED_BYTES', async () => {
    // Build a tiny compressed payload whose Brotli body expands well past the
    // 64 KB cap: 256 KB of zero bytes compresses to a few bytes.
    const brotliMod = await import('brotli-wasm')
    const brotli = await brotliMod.default
    const huge = new Uint8Array(256 * 1024) // 256 KB of 0x00 — far above the cap
    const compressedBody = brotli.compress(huge, { quality: 11 })

    // Wire frame: [MAGIC][VERSION | COMPRESSED_FLAG][compressed body...]
    const wire = new Uint8Array(2 + compressedBody.length)
    wire[0] = 0x56
    wire[1] = 0x01 | 0x80
    wire.set(compressedBody, 2)

    await expect(decodeInvoiceWire(wire)).rejects.toThrow(
      /MAX_DECOMPRESSED_BYTES/,
    )
  })
})

// ---------------------------------------------------------------------------
// G-11 TS parity: write_quantity(0.1234567891) scale clamps at 9, silent rounding.
// The TS shim calls the WASM encodeInvoiceCanonical which calls the Rust write_quantity.
// ---------------------------------------------------------------------------

describe('G-11: write_quantity clamps scale at 9 (TS parity)', () => {
  it('encodes 0.1234567891 without error (scale clamps at 9)', () => {
    const inv: Invoice = {
      ...MINIMAL_INVOICE,
      items: [{ description: 'Fractional qty', quantity: 0.1234567891, rate: '1000000' }],
    }
    // Must not throw — scale clamps silently at 9.
    expect(() => encodeInvoiceCanonical(inv)).not.toThrow()
  })

  it('decoded quantity is close to 0.1234567891 (within 1e-6)', async () => {
    const inv: Invoice = {
      ...MINIMAL_INVOICE,
      items: [{ description: 'Fractional qty', quantity: 0.1234567891, rate: '1000000' }],
    }
    const canonical = encodeInvoiceCanonical(inv)
    const decoded = decodeInvoiceCanonical(canonical) as { items: { quantity: number }[] }
    const qty = decoded.items[0]!.quantity
    expect(Math.abs(qty - 0.1234567891)).toBeLessThan(1e-6)
  })
})

// ---------------------------------------------------------------------------
// G-35: decodeInvoiceWire(encodeInvoiceCanonical(inv)) — canonical (uncompressed)
// payload fed to wire decoder → correct Invoice.
// The wire decoder must pass through uncompressed canonical bytes unchanged.
// ---------------------------------------------------------------------------

describe('G-35: decodeInvoiceWire accepts encodeInvoiceCanonical output', () => {
  it('decodes canonical (uncompressed) bytes as wire input — invoice_id matches', async () => {
    const canonical = encodeInvoiceCanonical(MINIMAL_INVOICE)
    // Canonical bytes have COMPRESSED_FLAG clear on version byte.
    expect(canonical[1]! & 0x80).toBe(0)
    const decoded = (await decodeInvoiceWire(canonical)) as DecodedInvoice
    expect(decoded.invoice_id).toBe('INV-001')
    expect(decoded.currency).toBe('USDC')
    expect(decoded.total).toBe('1000000')
    expect(decoded.decimals).toBe(6)
  })

  it('decodes canonical bytes for a larger invoice correctly', async () => {
    const canonical = encodeInvoiceCanonical(LARGE_INVOICE)
    const decoded = (await decodeInvoiceWire(canonical)) as DecodedInvoice
    expect(decoded.invoice_id).toBe('INV-LARGE-001')
    expect(decoded.total).toBe('14000000')
  })
})

describe('receiptHash (JS export coverage)', () => {
  // Hand-crafted canonical TLV: tag=0x01, length=0x03, value=[0xAA, 0xBB, 0xCC]
  const CANONICAL_FIXTURE = new Uint8Array([0x01, 0x03, 0xaa, 0xbb, 0xcc])

  it('returns a 32-byte Uint8Array and is deterministic', () => {
    const first = receiptHash(CANONICAL_FIXTURE)
    const second = receiptHash(CANONICAL_FIXTURE)
    expect(first).toBeInstanceOf(Uint8Array)
    expect(first).toHaveLength(32)
    expect(first).toEqual(second)
  })

  it('golden value — minimal-single-tlv canonical bytes', () => {
    // Keccak-256 of the canonical bytes for the minimal-single-tlv vector.
    // Value is independently verified against the receipt_hash_hex field in
    // vectors/v4-codec.json.
    const canonical = new Uint8Array(
      Buffer.from(
        '56010d0202000104046553f100060380a3050801060a14d8da6bf26964af9d7eed9e03e53415d37aa960450c0200010e10010a436f6e73756c74696e67000101061005416c6963651203426f621410deadbeefdeadbeefdeadbeefdeadbeef1607494e562d303031180201061f20e7620cf63c7f087f05bd266fba981b1e79c3697a22fcaf710f6c2b69db868be5',
        'hex',
      ),
    )
    const hash = receiptHash(canonical)
    expect(Buffer.from(hash).toString('hex')).toBe(
      'b5e4a21f39c8bdc09fd93a54806584fab25e3094c045835a7bd1928246223d53',
    )
  })
})
