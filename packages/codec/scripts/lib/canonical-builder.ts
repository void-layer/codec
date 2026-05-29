/**
 * Low-level canonical payload construction helpers.
 * Used for crafting malformed vectors that need a valid domain separator.
 */

import { receiptHash } from '../../pkg-node/void_layer_codec.js'

/** Encode a non-negative integer as LEB128 (unsigned). */
export function writeLEB128(value: number): Uint8Array {
  const bytes: number[] = []
  let v = value
  do {
    const byte = v & 0x7f
    v >>>= 7
    bytes.push(v !== 0 ? byte | 0x80 : byte)
  } while (v !== 0)
  return new Uint8Array(bytes)
}

/**
 * Mirrors compute_domain_separator from src/encode/fields.rs.
 *
 * domain_separator = keccak256("VOIDPAY_INVOICE_V1" || TLV_stream_excluding_tag_31)
 * where TLV_stream is the wire serialization of each record in ascending tag order.
 * Used to compute a valid domain separator for an arbitrary record set so that
 * malformed-canonical vectors reach the C-1/C-2 guard rather than ChecksumMismatch.
 *
 * @param records Map<tag, value_bytes> of ALL records (tag 31 is excluded automatically).
 */
export function computeDomainSeparatorBytes(records: Map<number, Uint8Array>): Uint8Array {
  const prefix = new TextEncoder().encode('VOIDPAY_INVOICE_V1')
  const parts: Uint8Array[] = [prefix]

  // Ascending tag order — mirrors BTreeMap iteration
  const sortedTags = [...records.keys()].filter((t) => t !== 31).sort((a, b) => a - b)

  for (const tag of sortedTags) {
    const value = records.get(tag)!
    // type byte (1)
    parts.push(new Uint8Array([tag]))
    // length as LEB128 varint
    parts.push(writeLEB128(value.length))
    // value bytes
    parts.push(value)
  }

  const total = parts.reduce((n, p) => n + p.length, 0)
  const body = new Uint8Array(total)
  let offset = 0
  for (const p of parts) {
    body.set(p, offset)
    offset += p.length
  }
  // receiptHash IS keccak256 of arbitrary bytes — it is compute_content_hash under
  // the hood. Reusing it avoids a new devDep (no @noble/hashes needed).
  return receiptHash(body)
}

/**
 * Build a canonical payload from an ordered record map + a pre-computed domain separator.
 * Layout: MAGIC(1) VERSION(1) COUNT(1) TLV_stream
 * Records are written in ascending tag order (BTreeMap order).
 */
export function buildCanonicalPayload(records: Map<number, Uint8Array>): Uint8Array {
  const domSep = computeDomainSeparatorBytes(records)
  const allRecords = new Map(records)
  allRecords.set(31, domSep)

  const sortedTags = [...allRecords.keys()].sort((a, b) => a - b)
  const count = sortedTags.length

  const parts: Uint8Array[] = []
  for (const tag of sortedTags) {
    const value = allRecords.get(tag)!
    parts.push(new Uint8Array([tag]))
    parts.push(writeLEB128(value.length))
    parts.push(value)
  }

  const bodyLen = parts.reduce((n, p) => n + p.length, 0)
  const buf = new Uint8Array(3 + bodyLen)
  buf[0] = 0x56 // MAGIC
  buf[1] = 0x01 // VERSION
  buf[2] = count
  let offset = 3
  for (const p of parts) {
    buf.set(p, offset)
    offset += p.length
  }
  return buf
}
