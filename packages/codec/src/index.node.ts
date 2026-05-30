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
 */

import { createRequire } from 'node:module'
import type { BrotliWasmType } from 'brotli-wasm'
import type { Invoice } from '@void-layer/types'

// createRequire lets ESM load CJS modules — necessary because wasm-pack
// --target nodejs emits CommonJS (exports.xxx + require('fs') + __dirname).
const require = createRequire(import.meta.url)

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeWasm = require('../pkg-node/void_layer_codec.js') as {
  encodeInvoiceCanonical: (invoice: unknown) => Uint8Array
  decodeInvoiceCanonical: (bytes: Uint8Array) => unknown
  receiptHash: (canonical_bytes: Uint8Array) => Uint8Array
}

export const encodeInvoiceCanonical = nodeWasm.encodeInvoiceCanonical
export const decodeInvoiceCanonical = nodeWasm.decodeInvoiceCanonical
export const receiptHash = nodeWasm.receiptHash

// ---------------------------------------------------------------------------
// Brotli lazy init — mirrors index.ts exactly
// ---------------------------------------------------------------------------

const COMPRESSED_FLAG = 0x80
const MAX_DECOMPRESSED_BYTES = 262144

// brotli-wasm exports map: `require` condition → index.node.js (fs + sync WASM init, no fetch).
// ESM `import` condition → index.web.js which uses fetch() and fails in plain Node.
// We must use createRequire to force the `require` condition.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const brotliNodeMod = require('brotli-wasm') as { default: Promise<BrotliWasmType> }

let _brotli: BrotliWasmType | null = null

async function getBrotli(): Promise<BrotliWasmType> {
  if (!_brotli) {
    _brotli = await brotliNodeMod.default
  }
  return _brotli
}

// ---------------------------------------------------------------------------
// decompressBounded — mirrors index.ts exactly
// ---------------------------------------------------------------------------

function decompressBounded(
  brotli: BrotliWasmType,
  input: Uint8Array,
  maxBytes: number,
): Uint8Array {
  const CHUNK = maxBytes
  const stream = new brotli.DecompressStream()
  const chunks: Uint8Array[] = []
  let total = 0
  let inputOffset = 0

  while (true) {
    const slice = input.slice(inputOffset)
    const result = stream.decompress(slice, CHUNK)
    inputOffset += result.input_offset

    if (result.buf.length === 0 && result.input_offset === 0) {
      throw new Error('truncated or corrupt brotli stream (no progress)')
    }

    if (result.buf.length > 0) {
      total += result.buf.length
      if (total > maxBytes) {
        throw new Error(
          `decompressed wire body exceeds MAX_DECOMPRESSED_BYTES (${maxBytes})`,
        )
      }
      chunks.push(result.buf)
    }

    if (result.code === 0) break
    if (result.code === 1) break
  }

  const out = new Uint8Array(total)
  let pos = 0
  for (const chunk of chunks) {
    out.set(chunk, pos)
    pos += chunk.length
  }
  return out
}

// ---------------------------------------------------------------------------
// Wire encode / decode — mirrors index.ts exactly
// ---------------------------------------------------------------------------

export async function encodeInvoiceWire(invoice: Invoice): Promise<Uint8Array> {
  const canonical: Uint8Array = encodeInvoiceCanonical(invoice)
  const brotli = await getBrotli()
  const body = canonical.slice(2)
  const compressed = brotli.compress(body, { quality: 11 })

  if (compressed.length >= body.length) return canonical

  const result = new Uint8Array(2 + compressed.length)
  result[0] = canonical[0]!
  result[1] = canonical[1]! | COMPRESSED_FLAG
  result.set(compressed, 2)
  return result
}

export async function decodeInvoiceWire(bytes: Uint8Array): Promise<Invoice> {
  if (bytes.length < 3 || !(bytes[1]! & COMPRESSED_FLAG)) {
    return decodeInvoiceCanonical(bytes) as Invoice
  }

  const brotli = await getBrotli()
  const compressedBody = bytes.slice(2)
  const decompressed = decompressBounded(brotli, compressedBody, MAX_DECOMPRESSED_BYTES)

  const canonical = new Uint8Array(2 + decompressed.length)
  canonical[0] = bytes[0]!
  canonical[1] = bytes[1]! & 0x7f
  canonical.set(decompressed, 2)

  return decodeInvoiceCanonical(canonical) as Invoice
}
