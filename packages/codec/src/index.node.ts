/**
 * @void-layer/codec — Node.js entry point (plain Node ESM, no bundler).
 *
 * Loads the wasm-pack --target nodejs CJS glue via createRequire (which
 * uses fs.readFileSync + WebAssembly.Module — no ESM .wasm import needed).
 * Re-exports the same public API as index.ts so the `node` export condition
 * resolves to a working module for plain `node` consumers.
 *
 * The bundler entry (index.ts / pkg/) remains available via the `import`
 * export condition for bundler consumers (e.g. vl/app via webpack/turbopack).
 *
 * Wire logic lives in wire.ts (shared with index.ts). The only difference
 * from index.ts is how brotli-wasm is loaded: createRequire forces the
 * `require` export condition → index.node.js (fs-based, no fetch). The ESM
 * `import` condition → index.web.js which calls fetch() and fails in Node.
 */

import { createRequire } from 'module'
import type { BrotliWasmType } from 'brotli-wasm'
import type { Invoice } from '@void-layer/types'
import { encodeWire, decodeWire } from './wire.js'

// createRequire lets ESM load CJS modules:
//   - wasm-pack --target nodejs glue (exports.xxx + require('fs') + __dirname)
//   - brotli-wasm: `require` condition → index.node.js (fs-based, no fetch)
const require = createRequire(import.meta.url)

const nodeWasm = require('../pkg-node/void_layer_codec.js') as {
  encodeInvoiceCanonical: (invoice: Invoice) => Uint8Array
  decodeInvoiceCanonical: (bytes: Uint8Array) => Invoice
  receiptHash: (canonical_bytes: Uint8Array) => Uint8Array
}

export const encodeInvoiceCanonical = nodeWasm.encodeInvoiceCanonical
export const decodeInvoiceCanonical = nodeWasm.decodeInvoiceCanonical
export const receiptHash = nodeWasm.receiptHash

// ---------------------------------------------------------------------------
// Brotli lazy init — uses require to force `require` condition (no fetch)
// ---------------------------------------------------------------------------

const brotliNodeMod = require('brotli-wasm') as { default: Promise<BrotliWasmType> }

let _brotli: BrotliWasmType | null = null

async function getBrotli(): Promise<BrotliWasmType> {
  if (!_brotli) {
    _brotli = await brotliNodeMod.default
  }
  return _brotli
}

export async function encodeInvoiceWire(invoice: Invoice): Promise<Uint8Array> {
  return encodeWire(invoice, encodeInvoiceCanonical, getBrotli)
}

export async function decodeInvoiceWire(bytes: Uint8Array): Promise<Invoice> {
  return decodeWire(bytes, decodeInvoiceCanonical, getBrotli)
}
