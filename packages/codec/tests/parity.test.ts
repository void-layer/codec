/**
 * Golden-vector parity test — TS/JS surface (T-P2-13).
 *
 * Proves both directions × both forms (canonical + wire) conform bit-exact
 * to the frozen vectors in vectors/v4-codec.json.
 *
 * Non-malformed vectors: canonical (sync) + wire (async) encode and decode.
 * Malformed decode-input: assert the thrown error contains a known substring
 *   that identifies the CodecError variant. The WASM layer surfaces errors as
 *   JS Error objects whose message matches the Rust #[error("...")] format string
 *   (e.g. "bad magic bytes" for BadMagic). The brotli-wasm node entry throws a raw
 *   string for decompression failures. Both paths are handled via ERROR_SUBSTRINGS.
 * Malformed encode-input (bigint-amount-over-u256): assert InvalidAmount on encode.
 */

import { describe, it, expect } from 'vitest'
import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
} from '../pkg/void_layer_codec.js'
import { encodeInvoiceWire, decodeInvoiceWire } from '../src/index.js'
import vectors from '../vectors/v4-codec.json'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function toHex(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString('hex')
}

function fromHex(hex: string): Uint8Array {
  return new Uint8Array(Buffer.from(hex, 'hex'))
}

/**
 * Maps CodecError variant names (as stored in expected_error) to a unique
 * substring of the actual thrown message. The WASM layer formats errors from
 * Rust's #[error("...")] strings; brotli-wasm throws a raw string for wire
 * decompression failures.
 *
 * These substrings are stable: they are part of the codec's public error
 * contract and changing them would be a breaking change.
 */
const ERROR_SUBSTRINGS: Record<string, string> = {
  BadMagic: 'bad magic',
  VarintOverflow: 'varint overflow',
  Truncated: 'truncated payload',
  ChecksumMismatch: 'checksum mismatch',
  CompressionFailed: 'Brotli decompress failed',
  InvalidAmount: 'invalid amount',
  UnsupportedVersion: 'unsupported version',
  DictionaryMismatch: 'dictionary mismatch',
  UnknownExtension: 'unknown extension',
  SignatureInvalid: 'signature invalid',
}

function errorSubstring(expectedError: string): string {
  const sub = ERROR_SUBSTRINGS[expectedError]
  if (!sub) throw new Error(`No error substring mapping for: ${expectedError}`)
  return sub
}

type AnyVector = (typeof vectors.vectors)[number]

// Non-malformed vectors have roundtrip:true, canonical_hex, wire_hex, and decoded.
// This type guard narrows the union so those fields are known to be string/non-null.
function isNonMalformed(
  v: AnyVector,
): v is AnyVector & {
  canonical_hex: string
  wire_hex: string
  decoded: NonNullable<AnyVector['decoded']>
} {
  return (
    'roundtrip' in v &&
    (v as { roundtrip?: unknown }).roundtrip === true &&
    typeof (v as { canonical_hex?: unknown }).canonical_hex === 'string' &&
    typeof (v as { wire_hex?: unknown }).wire_hex === 'string' &&
    (v as { decoded?: unknown }).decoded != null
  )
}

const nonMalformed = vectors.vectors.filter(isNonMalformed)

// ---------------------------------------------------------------------------
// Non-malformed vectors — canonical (sync) + wire (async) both directions
// ---------------------------------------------------------------------------

describe('golden-vector parity: canonical (sync)', () => {
  for (const v of nonMalformed) {
    it(`encode:canonical:${v.name}`, () => {
      const encoded = encodeInvoiceCanonical(v.decoded)
      expect(toHex(encoded)).toBe(v.canonical_hex)
    })

    it(`decode:canonical:${v.name}`, () => {
      const decoded = decodeInvoiceCanonical(fromHex(v.canonical_hex))
      expect(decoded).toEqual(v.decoded)
    })
  }
})

describe('golden-vector parity: wire (async)', () => {
  for (const v of nonMalformed) {
    it(`encode:wire:${v.name}`, async () => {
      const encoded = await encodeInvoiceWire(v.decoded)
      expect(toHex(encoded)).toBe(v.wire_hex)
    })

    it(`decode:wire:${v.name}`, async () => {
      const decoded = await decodeInvoiceWire(fromHex(v.wire_hex))
      expect(decoded).toEqual(v.decoded)
    })
  }
})

// ---------------------------------------------------------------------------
// Malformed decode-input vectors — expect error containing known substring
// ---------------------------------------------------------------------------

describe('golden-vector parity: malformed decode-input', () => {
  // Vectors with diagnostic "malformed:canonical" — decode via canonical
  const malformedCanonical = vectors.vectors.filter(
    (v): v is AnyVector & { canonical_hex: string; expected_error: string } =>
      v.diagnostic === 'malformed:canonical' &&
      typeof (v as { canonical_hex?: unknown }).canonical_hex === 'string' &&
      typeof (v as { expected_error?: unknown }).expected_error === 'string',
  )

  for (const v of malformedCanonical) {
    const sub = errorSubstring(v.expected_error)
    it(`malformed:canonical:${v.name} throws containing "${sub}"`, () => {
      expect(() => decodeInvoiceCanonical(fromHex(v.canonical_hex))).toThrow(sub)
    })
  }

  // Vectors with diagnostic "malformed:wire" — decode via wire.
  // brotli-wasm node entry throws a raw string on decompress failure,
  // so we catch manually and assert on String(thrown).
  const malformedWire = vectors.vectors.filter(
    (v): v is AnyVector & { wire_hex: string; expected_error: string } =>
      v.diagnostic === 'malformed:wire' &&
      typeof (v as { wire_hex?: unknown }).wire_hex === 'string' &&
      typeof (v as { expected_error?: unknown }).expected_error === 'string',
  )

  for (const v of malformedWire) {
    const sub = errorSubstring(v.expected_error)
    it(`malformed:wire:${v.name} throws containing "${sub}"`, async () => {
      let thrown: unknown
      try {
        await decodeInvoiceWire(fromHex(v.wire_hex))
      } catch (e) {
        thrown = e
      }
      expect(thrown).toBeDefined()
      // brotli-wasm node entry throws a raw string, not an Error object.
      // String(thrown) works for both Error.message and raw string throws.
      expect(String(thrown)).toContain(sub)
    })
  }
})

// ---------------------------------------------------------------------------
// Malformed encode-input vector — bigint-amount-over-u256 → InvalidAmount
// ---------------------------------------------------------------------------

describe('golden-vector parity: malformed encode-input', () => {
  const encodeInputMalformed = vectors.vectors.filter(
    (
      v,
    ): v is AnyVector & {
      decoded: NonNullable<AnyVector['decoded']>
      expected_error: string
    } =>
      v.diagnostic === 'malformed:encode-input' &&
      (v as { decoded?: unknown }).decoded != null &&
      typeof (v as { expected_error?: unknown }).expected_error === 'string',
  )

  for (const v of encodeInputMalformed) {
    const sub = errorSubstring(v.expected_error)
    it(`malformed:encode-input:${v.name} throws containing "${sub}"`, () => {
      expect(() => encodeInvoiceCanonical(v.decoded)).toThrow(sub)
    })
  }
})
