/**
 * @void-layer/codec JS shim — public entry point.
 *
 * Exposes 4 functions:
 *   - encodeInvoiceCanonical / decodeInvoiceCanonical  (WASM canonical, no Brotli)
 *   - encodeInvoiceWire / decodeInvoiceWire             (Brotli-compressed wire format)
 *
 * Brotli compression is handled here via `brotli-wasm` peerDependency.
 * COMPRESSED_FLAG logic mirrors vl/app/src/shared/lib/tlv-codec/compress.ts §compressPayload.
 */

import type { BrotliWasmType } from 'brotli-wasm'

// Re-export the 2 canonical WASM functions directly.
export {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
} from '../pkg/void_layer_codec.js'

// ---------------------------------------------------------------------------
// Brotli lazy init (mirrors compressPayload reference pattern)
// ---------------------------------------------------------------------------

const COMPRESSED_FLAG = 0x80

let _brotli: BrotliWasmType | null = null

async function getBrotli(): Promise<BrotliWasmType> {
  if (!_brotli) {
    const mod = await import('brotli-wasm')
    const instance = await mod.default
    _brotli = instance
  }
  return _brotli
}

// ---------------------------------------------------------------------------
// Wire encode — MAGIC + (VERSION | COMPRESSED_FLAG) + brotli(body)
// Falls back to uncompressed if Brotli expands the payload.
//
// Input:  invoice object (same shape as encodeInvoiceCanonical)
// Output: [MAGIC][VERSION | 0x80][brotli([COUNT][TLV records...])]
//         OR uncompressed canonical bytes if Brotli would expand.
//
// Mirrors: compressPayload() in tlv-codec/compress.ts
// ---------------------------------------------------------------------------

export async function encodeInvoiceWire(invoice: unknown): Promise<Uint8Array> {
  const { encodeInvoiceCanonical: encodeCanonical } = await import(
    '../pkg/void_layer_codec.js'
  )
  const canonical: Uint8Array = encodeCanonical(invoice)

  if (canonical.length < 3) return canonical

  const brotli = await getBrotli()
  const body = canonical.slice(2) // [COUNT][TLV records...]
  const compressed = brotli.compress(body, { quality: 11 })

  if (compressed.length >= body.length) return canonical

  const result = new Uint8Array(2 + compressed.length)
  result[0] = canonical[0]! // MAGIC
  result[1] = canonical[1]! | COMPRESSED_FLAG // VERSION | 0x80
  result.set(compressed, 2)
  return result
}

// ---------------------------------------------------------------------------
// Wire decode — detects COMPRESSED_FLAG and decompresses if set.
// Accepts both compressed wire bytes and uncompressed canonical bytes.
//
// Mirrors: decompressPayload() in tlv-codec/compress.ts
// ---------------------------------------------------------------------------

export async function decodeInvoiceWire(bytes: Uint8Array): Promise<unknown> {
  const { decodeInvoiceCanonical: decodeCanonical } = await import(
    '../pkg/void_layer_codec.js'
  )

  if (bytes.length < 3 || !(bytes[1]! & COMPRESSED_FLAG)) {
    return decodeCanonical(bytes)
  }

  const brotli = await getBrotli()
  const compressedBody = bytes.slice(2)
  const decompressed = brotli.decompress(compressedBody)

  const canonical = new Uint8Array(2 + decompressed.length)
  canonical[0] = bytes[0]! // MAGIC
  canonical[1] = bytes[1]! & 0x7f // VERSION without COMPRESSED_FLAG
  canonical.set(decompressed, 2)

  return decodeCanonical(canonical)
}
