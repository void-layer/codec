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

  // 5. Malformed (3)

  // 5a. Corrupted brotli: COMPRESSED_FLAG set, body is not valid Brotli
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
