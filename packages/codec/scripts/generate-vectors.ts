/**
 * Golden vector generator — @void-layer/codec v4-codec.json
 *
 * Produces the starter set of 18 canonical golden vectors per spec §D-R6.1 and
 * plan-phase2c §T-P2-12 (C2 amendment: TypeScript generator, not Rust bin).
 *
 * Run (from packages/codec root):
 *   pnpm -C packages/codec exec vite-node scripts/generate-vectors.ts
 *
 * Imports canonical encode/decode from the nodejs-target pkg-node/ (synchronous
 * CJS-style — no Vite plugin required). Wire encode/decode mirrors the JS shim
 * in src/index.ts using the same brotli-wasm peerDep.
 *
 * C4 amendment: wire_hex == canonical_hex when Brotli would expand the payload
 * (small invoices). Each non-malformed vector carries both hex fields regardless.
 */

import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
  receiptHash,
} from '../pkg-node/void_layer_codec.js'
// brotli-wasm: resolve the Node-compatible entry via bare specifier.
// vitest.config.ts aliases 'brotli-wasm' → the CJS-friendly Node build.
import brotliWasmInit from 'brotli-wasm'

const _filename = fileURLToPath(import.meta.url)
const _dirname = path.dirname(_filename)
const VECTORS_DIR = path.resolve(_dirname, '../vectors')
const OUT_PATH = path.join(VECTORS_DIR, 'v4-codec.json')

const COMPRESSED_FLAG = 0x80

// ---------------------------------------------------------------------------
// Wire encode/decode — mirrors src/index.ts logic exactly
// ---------------------------------------------------------------------------

async function wireEncode(invoice: unknown): Promise<Uint8Array> {
  const brotli = await brotliWasmInit
  const canonical: Uint8Array = encodeInvoiceCanonical(invoice)
  if (canonical.length < 3) return canonical
  const body = canonical.slice(2)
  const compressed = brotli.compress(body, { quality: 11 })
  if (compressed.length >= body.length) return canonical
  const result = new Uint8Array(2 + compressed.length)
  result[0] = canonical[0]!
  result[1] = canonical[1]! | COMPRESSED_FLAG
  result.set(compressed, 2)
  return result
}

async function wireDecode(bytes: Uint8Array): Promise<unknown> {
  if (bytes.length < 3 || !(bytes[1]! & COMPRESSED_FLAG)) {
    return decodeInvoiceCanonical(bytes)
  }
  const brotli = await brotliWasmInit
  const decompressed = brotli.decompress(bytes.slice(2))
  const canonical = new Uint8Array(2 + decompressed.length)
  canonical[0] = bytes[0]!
  canonical[1] = bytes[1]! & 0x7f
  canonical.set(decompressed, 2)
  return decodeInvoiceCanonical(canonical)
}

// ---------------------------------------------------------------------------
// Invoice fixtures
// ---------------------------------------------------------------------------

const ISSUED_AT = 1_700_000_000
const DUE_AT = 1_700_086_400
const FROM_WALLET = '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045'
const CLIENT_WALLET = '0x70997970C51812dc3A010C7d01b50e0d17dc79C8'
const SALT = 'deadbeefdeadbeefdeadbeefdeadbeef'

