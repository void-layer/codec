/**
 * Loader-agnostic wire encode/decode logic.
 *
 * Both entry points (index.ts bundler path, index.node.ts Node path) share
 * this module. The ONLY per-entry difference is how brotli-wasm is obtained:
 * callers pass a `getBrotli` factory; this module owns all wire framing logic.
 *
 * Security note: decompressBounded implements the truncated-stream DoS guard
 * (d83cef9). Any change here applies to both entry points simultaneously —
 * that is the point of this extraction.
 */

import type { BrotliWasmType } from 'brotli-wasm'
import type { Invoice } from '@void-layer/types'

export const COMPRESSED_FLAG = 0x80

/**
 * Hard cap on the size of a Brotli-decompressed wire body. A small (~1 KB)
 * compressed payload can otherwise expand to hundreds of MB — a decompression
 * bomb that OOMs the client.
 *
 * = MAX_TLV_COUNT(64) * MAX_VALUE_SIZE(4096) — must accept any valid canonical payload.
 * A valid invoice is bounded well below the ~2 KB URL budget in practice;
 * this cap exists to reject decompression bombs, not to restrict valid payloads.
 */
export const MAX_DECOMPRESSED_BYTES = 262144

/**
 * Bounded streaming Brotli decompression.
 *
 * Uses `DecompressStream` to decompress in chunks of `chunkSize` bytes,
 * checking the accumulated total BEFORE appending each chunk. Aborts as soon
 * as `total > MAX_DECOMPRESSED_BYTES` — the bomb never fully materialises in
 * memory.
 */
export function decompressBounded(
  brotli: BrotliWasmType,
  input: Uint8Array,
  maxBytes: number,
): Uint8Array {
  // Output chunk size: use the cap itself as the chunk size so we can detect
  // overrun in a single iteration for valid payloads, while still catching
  // multi-chunk bombs on the second iteration.
  const CHUNK = maxBytes
  const stream = new brotli.DecompressStream()
  const chunks: Uint8Array[] = []
  let total = 0
  let inputOffset = 0

  // Feed all input; loop over output chunks.
  // BrotliStreamResultCode: ResultSuccess=0, NeedsMoreInput=1, NeedsMoreOutput=2
  // The brotli-wasm DecompressStream API: corrupt input throws synchronously.
  // code=1 (NeedsMoreInput) with all input consumed = terminal success state.
  // code=2 (NeedsMoreOutput) = more output available; loop with same/empty input.
  while (true) {
    const slice = input.slice(inputOffset)
    const result = stream.decompress(slice, CHUNK)
    inputOffset += result.input_offset

    if (result.buf.length === 0 && result.input_offset === 0) {
      throw new Error('truncated or corrupt brotli stream (no progress)')
    }

    if (result.buf.length > 0) {
      total += result.buf.length
      // Check BEFORE accumulating this chunk — bomb guard fires here.
      if (total > maxBytes) {
        throw new Error(
          `decompressed wire body exceeds MAX_DECOMPRESSED_BYTES (${maxBytes})`,
        )
      }
      chunks.push(result.buf)
    }

    // code=0 (ResultSuccess) — stream fully closed.
    if (result.code === 0) break

    // code=1 (NeedsMoreInput) — all input consumed; this is the normal terminal
    // state for a single-chunk decompress (ResultSuccess is only emitted when
    // the underlying Brotli stream closes, which may not happen here).
    if (result.code === 1) break

    // code=2 (NeedsMoreOutput) — continue the loop to drain more output chunks.
  }

  // Concatenate all chunks into a single Uint8Array.
  const out = new Uint8Array(total)
  let pos = 0
  for (const chunk of chunks) {
    out.set(chunk, pos)
    pos += chunk.length
  }
  return out
}

// ---------------------------------------------------------------------------
// Wire encode — MAGIC + (VERSION | COMPRESSED_FLAG) + brotli(body)
// Falls back to uncompressed if Brotli expands the payload.
//
// Input:  invoice object + per-entry WASM encode fn + getBrotli factory
// Output: [MAGIC][VERSION | 0x80][brotli([COUNT][TLV records...])]
//         OR uncompressed canonical bytes if Brotli would expand.
//
// Mirrors: compressPayload() in tlv-codec/compress.ts
// ---------------------------------------------------------------------------

export async function encodeWire(
  invoice: Invoice,
  encodeCanonical: (inv: Invoice) => Uint8Array,
  getBrotli: () => Promise<BrotliWasmType>,
): Promise<Uint8Array> {
  const canonical: Uint8Array = encodeCanonical(invoice)

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

export async function decodeWire(
  bytes: Uint8Array,
  decodeCanonical: (b: Uint8Array) => Invoice,
  getBrotli: () => Promise<BrotliWasmType>,
): Promise<Invoice> {
  if (bytes.length < 3 || !(bytes[1]! & COMPRESSED_FLAG)) {
    return decodeCanonical(bytes)
  }

  const brotli = await getBrotli()
  const compressedBody = bytes.slice(2)

  // Decompression-bomb guard: streaming bounded decompress — the check fires
  // INSIDE the loop before each chunk is accumulated, so the bomb never fully
  // allocates. JS Error (not CodecError — this is the JS shim layer).
  const decompressed = decompressBounded(brotli, compressedBody, MAX_DECOMPRESSED_BYTES)

  const canonical = new Uint8Array(2 + decompressed.length)
  canonical[0] = bytes[0]! // MAGIC
  canonical[1] = bytes[1]! & 0x7f // VERSION without COMPRESSED_FLAG
  canonical.set(decompressed, 2)

  return decodeCanonical(canonical)
}
