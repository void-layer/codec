/**
 * Vitest wrapper — runs the freeze-gate checker as a test so vitest's
 * module resolver (brotli-wasm alias, wasm plugin) are active.
 *
 * CI usage:
 *   pnpm -C packages/codec exec vitest run scripts/run-check-vectors.test.ts --coverage.enabled=false
 *
 * This test FAILS if any derived vector byte differs from the committed oracle.
 * A failure here means @void-layer/codec output has drifted from the frozen
 * vl/app reference (captured at c658fff). Do NOT suppress — escalate to Kai.
 *
 * NEGATIVE TEST: a deliberately mutated single byte also lives here to prove
 * the gate has teeth (see "freeze-gate rejects mutated vector").
 */
import { test, expect } from 'vitest'
import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'
import { runFreezeCheck } from './check-vectors.js'

const _dirname = path.dirname(fileURLToPath(import.meta.url))
const VECTORS_PATH = path.resolve(_dirname, '../vectors/v4-codec.json')

// ---------------------------------------------------------------------------
// Positive test — codec output must match frozen oracle
// ---------------------------------------------------------------------------

test(
  'freeze-gate: codec output matches committed v4-codec.json (oracle integrity)',
  async () => {
    const mismatches = await runFreezeCheck()
    if (mismatches.length > 0) {
      // Surface all mismatches in one assertion so CI output is actionable.
      throw new Error(
        `[freeze-gate] ${mismatches.length} mismatch(es) detected:\n\n` +
        mismatches.map((m) => `  ${m}`).join('\n\n') +
        '\n\nTo re-capture: pnpm check-vectors -- --write (requires Kai review)',
      )
    }
  },
  120_000,
)

// ---------------------------------------------------------------------------
// Negative test — proves the gate has teeth: a mutated byte must be caught
// ---------------------------------------------------------------------------

test('freeze-gate rejects mutated vector (gate has teeth)', async () => {
  if (!fs.existsSync(VECTORS_PATH)) {
    // Skip gracefully if oracle is absent (fresh checkout pre-write)
    return
  }

  const original = fs.readFileSync(VECTORS_PATH, 'utf-8')
  const parsed = JSON.parse(original) as {
    vectors: Array<{ name: string; canonical_hex?: string; [k: string]: unknown }>
  }

  // Find first non-malformed vector with a canonical_hex to mutate
  const target = parsed.vectors.find((v) => typeof v.canonical_hex === 'string')
  if (!target || typeof target.canonical_hex !== 'string') {
    throw new Error('No non-malformed vector found to mutate — test is invalid')
  }

  // Flip one nibble at position 4 (after the magic+version bytes)
  const original_hex = target.canonical_hex
  const mutated_hex =
    original_hex.slice(0, 4) +
    // XOR the nibble at position 4 with 0x1 to ensure it changes
    (parseInt(original_hex[4]!, 16) ^ 1).toString(16) +
    original_hex.slice(5)
  target.canonical_hex = mutated_hex

  const mutatedPath = path.resolve(_dirname, '../vectors/v4-codec-mutated-test.json')
  fs.writeFileSync(mutatedPath, JSON.stringify(parsed, null, 2))

  try {
    // Temporarily point the checker at the mutated file by monkey-patching
    // the environment. We do this by writing the mutated file over the real
    // path, running the check, then restoring.
    fs.writeFileSync(VECTORS_PATH, JSON.stringify(parsed, null, 2))
    const mismatches = await runFreezeCheck()
    expect(mismatches.length).toBeGreaterThan(0)
    expect(mismatches.some((m) => m.includes('CANONICAL_MISMATCH'))).toBe(true)
  } finally {
    // Always restore the original file
    fs.writeFileSync(VECTORS_PATH, original)
    fs.rmSync(mutatedPath, { force: true })
  }
})
