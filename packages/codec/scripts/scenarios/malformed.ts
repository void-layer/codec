/**
 * Malformed vector builders.
 *
 * Hand-crafted byte sequences that must produce a specific CodecError on
 * decode (or InvalidAmount on encode). Split into two groups matching their
 * position in the corpus: early (after bigint non-malformed) and late (after
 * unicode non-malformed).
 */

import { encodeInvoiceCanonical } from '../../pkg-node/void_layer_codec.js'
import { base } from '../lib/invoice-base.js'
import { toHex } from '../lib/utils.js'
import {
  writeLEB128,
  buildCanonicalPayload,
  computeDomainSeparatorBytes,
} from '../lib/canonical-builder.js'

export interface MalformedVector {
  name: string
  canonical_hex?: string
  wire_hex?: string
  decoded?: unknown
  diagnostic: string
  expected_error: string
}

/**
 * Early malformed vectors (corpus positions 10–12): bigint-amount-over-u256,
 * malformed-checksum-mismatch, malformed-varint-overflow.
 * Emitted after the bigint non-malformed group, before extensions.
 */
export function buildEarlyMalformedVectors(): MalformedVector[] {
  const vectors: MalformedVector[] = []

  // 3d. 2^256 — one above U256::MAX, encode must produce InvalidAmount.
  {
    const OVER_U256 = '115792089237316195423570985008687907853269984665640564039457584007913129639936'
    const overU256Invoice = base({
      invoice_id: 'INV-BIGINT-OVER-U256',
      currency: 'ETH',
      decimals: 18,
      items: [{ description: 'Over U256 payment', quantity: 1.0, rate: OVER_U256 }],
      total: OVER_U256,
    })
    try {
      encodeInvoiceCanonical(overU256Invoice)
      throw new Error('Expected InvalidAmount error but encode succeeded — codec regression')
    } catch (err: unknown) {
      if (err instanceof Error && err.message.startsWith('Expected InvalidAmount')) throw err
      // encode threw as expected — no bytes produced
    }
    vectors.push({
      name: 'bigint-amount-over-u256',
      decoded: overU256Invoice,
      diagnostic: 'malformed:encode-input',
      expected_error: 'InvalidAmount',
    })
  }

  // 3e. malformed-checksum-mismatch
  {
    const checksumBytes = new Uint8Array(
      Buffer.from(
        '56010118268080808080808080808080808080808080808080808080808080808080808080808080808080',
        'hex',
      ),
    )
    vectors.push({
      name: 'malformed-checksum-mismatch',
      canonical_hex: toHex(checksumBytes),
      diagnostic: 'malformed:canonical',
      expected_error: 'ChecksumMismatch',
    })
  }

  // 3f. malformed-varint-overflow
  {
    const buf = new Uint8Array(4 + 37)
    buf[0] = 0x56 // MAGIC
    buf[1] = 0x01 // VERSION
    buf[2] = 0x01 // COUNT=1
    buf[3] = 0x18 // TLV type=24 (TLV_TOTAL) — type byte is valid; overflow is in LENGTH
    buf.fill(0x80, 4) // 37 bytes all with continuation bit set, no terminal → VarintOverflow
    vectors.push({
      name: 'malformed-varint-overflow',
      canonical_hex: toHex(buf),
      diagnostic: 'malformed:canonical',
      expected_error: 'VarintOverflow',
    })
  }

  return vectors
}

/**
 * Late malformed vectors (corpus positions 21–27): corrupted-brotli, oversize,
 * bad-magic, unknown-tlv-tag, duplicate-tlv-tag, non-canonical-varint,
 * unknown-content-tag.
 * Emitted after the unicode non-malformed group, before demo-invoices.
 */
