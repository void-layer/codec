/**
 * Golden vector freeze-gate — @void-layer/codec
 *
 * Modes:
 *   --check  (default / CI): re-derive vectors from the codec WASM and diff
 *            against the committed v4-codec.json. Returns a non-zero exit code
 *            on ANY mismatch. Never overwrites the file.
 *   --write  (deliberate local re-capture only): same derivation, then writes
 *            the result to disk. Use only after a reviewed, intentional update
 *            to the frozen oracle (new TLV type, dict code, etc.).
 *
 * The frozen oracle was captured from vl/app master c658fff (release v1.1.2).
 * Any change to canonical_hex or wire_hex is a regression against that deployed
 * reference and must be reviewed by Kai before --write is used.
 *
 * Run (from packages/codec root):
 *   pnpm check-vectors           # check mode (CI default)
 *   pnpm check-vectors -- --write  # write mode (local only)
 */

import * as fs from 'node:fs'
import * as path from 'node:path'
import * as crypto from 'node:crypto'
import { fileURLToPath } from 'node:url'
import { buildAllVectors } from './scenarios/all-vectors.js'
import { demoinvoiceVectors } from './scenarios/demo-invoices.js'
import type { NonMalformedVector } from './scenarios/non-malformed.js'
import type { MalformedVector } from './scenarios/malformed.js'

const _filename = fileURLToPath(import.meta.url)
const _dirname = path.dirname(_filename)
const VECTORS_DIR = path.resolve(_dirname, '../vectors')
const OUT_PATH = path.join(VECTORS_DIR, 'v4-codec.json')

// Provenance constants — update only on deliberate oracle re-capture.
const CAPTURED_FROM_VL_APP_SHA = 'c658fff'
const FROZEN_AT = '2026-05-30'

type Vector = NonMalformedVector | MalformedVector

/**
 * Compute sha256 of the canonical vector payload (the "vectors" array JSON,
 * without the header fields). This is the content_hash stored in the frozen JSON.
 */
export function computeContentHash(vectors: unknown[]): string {
  const payload = JSON.stringify(vectors)
  return crypto.createHash('sha256').update(payload).digest('hex')
}

async function deriveVectors(): Promise<Vector[]> {
  return [
    ...(await buildAllVectors()),
    ...(await demoinvoiceVectors()),
  ]
}

/**
 * Run the freeze-gate check. Returns a list of mismatch descriptions.
 * Empty list = PASS. Non-empty = FAIL.
 *
 * Exported so the vitest runner can assert without process.exit().
 */
