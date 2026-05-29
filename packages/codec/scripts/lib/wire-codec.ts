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

// Defense-in-depth: same cap as src/index.ts — prevents a bomb vector in the
// parity corpus from OOM-ing CI. Dev-only script, not published.
const MAX_DECOMPRESSED_BYTES = 262144
const CHUNK = MAX_DECOMPRESSED_BYTES

export async function wireDecode(bytes: Uint8Array): Promise<unknown> {
  if (bytes.length < 3 || !(bytes[1]! & COMPRESSED_FLAG)) {
    return decodeInvoiceCanonical(bytes)
  }
  const brotli = await brotliWasmInit
  const input = bytes.slice(2)
  const stream = new brotli.DecompressStream()
  const chunks: Uint8Array[] = []
  let total = 0
  let inputOffset = 0
  while (true) {
    const result = stream.decompress(input.slice(inputOffset), CHUNK)
    inputOffset += result.input_offset
    if (result.buf.length === 0 && result.input_offset === 0) {
      throw new Error('truncated or corrupt brotli stream (no progress)')
    }
    if (result.buf.length > 0) {
      total += result.buf.length
      if (total > MAX_DECOMPRESSED_BYTES) {
        throw new Error(`decompressed body exceeds MAX_DECOMPRESSED_BYTES (${MAX_DECOMPRESSED_BYTES})`)
      }
      chunks.push(result.buf)
    }
    // code=0 (ResultSuccess) or code=1 (NeedsMoreInput, terminal for single-chunk) = done.
    if (result.code === 0 || result.code === 1) break
    // code=2 (NeedsMoreOutput) — continue to drain more output.
  }
  const decompressed = new Uint8Array(total)
  let pos = 0
  for (const chunk of chunks) { decompressed.set(chunk, pos); pos += chunk.length }
  const canonical = new Uint8Array(2 + decompressed.length)
  canonical[0] = bytes[0]!
  canonical[1] = bytes[1]! & 0x7f
  canonical.set(decompressed, 2)
  return decodeInvoiceCanonical(canonical)
}