export function buildLateMalformedVectors(): MalformedVector[] {
  const vectors: MalformedVector[] = []

  // 6a. malformed-corrupted-brotli
  {
    const bytes = new Uint8Array([0x56, 0x81, 0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe])
    vectors.push({
      name: 'malformed-corrupted-brotli',
      wire_hex: toHex(bytes),
      diagnostic: 'malformed:wire',
      expected_error: 'CompressionFailed',
    })
  }

  // 6b. malformed-oversize: claims 1494-byte TLV value but buffer has only 4 bytes → Truncated
  {
    const bytes = new Uint8Array(10)
    bytes[0] = 0x56; bytes[1] = 0x01; bytes[2] = 0x01
    bytes[3] = 0x18              // TLV_TOTAL=24
    bytes[4] = 0xd6; bytes[5] = 0x0b  // LEB128(1494)
    // bytes[6..9] = 0x00 — far fewer than claimed 1494
    vectors.push({
      name: 'malformed-oversize',
      canonical_hex: toHex(bytes),
      diagnostic: 'malformed:canonical',
      expected_error: 'Truncated',
    })
  }

  // 6c. malformed-bad-magic: first byte is not 0x56
  {
    const bytes = new Uint8Array([0xff, 0x01, 0x01, 0x18, 0x02, 0x01, 0x00])
    vectors.push({
      name: 'malformed-bad-magic',
      canonical_hex: toHex(bytes),
      diagnostic: 'malformed:canonical',
      expected_error: 'BadMagic',
    })
  }

  // 7a. malformed-unknown-tlv-tag
  {
    const minHex =
      '56010d0202000104046553f100060380a3050801060a14d8da6bf26964af9d7eed9e03e53415d37aa960450c0200010e10010a436f6e73756c74696e67000101061005416c6963651203426f621410deadbeefdeadbeefdeadbeefdeadbeef1607494e562d303031180201061f20e7620cf63c7f087f05bd266fba981b1e79c3697a22fcaf710f6c2b69db868be5'
    const minBytes = new Uint8Array(Buffer.from(minHex, 'hex'))

    const contentRecords = new Map<number, Uint8Array>()
    let off = 3
    while (off < minBytes.length) {
      const tag = minBytes[off]!
      off++
      let len = 0
      let shift = 0
      while (true) {
        const b = minBytes[off]!
        off++
        len |= (b & 0x7f) << shift
        shift += 7
        if (!(b & 0x80)) break
      }
      const val = minBytes.slice(off, off + len)
      off += len
      if (tag !== 31) contentRecords.set(tag, val)
    }

    contentRecords.set(99, new Uint8Array([0xde, 0xad]))

    const payload = buildCanonicalPayload(contentRecords)
    vectors.push({
      name: 'malformed-unknown-tlv-tag',
      canonical_hex: toHex(payload),
      diagnostic: 'malformed:canonical',
      expected_error: 'UnknownExtension',
    })
  }

  // 7b. malformed-duplicate-tlv-tag
  {
    const minHex =
      '56010d0202000104046553f100060380a3050801060a14d8da6bf26964af9d7eed9e03e53415d37aa960450c0200010e10010a436f6e73756c74696e67000101061005416c6963651203426f621410deadbeefdeadbeefdeadbeefdeadbeef1607494e562d303031180201061f20e7620cf63c7f087f05bd266fba981b1e79c3697a22fcaf710f6c2b69db868be5'
    const minBytes = new Uint8Array(Buffer.from(minHex, 'hex'))

    const contentRecords = new Map<number, Uint8Array>()
    let off = 3
    while (off < minBytes.length) {
      const tag = minBytes[off]!
      off++
      let len = 0
      let shift = 0
      while (true) {
        const b = minBytes[off]!
        off++
        len |= (b & 0x7f) << shift
        shift += 7
        if (!(b & 0x80)) break
      }
      const val = minBytes.slice(off, off + len)
      off += len
      if (tag !== 31) contentRecords.set(tag, val)
    }

    const firstTotal = contentRecords.get(24)!
    const domSepBytes = computeDomainSeparatorBytes(contentRecords)

    function tlvRecord(tag: number, value: Uint8Array): Uint8Array {
      const lenBytes = writeLEB128(value.length)
      const rec = new Uint8Array(1 + lenBytes.length + value.length)
      rec[0] = tag
      rec.set(lenBytes, 1)
      rec.set(value, 1 + lenBytes.length)
      return rec
    }

    const altTotalValue = new Uint8Array([0x02, 0x02])

    const sortedTags = [...contentRecords.keys()].sort((a, b) => a - b)
    const streamParts: Uint8Array[] = []
    for (const tag of sortedTags) {
      if (tag === 24) {
        streamParts.push(tlvRecord(24, altTotalValue))
        streamParts.push(tlvRecord(24, firstTotal))
      } else {
        streamParts.push(tlvRecord(tag, contentRecords.get(tag)!))
      }
    }
    streamParts.push(tlvRecord(31, domSepBytes))

    const streamLen = streamParts.reduce((n, p) => n + p.length, 0)
    const count = sortedTags.length + 1 + 1
    const payload = new Uint8Array(3 + streamLen)
    payload[0] = 0x56
    payload[1] = 0x01
    payload[2] = count
    let woff = 3
    for (const p of streamParts) {
      payload.set(p, woff)
      woff += p.length
    }

    vectors.push({
      name: 'malformed-duplicate-tlv-tag',
      canonical_hex: toHex(payload),
      diagnostic: 'malformed:canonical',
      expected_error: 'InvalidData',
    })
  }

  // 8a. malformed-non-canonical-varint
  {
    const bytes = new Uint8Array([0x56, 0x01, 0x80, 0x00])
    vectors.push({
      name: 'malformed-non-canonical-varint',
      canonical_hex: toHex(bytes),
      diagnostic:
        'malformed:canonical — LEB128 varint [0x80, 0x00] encodes value 0 with a spurious continuation byte; canonical form requires the shortest encoding (single 0x00 byte). Decoder must reject.',
      expected_error: 'Truncated',
    })
  }

  // 8b. malformed-unknown-content-tag
  {
    const bytes = new Uint8Array(
      Buffer.from(
        '56010e0202000104046553f100060380a3050801060a14d8da6bf26964af9d7eed9e03e53415d37aa960450c0200010e10010a436f6e73756c74696e67000101061005416c6963651203426f621410deadbeefdeadbeefdeadbeefdeadbeef1607494e562d303031180201061f20e7620cf63c7f087f05bd266fba981b1e79c3697a22fcaf710f6c2b69db868be52702dead',
        'hex',
      ),
    )
    vectors.push({
      name: 'malformed-unknown-content-tag',
      canonical_hex: toHex(bytes),
      diagnostic:
        'malformed:canonical — TLV tag 39 (0x27) is outside the v1 KNOWN_TAGS set. Decoder must reject with UnknownExtension before checksum validation.',
      expected_error: 'UnknownExtension',
    })
  }

  return vectors
}
