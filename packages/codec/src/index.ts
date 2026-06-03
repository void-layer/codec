/**
 * @void-layer/codec JS shim — public entry point (bundler / ESM).
 *
 * Exposes 5 functions:
 *   - encodeInvoiceCanonical / decodeInvoiceCanonical  (WASM canonical, no Brotli)
 *   - receiptHash                                       (keccak-256 of canonical bytes)
 *   - encodeInvoiceWire / decodeInvoiceWire             (Brotli-compressed wire format)
 *
 * Wire logic lives in wire.ts (shared with index.node.ts). The only difference
 * between this entry and index.node.ts is how brotli-wasm is loaded:
 * ESM dynamic import() here vs createRequire (require condition) in index.node.ts.
 */

import type { BrotliWasmType } from 'brotli-wasm'
import type { Invoice } from '@void-layer/types'
import { encodeWire, decodeWire } from './wire.js'

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

let _brotli: BrotliWasmType | null = null

async function getBrotli(): Promise<BrotliWasmType> {
  if (!_brotli) {
    const mod = await import('brotli-wasm')
    const instance = await mod.default
    _brotli = instance
  }
  return _brotli
}

export async function encodeInvoiceWire(invoice: Invoice): Promise<Uint8Array> {
  return encodeWire(invoice, encodeInvoiceCanonical, getBrotli)
}

export async function decodeInvoiceWire(bytes: Uint8Array): Promise<Invoice> {
  return decodeWire(bytes, decodeInvoiceCanonical, getBrotli)
}