export async function runFreezeCheck(committedPath: string = OUT_PATH): Promise<string[]> {
  const derived = await deriveVectors()
  const mismatches: string[] = []

  if (!fs.existsSync(committedPath)) {
    return ['vectors/v4-codec.json does not exist — run pnpm check-vectors -- --write']
  }

  const committed = JSON.parse(fs.readFileSync(committedPath, 'utf-8')) as {
    vectors: Vector[]
    content_hash?: string
    frozen?: boolean
    captured_from_vl_app_sha?: string
  }

  const committedVectors = committed.vectors ?? []
  const derivedByName = new Map(derived.map((v) => [v.name, v]))
  const committedByName = new Map(committedVectors.map((v) => [v.name, v]))

  // Check every derived vector against committed
  for (const [name, derivedV] of derivedByName) {
    const committedV = committedByName.get(name)
    if (!committedV) {
      mismatches.push(`MISSING_IN_COMMITTED: vector=${name}`)
      continue
    }

    const dv = derivedV as NonMalformedVector
    const cv = committedV as NonMalformedVector

    if ('canonical_hex' in dv && 'canonical_hex' in cv) {
      if (dv.canonical_hex !== cv.canonical_hex) {
        mismatches.push(
          `CANONICAL_MISMATCH vector=${name}\n` +
          `  committed: ${cv.canonical_hex}\n` +
          `  derived:   ${dv.canonical_hex}`,
        )
      }
    }
    if ('wire_hex' in dv && 'wire_hex' in cv) {
      if (dv.wire_hex !== cv.wire_hex) {
        mismatches.push(
          `WIRE_MISMATCH vector=${name}\n` +
          `  committed: ${cv.wire_hex}\n` +
          `  derived:   ${dv.wire_hex}`,
        )
      }
    }
    if ('receipt_hash_hex' in dv && 'receipt_hash_hex' in cv) {
      if (dv.receipt_hash_hex !== cv.receipt_hash_hex) {
        mismatches.push(
          `RECEIPT_HASH_MISMATCH vector=${name}\n` +
          `  committed: ${cv.receipt_hash_hex}\n` +
          `  derived:   ${dv.receipt_hash_hex}`,
        )
      }
    }
  }

  // Check for vectors in committed but not in derived.
  // decode_* vectors are static forward-compat fixtures not re-derived from the
  // codec scenario builders — intentionally present only in the JSON oracle.
  for (const name of committedByName.keys()) {
    if (!derivedByName.has(name) && !name.startsWith('decode_')) {
      mismatches.push(`EXTRA_IN_COMMITTED: vector=${name} (not in derived set — stale?)`)
    }
  }

  // Integrity gate: sha256 is computed over the COMMITTED vectors array as
  // stored in the frozen JSON — not over WASM-derived output. decode-only
  // fixtures (roundtrip===false) are part of the frozen oracle and must be
  // included in the hash. This detects accidental corruption (bitrot, bad
  // merge, unintended manual edit). It is NOT a tamper-proof security
  // boundary — a malicious editor can edit the vectors and re-stamp the hash.
  // Real cross-impl byte identity is enforced by the codec-drift gate (check a).
  if (committed.content_hash) {
    const expectedHash = computeContentHash(committedVectors)
    if (committed.content_hash !== expectedHash) {
      mismatches.push(
        `CONTENT_HASH_MISMATCH (integrity check failed — vectors edited without re-stamping hash)\n` +
        `  committed: ${committed.content_hash}\n` +
        `  recomputed: ${expectedHash}`,
      )
    }
  }

  return mismatches
}

/**
 * Write mode: re-derive and stamp the oracle with provenance headers.
 * Only called with explicit --write flag.
 */
export async function runWrite(): Promise<void> {
  const derived = await deriveVectors()
  const content_hash = computeContentHash(derived)
  const output = {
    schema_version: 1,
    frozen: true,
    captured_from_vl_app_sha: CAPTURED_FROM_VL_APP_SHA,
    frozen_at: FROZEN_AT,
    content_hash,
    generated_by: '@void-layer/codec v0.1.0',
    vectors: derived,
  }
  fs.mkdirSync(VECTORS_DIR, { recursive: true })
  fs.writeFileSync(OUT_PATH, JSON.stringify(output, null, 2) + '\n')
  console.log(`[freeze-gate] wrote ${derived.length} vectors → ${OUT_PATH}`)
  console.log(`[freeze-gate] content_hash: ${content_hash}`)
}

// ---------------------------------------------------------------------------
// CLI entry point (direct invocation only — not called when imported as module)
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const args = process.argv.slice(2)
  const writeMode = args.includes('--write')

  if (writeMode) {
    console.log(`\n[freeze-gate] mode=write`)
    console.log(`[freeze-gate] oracle: vectors/v4-codec.json`)
    console.log(`[freeze-gate] captured_from_vl_app_sha: ${CAPTURED_FROM_VL_APP_SHA}\n`)
    await runWrite()
    return
  }

  console.log(`\n[freeze-gate] mode=check`)
  console.log(`[freeze-gate] oracle: vectors/v4-codec.json`)
  console.log(`[freeze-gate] captured_from_vl_app_sha: ${CAPTURED_FROM_VL_APP_SHA}\n`)

  const mismatches = await runFreezeCheck()

  if (mismatches.length > 0) {
    console.error('[freeze-gate] FAIL: v4-codec.json has drifted from codec output\n')
    for (const m of mismatches) console.error(`  ${m}\n`)
    console.error('\nTo re-capture: pnpm check-vectors -- --write (requires Kai review)')
    process.exit(1)
  }

  console.log(`[freeze-gate] PASS — all vectors match committed oracle`)
}

// Only run main() when this file is the direct entrypoint, not when imported as a module.
// Vitest imports this as a module to call runFreezeCheck()/runWrite() directly.
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((err) => {
    console.error('[freeze-gate] fatal:', err)
    process.exit(1)
  })
}
