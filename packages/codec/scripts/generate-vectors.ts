/**
 * Golden vector generator — @void-layer/codec v4-codec.json
 *
 * Produces the starter set of canonical golden vectors per spec §D-R6.1 and
 * plan-phase2c §T-P2-12 (C2 amendment: TypeScript generator, not Rust bin).
 *
 * Run (from packages/codec root):
 *   pnpm generate-vectors
 *
 * Imports canonical encode/decode from the nodejs-target pkg-node/ (synchronous
 * CJS-style — no Vite plugin required). Wire encode/decode mirrors the JS shim
 * in src/index.ts using the same brotli-wasm peerDep.
 *
 * C4 amendment: wire_hex == canonical_hex when Brotli would expand the payload
 * (small invoices). Each non-malformed vector carries both hex fields regardless.
 */

import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'
import { isCompressed } from './lib/utils.js'
import { type NonMalformedVector } from './scenarios/non-malformed.js'
import { type MalformedVector } from './scenarios/malformed.js'
import { buildAllVectors } from './scenarios/all-vectors.js'
import { demoinvoiceVectors } from './scenarios/demo-invoices.js'

const _filename = fileURLToPath(import.meta.url)
const _dirname = path.dirname(_filename)
const VECTORS_DIR = path.resolve(_dirname, '../vectors')
const OUT_PATH = path.join(VECTORS_DIR, 'v4-codec.json')

type Vector = NonMalformedVector | MalformedVector

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const vectors: Vector[] = [
    ...(await buildAllVectors()),
    ...(await demoinvoiceVectors()),
  ]

  // ---------------------------------------------------------------------------
  // Write output
  // ---------------------------------------------------------------------------

  const output = {
    schema_version: 1,
    generated_by: '@void-layer/codec v0.1.0',
    generated_at: '2026-05-25',
    vectors,
  }

  fs.mkdirSync(VECTORS_DIR, { recursive: true })
  fs.writeFileSync(OUT_PATH, JSON.stringify(output, null, 2) + '\n')

  console.log(`\nGenerated ${vectors.length} vectors → ${OUT_PATH}\n`)
  for (const v of vectors) {
    if ('expected_error' in v) {
      const mv = v as MalformedVector
      const hex = mv.canonical_hex ?? mv.wire_hex ?? ''
      console.log(
        `  [MALFORMED] ${mv.name.padEnd(38)} hex_len=${String(hex.length).padStart(4)} expected_error=${mv.expected_error}`,
      )
    } else {
      const nv = v as NonMalformedVector
      const comp = isCompressed(nv.wire_hex)
      console.log(
        `  [OK]        ${nv.name.padEnd(38)} canonical_hex_len=${String(nv.canonical_hex.length).padStart(4)} wire_compressed=${comp} roundtrip=${nv.roundtrip}`,
      )
    }
  }

  const failed = vectors.filter(
    (v) => !('expected_error' in v) && !(v as NonMalformedVector).roundtrip,
  )
  if (failed.length > 0) {
    console.error(`\nROUNDTRIP FAILURES: ${failed.map((v) => v.name).join(', ')}`)
    process.exit(1)
  }
  console.log('\nAll roundtrips: OK')
}

main().catch((err) => {
  console.error('Vector generation failed:', err)
  process.exit(1)
})
