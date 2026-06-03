/**
 * One-shot oracle freeze writer.
 *
 * Rewrites vectors/v4-codec.json with provenance headers and the current
 * codec's derived bytes. Run ONLY when making a deliberate, reviewed update
 * to the frozen oracle (e.g. adding a new TLV type, new dict code).
 *
 * Usage (from packages/codec root):
 *   FREEZE_GATE_WRITE=1 pnpm exec vitest run scripts/run-freeze-write.test.ts --config scripts/check-vectors.config.ts
 *
 * After running, verify with:
 *   pnpm check-vectors
 *
 * Requires Kai review before merging any oracle update.
 */
import { test } from 'vitest'
import { runWrite } from './check-vectors.js'

test(
  'freeze-write: stamp v4-codec.json with provenance headers',
  async () => {
    if (!process.env['FREEZE_GATE_WRITE']) {
      console.log(
        '[freeze-write] Skipped — set FREEZE_GATE_WRITE=1 to actually write the oracle',
      )
      return
    }
    await runWrite()
    console.log('[freeze-write] Oracle written successfully')
  },
  120_000,
)
