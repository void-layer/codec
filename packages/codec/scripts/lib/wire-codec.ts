/**
 * Wire encode/decode — mirrors src/index.ts logic exactly.
 * Brotli-compresses the canonical payload body when compression saves bytes.
 */

import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
} from '../../pkg-node/void_layer_codec.js'
import brotliWasmInit from 'brotli-wasm'

export const COMPRESSED_FLAG = 0x80

export async function wireEncode(invoice: unknown): Promise<Uint8Array> {
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

export async function wireDecode(bytes: Uint8Array): Promise<unknown> {
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
