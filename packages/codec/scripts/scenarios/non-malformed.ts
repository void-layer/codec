/**
 * Non-malformed vector generator — well-formed invoices that must roundtrip.
 */

import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
  receiptHash,
} from '../../pkg-node/void_layer_codec.js'
import { wireEncode, wireDecode } from '../lib/wire-codec.js'
import { toHex } from '../lib/utils.js'

export const WIRE_DIAG =
  'wire_hex = Brotli-compressed wire, or == canonical_hex when Brotli expands (small payloads)'

export interface NonMalformedVector {
  name: string
  canonical_hex: string
  wire_hex: string
  receipt_hash_hex: string
  decoded: unknown
  roundtrip: boolean
  diagnostic: string
}

export async function nonMalformed(
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
