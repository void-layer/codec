/**
 * vl/app provenance verification test.
 *
 * Proves that committed canonical_hex / wire_hex in v4-codec.json were
 * genuinely captured from vl/app's TS encoder at master c658fff.
 *
 * Method: imports vl/app encode.ts directly (via @/ alias → voidpay/src),
 * encodes each non-malformed vector's decoded invoice with its deterministic
 * salt, decompresses the wire output to recover canonical bytes, and compares
 * against the committed hex strings.
 *
 * Run (from packages/codec):
 *   pnpm exec vitest run scripts/vlapp-provenance.test.ts --config scripts/vlapp-provenance.config.ts
 */

import { test } from 'vitest'
import * as fs from 'node:fs'
import { encodeInvoice } from '@/features/invoice-codec/lib/encode.js'
import { decompressPayload, decodeBase64url } from '@/shared/lib/tlv-codec/index.js'

const VECTORS_PATH = new URL('../vectors/v4-codec.json', import.meta.url).pathname

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

function hexToBytes(hex: string): Uint8Array {
  const b = new Uint8Array(hex.length / 2)
  for (let i = 0; i < hex.length; i += 2) b[i / 2] = parseInt(hex.slice(i, i + 2), 16)
  return b
}

function vectorDecodedToInvoice(d: Record<string, unknown>) {
  const from = d['from'] as Record<string, unknown>
  const client = d['client'] as Record<string, unknown>
  const items = d['items'] as Array<Record<string, unknown>>
  return {
    invoiceId: d['invoice_id'] as string,
    issuedAt: d['issued_at'] as number,
    dueAt: d['due_at'] as number,
    networkId: d['network_id'] as number,
    currency: d['currency'] as string,
    decimals: d['decimals'] as number,
    tokenAddress: d['token_address'] as string | undefined,
    from: {
      name: from['name'] as string,
      walletAddress: from['wallet_address'] as string,
      email: from['email'] as string | undefined,
      phone: from['phone'] as string | undefined,
      physicalAddress: from['physical_address'] as string | undefined,
      taxId: from['tax_id'] as string | undefined,
    },
    client: {
      name: client['name'] as string,
      walletAddress: client['wallet_address'] as string | undefined,
      email: client['email'] as string | undefined,
      phone: client['phone'] as string | undefined,
      physicalAddress: client['physical_address'] as string | undefined,
      taxId: client['tax_id'] as string | undefined,
    },
    items: items.map((i) => ({
      description: i['description'] as string,
      quantity: i['quantity'] as number,
      rate: i['rate'] as string,
    })),
    notes: d['notes'] as string | undefined,
    tax: d['tax'] as string | undefined,
    discount: d['discount'] as string | undefined,
    total: d['total'] as string,
  }
}

const json = JSON.parse(fs.readFileSync(VECTORS_PATH, 'utf-8')) as {
  captured_from_vl_app_sha: string
  vectors: Array<{
    name: string
    canonical_hex?: string
    wire_hex?: string
    decoded?: Record<string, unknown> | null
    diagnostic?: string
    roundtrip?: boolean
  }>
}

// Encoder-provable vectors: have canonical bytes, a decoded invoice to re-encode,
// are not malformed fixtures, and are not decode-only forward-compat fixtures
// (roundtrip===false means the encoder never emits this wire — correct by design,
// per BOLT-12 odd-ignore and strict-monotone rules; verified by DECODER tests only).
const nonMalformed = json.vectors.filter(
  (v) =>
    v.canonical_hex &&
    v.decoded &&
    !v.diagnostic?.startsWith('malformed') &&
    v.roundtrip !== false,
)

test(
  `vl/app provenance: all ${nonMalformed.length} non-malformed vectors match vl/app TS encoder (sha=${json.captured_from_vl_app_sha})`,
  async () => {
    const mismatches: string[] = []

    for (const v of nonMalformed) {
      const d = v.decoded!
      const salt = hexToBytes(d['salt'] as string)
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const invoice = vectorDecodedToInvoice(d) as any

      const wireB64 = await encodeInvoice(invoice, salt)
      const wireBytes = decodeBase64url(wireB64)
      const wire_hex = toHex(wireBytes)
      const canonicalBytes = await decompressPayload(wireBytes)
      const canonical_hex = toHex(canonicalBytes)

      if (canonical_hex !== v.canonical_hex) {
        mismatches.push(
          `CANONICAL_MISMATCH vector=${v.name}\n` +
            `  committed: ${v.canonical_hex}\n` +
            `  derived:   ${canonical_hex}`,
        )
      }
      if (wire_hex !== v.wire_hex) {
        mismatches.push(
          `WIRE_MISMATCH vector=${v.name}\n` +
            `  committed: ${v.wire_hex}\n` +
            `  derived:   ${wire_hex}`,
        )
      }
    }

    if (mismatches.length > 0) {
      throw new Error(
        `[vl/app provenance] ${mismatches.length} mismatch(es) — committed hex ≠ vl/app TS output:\n\n` +
          mismatches.join('\n\n') +
          '\n\nProvenance stamp is INVALID — escalate to Kai before committing.',
      )
    }
  },
  120_000,
)