function base(overrides: Record<string, unknown>): Record<string, unknown> {
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

function toHex(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString('hex')
}

function isCompressed(hex: string): boolean {
  if (hex.length < 4) return false
  return (parseInt(hex.slice(2, 4), 16) & COMPRESSED_FLAG) !== 0
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
function computeDomainSeparatorBytes(records: Map<number, Uint8Array>): Uint8Array {
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

/** Encode a non-negative integer as LEB128 (unsigned). */
function writeLEB128(value: number): Uint8Array {
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
 * Build a canonical payload from an ordered record map + a pre-computed domain separator.
 * Layout: MAGIC(1) VERSION(1) COUNT(1) TLV_stream
 * Records are written in ascending tag order (BTreeMap order).
 */
function buildCanonicalPayload(records: Map<number, Uint8Array>): Uint8Array {
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

interface NonMalformedVector {
  name: string
  canonical_hex: string
  wire_hex: string
  receipt_hash_hex: string
  decoded: unknown
  roundtrip: boolean
  diagnostic: string
}

interface MalformedVector {
  name: string
  canonical_hex?: string
  wire_hex?: string
  decoded?: unknown
  diagnostic: string
  expected_error: string
}

type Vector = NonMalformedVector | MalformedVector

const WIRE_DIAG =
  'wire_hex = Brotli-compressed wire, or == canonical_hex when Brotli expands (small payloads)'

async function nonMalformed(
  name: string,
  invoice: Record<string, unknown>,
  diagnostic?: string,
): Promise<NonMalformedVector> {
  const canonical = encodeInvoiceCanonical(invoice)
  const wire = await wireEncode(invoice)
  const canonical_hex = toHex(canonical)
  const wire_hex = toHex(wire)
  const receipt_hash_hex = toHex(receiptHash(canonical))
  const decodedC = decodeInvoiceCanonical(canonical)
  const decodedW = await wireDecode(wire)
  const roundtrip = JSON.stringify(decodedC) === JSON.stringify(decodedW)
  return {
    name,
    canonical_hex,
    wire_hex,
    receipt_hash_hex,
    decoded: decodedC,
    roundtrip,
    diagnostic: diagnostic ?? WIRE_DIAG,
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const vectors: Vector[] = []

  // 1. Minimal
  vectors.push(
    await nonMalformed(
      'minimal-single-tlv',
      base({}),
      `Smallest valid invoice — all required fields, one item, no optional fields. ${WIRE_DIAG}`,
    ),
  )

  // 2. Chain selectors (5)
  const chains: Array<[number, string]> = [
    [1, 'ethereum'],
    [8453, 'base'],
    [42161, 'arbitrum'],
    [10, 'optimism'],
    [137, 'polygon'],
  ]
  for (const [network_id, chainName] of chains) {
    vectors.push(
      await nonMalformed(
        `chain-${chainName}`,
        base({ network_id, invoice_id: `INV-CHAIN-${network_id}` }),
        `Chain selector: ${chainName} (network_id=${network_id}). ${WIRE_DIAG}`,
      ),
    )
  }

  // 3. BigInt edges (4)

  // 3a. amount = 0
  vectors.push(
    await nonMalformed(
      'bigint-amount-zero',
      base({
        invoice_id: 'INV-BIGINT-ZERO',
        items: [{ description: 'Zero payment', quantity: 1.0, rate: '0' }],
        total: '0',
      }),
      `BigInt edge: total = 0 (LEB128 single 0x00 byte). ${WIRE_DIAG}`,
    ),
  )

  // 3b. amount = 1
  vectors.push(
    await nonMalformed(
      'bigint-amount-one',
      base({
        invoice_id: 'INV-BIGINT-ONE',
        items: [{ description: 'One atomic unit', quantity: 1.0, rate: '1' }],
        total: '1',
      }),
      `BigInt edge: total = 1 (smallest nonzero, no trailing zeros). ${WIRE_DIAG}`,
    ),
  )

  // 3c. U256::MAX — largest value the U256 codec accepts without overflow.
  // Codec widened to U256 in T-P2-12a: this must now encode successfully (roundtrip true).
  const U256_MAX = '115792089237316195423570985008687907853269984665640564039457584007913129639935'
  vectors.push(
    await nonMalformed(
      'bigint-amount-uint256-max',
      base({
        invoice_id: 'INV-BIGINT-U256MAX',
        currency: 'ETH',
        decimals: 18,
        items: [{ description: 'Max uint256 payment', quantity: 1.0, rate: U256_MAX }],
        total: U256_MAX,
      }),
      `BigInt edge: total = U256::MAX (${U256_MAX}) — largest encodable value after U256 widening. ${WIRE_DIAG}`,
    ),
  )

  // 3d. 2^256 — one above U256::MAX, must produce InvalidAmount error.
  // diagnostic: "malformed:encode-input" — error fires at encode time, no bytes produced.
  // decoded field is present so T-P2-13 can construct the Invoice and assert InvalidAmount.
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
    } as MalformedVector)
  }

  // 3e. malformed-checksum-mismatch — bytes with valid header + COUNT=1 but payload
  // has no valid domain-separator/checksum TLV → ChecksumMismatch.
  // This is the corrected classification of the original "malformed-varint-overflow"
  // vector (hex is unchanged; only name + expected_error corrected per Kai decision
  // 2026-05-20: the codec hits ChecksumMismatch before any varint overflow path).
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

  // 3f. malformed-varint-overflow — crafted bytes where the LENGTH field of the
  // first TLV record is a varint with 37 continuation bytes and no terminator.
  // Wire: MAGIC VERSION COUNT=1 TYPE=0x18 [37× 0x80 with MSB set, no terminal byte]
  // read_varint fires VarintOverflow at bytes_read == MAX_BYTES (37) before reaching
  // the checksum validation stage.
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

  // 4. Extensions (3)

  // 4a. magic-dust: micro-amount uniquifier in total
  vectors.push(
    await nonMalformed(
      'extension-magic-dust',
      base({
        invoice_id: 'INV-EXT-DUST',
        total: '1000042',
        notes: 'Magic dust applied: +0.000042 for unique matching',
        items: [{ description: 'Consulting', quantity: 1.0, rate: '1000042' }],
      }),
      `Extension: magic-dust (micro-amount uniquifier in total + notes field). ${WIRE_DIAG}`,
    ),
  )

  // 4b. OG-param: from.email + client.wallet_address + notes
  vectors.push(
    await nonMalformed(
      'extension-og-param',
      base({
        invoice_id: 'INV-EXT-OG',
        from: { name: 'Alice Dev Studio', wallet_address: FROM_WALLET, email: 'alice@dev.io' },
        client: { name: 'Acme Corp', wallet_address: CLIENT_WALLET },
        notes: 'Please pay within 30 days',
        total: '5000000',
        items: [{ description: 'Design work', quantity: 1.0, rate: '5000000' }],
      }),
      `Extension: OG-param fields (from.email, client.wallet_address, notes) for social preview. ${WIRE_DIAG}`,
    ),
  )

  // 4c. sub-invoice-chain: ETH on Arbitrum with tax + discount
  vectors.push(
    await nonMalformed(
      'extension-sub-invoice-chain',
      base({
        invoice_id: 'INV-EXT-SUBCHAIN',
        network_id: 42161,
        currency: 'ETH',
        decimals: 18,
        total: '500000000000000000',
        items: [{ description: 'Cross-chain consulting', quantity: 1.0, rate: '500000000000000000' }],
        tax: '10',
        discount: '5',
      }),
      `Extension: sub-invoice chain — ETH on Arbitrum with tax and discount fields. ${WIRE_DIAG}`,
    ),
  )

  // 5. Unicode (multi-byte UTF-8) vectors
  // All 18 original vectors are 100% ASCII; these cover multi-byte code points.

  // 5a. Cyrillic text — 2-byte UTF-8 sequences in name, client.name, description, notes
  vectors.push(
    await nonMalformed(
      'unicode-cyrillic',
      base({
        invoice_id: 'INV-UNI-CYR',
        from: { name: 'Алиса Разработчик', wallet_address: FROM_WALLET },
        client: { name: 'Боб Клиент' },
        items: [{ description: 'Консультационные услуги', quantity: 1.0, rate: '2000000' }],
        total: '2000000',
        notes: 'Оплата в течение 30 дней',
      }),
      `Unicode: Cyrillic (2-byte UTF-8) in from.name, client.name, item.description, notes. ${WIRE_DIAG}`,
    ),
  )

  // 5b. CJK — 3-byte UTF-8 sequences in description and notes
  vectors.push(
    await nonMalformed(
      'unicode-cjk',
      base({
        invoice_id: 'INV-UNI-CJK',
        from: { name: 'Alice', wallet_address: FROM_WALLET },
        client: { name: 'Bob' },
        items: [{ description: '软件开发咨询服务', quantity: 1.0, rate: '3000000' }],
        total: '3000000',
        notes: '請在30天內付款。感謝您的支持。',
      }),
      `Unicode: CJK (3-byte UTF-8) in item.description and notes. ${WIRE_DIAG}`,
    ),
  )

  // 5c. Emoji — 4-byte surrogate pairs in notes and from.name
  vectors.push(
    await nonMalformed(
      'unicode-emoji',
      base({
        invoice_id: 'INV-UNI-EMJ',
        from: { name: 'Alice 🚀', wallet_address: FROM_WALLET },
        client: { name: 'Bob' },
        items: [{ description: 'Premium consulting', quantity: 1.0, rate: '5000000' }],
        total: '5000000',
        notes: '✅ Payment confirmed 🎉 Thank you! 💎',
      }),
      `Unicode: emoji (4-byte UTF-8 surrogate pairs) in from.name and notes. Codec treats as bytes — no normalization. ${WIRE_DIAG}`,
    ),
  )

  // 5d. RTL — Arabic text in from.name and item.description
  // Codec treats strings as opaque bytes — must NOT normalize or reorder RTL text.
  // Verify decode produces byte-identical output.
  vectors.push(
    await nonMalformed(
      'unicode-rtl',
      base({
        invoice_id: 'INV-UNI-RTL',
        from: { name: 'أليس المطور', wallet_address: FROM_WALLET },
        client: { name: 'Bob' },
        items: [{ description: 'خدمات استشارية', quantity: 1.0, rate: '1500000' }],
        total: '1500000',
        notes: 'يرجى الدفع خلال 30 يوماً',
      }),
      `Unicode: Arabic RTL (2-4 byte UTF-8) in from.name, description, notes. Codec treats as opaque bytes — no reorder or normalize. ${WIRE_DIAG}`,
    ),
  )

  // 5e. Mixed — all scripts combined in different fields
  vectors.push(
    await nonMalformed(
      'unicode-mixed',
      base({
        invoice_id: 'INV-UNI-MIX',
        from: { name: 'Alice 🌍', wallet_address: FROM_WALLET },
        client: { name: 'Боб / 鲍勃' },
        items: [
          { description: '咨询服务 / Consulting / Консультации', quantity: 1.0, rate: '4000000' },
        ],
        total: '4000000',
        notes: 'Mixed: Кириллица + 中文 + العربية + emoji 🎯',
      }),
      `Unicode: mixed scripts (ASCII + Cyrillic + CJK + Arabic + emoji) across all text fields. ${WIRE_DIAG}`,
    ),
  )

  // 6. Malformed (3)
  {
    const bytes = new Uint8Array([0x56, 0x81, 0xde, 0xad, 0xbe, 0xef, 0xca, 0xfe, 0xba, 0xbe])
    vectors.push({
      name: 'malformed-corrupted-brotli',
      wire_hex: toHex(bytes),
      diagnostic: 'malformed:wire',
      expected_error: 'CompressionFailed',
    })
  }

  // 5b. Oversize: claims a 1494-byte TLV value but the buffer has only 4 bytes → Truncated
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

  // 5c. Bad magic: first byte is not 0x56
  {
    const bytes = new Uint8Array([0xff, 0x01, 0x01, 0x18, 0x02, 0x01, 0x00])
    vectors.push({
      name: 'malformed-bad-magic',
      canonical_hex: toHex(bytes),
      diagnostic: 'malformed:canonical',
      expected_error: 'BadMagic',
    })
  }

  // 7. Tranche B malformed vectors — C-1/C-2 regression anchors.
  //    Both carry a VALID domain separator computed over the malformed record set
  //    so the decoder reaches the duplicate/unknown-tag guard rather than short-
  //    circuiting at ChecksumMismatch.

  // 7a. malformed-unknown-tlv-tag — unknown tag 99 in the TLV stream.
  //     Domain separator computed over all records including tag 99 → decoder
  //     passes checksum but hits the C-2 unknown-tag guard → UnknownExtension.
  {
    // Extract the 12 content records from the minimal-single-tlv canonical hex
    // (tags 2,4,6,8,10,12,14,16,18,20,22,24).  Re-parse from the frozen hex so
    // this vector is independent of the live encoder.
    const minHex =
      '56010d0202000104046553f100060380a3050801060a14d8da6bf26964af9d7eed9e03e53415d37aa960450c0200010e10010a436f6e73756c74696e67000101061005416c6963651203426f621410deadbeefdeadbeefdeadbeefdeadbeef1607494e562d303031180201061f20e7620cf63c7f087f05bd266fba981b1e79c3697a22fcaf710f6c2b69db868be5'
    const minBytes = new Uint8Array(Buffer.from(minHex, 'hex'))

    // Parse TLV stream (skip 3-byte header, skip tag-31 domain separator)
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

    // Inject unknown tag 99 with a 2-byte dummy value
    contentRecords.set(99, new Uint8Array([0xde, 0xad]))

    const payload = buildCanonicalPayload(contentRecords)
    vectors.push({
      name: 'malformed-unknown-tlv-tag',
      canonical_hex: toHex(payload),
      diagnostic: 'malformed:canonical',
      expected_error: 'UnknownExtension',
    })
  }

  // 7b. malformed-duplicate-tlv-tag — TLV_TOTAL (tag 24) appears twice.
  //     The domain separator is computed over the last-write-wins projection of
  //     the duplicate (i.e. only the second TLV_TOTAL value appears in the
  //     BTreeMap used for the separator hash). The raw wire bytes contain both
  //     occurrences so read_tlv_stream detects the duplicate → InvalidData.
  {
    // Re-use the same minimal content records (no tag 31)
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

    // Compute separator over last-write-wins projection (second TLV_TOTAL value)
    // The second TLV_TOTAL carries value 0x0201 (same as first — makes LWW detectable)
    const firstTotal = contentRecords.get(24)! // 0x0201
    const domSep = computeDomainSeparatorBytes(contentRecords)

    // Build the raw wire stream manually with two TLV_TOTAL records
    // Layout: all content records in ascending order, BUT tag 24 appears twice
    // (first occurrence before tag 24's normal position, second in normal position),
    // then tag 31 with the valid separator.
    // Simplest: emit all records in order, then append a second tag 24 after tag 31.
    // But the BTreeMap in Rust reads all records before checksum — so both TLVs must
    // be in the stream. Place the first TLV_TOTAL at its natural position and append
    // a second TLV_TOTAL with a different value BEFORE tag 31 so the parser sees it.
    //
    // Chosen layout (ascending except second tag-24 injected after tag-22):
    //   tags 2,4,6,8,10,12,14,16,18,20,22 | 24 (first, value=0x0202) | 24 (second=0x0201) | 31
    // The separator is over {2,4,6,8,10,12,14,16,18,20,22,24(0x0201)} — LWW.

    const altTotalValue = new Uint8Array([0x02, 0x02]) // different from original 0x0201

    // Build the TLV stream bytes directly
    function tlvRecord(tag: number, value: Uint8Array): Uint8Array {
      const lenBytes = writeLEB128(value.length)
      const rec = new Uint8Array(1 + lenBytes.length + value.length)
      rec[0] = tag
      rec.set(lenBytes, 1)
      rec.set(value, 1 + lenBytes.length)
      return rec
    }

    const sortedTags = [...contentRecords.keys()].sort((a, b) => a - b)
    const streamParts: Uint8Array[] = []
    for (const tag of sortedTags) {
      if (tag === 24) {
        // First occurrence: alternative value
        streamParts.push(tlvRecord(24, altTotalValue))
        // Second occurrence: original value (this is what LWW projection keeps)
        streamParts.push(tlvRecord(24, firstTotal))
      } else {
        streamParts.push(tlvRecord(tag, contentRecords.get(tag)!))
      }
    }
    // Append domain separator (tag 31)
    streamParts.push(tlvRecord(31, domSep))

    const streamLen = streamParts.reduce((n, p) => n + p.length, 0)
    // COUNT = contentRecords.size + 1 (tag-31) + 1 (extra tag-24) = 14
    const count = sortedTags.length + 1 + 1
    const payload = new Uint8Array(3 + streamLen)
    payload[0] = 0x56 // MAGIC
    payload[1] = 0x01 // VERSION
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

  // ---------------------------------------------------------------------------
  // Write output
  // ---------------------------------------------------------------------------

  const output = {
    schema_version: 1,
    generated_by: '@void-layer/codec v0.1.0',
    generated_at: '2026-05-20',
    vectors,
  }

  fs.mkdirSync(VECTORS_DIR, { recursive: true })
  fs.writeFileSync(OUT_PATH, JSON.stringify(output, null, 2) + '\n')

  console.log(`\nGenerated ${vectors.length} vectors → ${OUT_PATH}\n`)
  for (const v of vectors) {
    if ('expected_error' in v) {
      const mv = v as MalformedVector
      const hex = mv.canonical_hex ?? mv.wire_hex ?? ''
      console.log(
        `  [MALFORMED] ${mv.name.padEnd(38)} hex_len=${String(hex.length).padStart(4)} expected_error=${mv.expected_error}`,
      )
    } else {
      const nv = v as NonMalformedVector
      const comp = isCompressed(nv.wire_hex)
      console.log(
        `  [OK]        ${nv.name.padEnd(38)} canonical_hex_len=${String(nv.canonical_hex.length).padStart(4)} wire_compressed=${comp} roundtrip=${nv.roundtrip}`,
      )
    }
  }

  const failed = vectors.filter(
    (v) => !('expected_error' in v) && !(v as NonMalformedVector).roundtrip,
  )
  if (failed.length > 0) {
    console.error(`\nROUNDTRIP FAILURES: ${failed.map((v) => v.name).join(', ')}`)
    process.exit(1)
  }
  console.log('\nAll roundtrips: OK')
}

main().catch((err) => {
  console.error('Vector generation failed:', err)
  process.exit(1)
})
