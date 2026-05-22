/**
 * @void-layer/codec JS shim — public entry point.
 *
 * Exposes 5 functions:
 *   - encodeInvoiceCanonical / decodeInvoiceCanonical  (WASM canonical, no Brotli)
 *   - receiptHash                                       (keccak-256 of canonical bytes)
 *   - encodeInvoiceWire / decodeInvoiceWire             (Brotli-compressed wire format)
 *
 * Brotli compression is handled here via `brotli-wasm` peerDependency.
 * COMPRESSED_FLAG logic mirrors vl/app/src/shared/lib/tlv-codec/compress.ts §compressPayload.
 */

import type { BrotliWasmType } from 'brotli-wasm'
import type { Invoice } from '@void-layer/types'

// Import the canonical WASM functions for use in the wire shim below, and
// re-export them as part of the public API.
import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
  receiptHash,
} from '../pkg/void_layer_codec.js'

export { encodeInvoiceCanonical, decodeInvoiceCanonical, receiptHash }

// ---------------------------------------------------------------------------
// Brotli lazy init (mirrors compressPayload reference pattern)
// ---------------------------------------------------------------------------

const COMPRESSED_FLAG = 0x80

/**
 * Hard cap on the size of a Brotli-decompressed wire body. A small (~1 KB)
 * compressed payload can otherwise expand to hundreds of MB — a decompression
 * bomb that OOMs the client. 64 KB is generous: a valid canonical invoice is
 * bounded well below the ~2 KB URL budget.
 */
const MAX_DECOMPRESSED_BYTES = 65536

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

export async function encodeInvoiceWire(invoice: Invoice): Promise<Uint8Array> {
  // encodeInvoiceCanonical is statically re-exported above — no dynamic import.
  const canonical: Uint8Array = encodeInvoiceCanonical(invoice)

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

export async function decodeInvoiceWire(bytes: Uint8Array): Promise<Invoice> {
  // decodeInvoiceCanonical is statically re-exported above — no dynamic import.
  if (bytes.length < 3 || !(bytes[1]! & COMPRESSED_FLAG)) {
    return decodeInvoiceCanonical(bytes)
  }

  const brotli = await getBrotli()
  const compressedBody = bytes.slice(2)
  const decompressed = brotli.decompress(compressedBody)

  // Decompression-bomb guard: reject a body that expands past the cap before
  // allocating the canonical buffer.
  if (decompressed.length > MAX_DECOMPRESSED_BYTES) {
    throw new Error(
      `decompressed wire body ${decompressed.length} bytes exceeds ` +
        `MAX_DECOMPRESSED_BYTES (${MAX_DECOMPRESSED_BYTES})`,
    )
  }

  const canonical = new Uint8Array(2 + decompressed.length)
  canonical[0] = bytes[0]! // MAGIC
  canonical[1] = bytes[1]! & 0x7f // VERSION without COMPRESSED_FLAG
  canonical.set(decompressed, 2)

  return decodeInvoiceCanonical(canonical)
}
