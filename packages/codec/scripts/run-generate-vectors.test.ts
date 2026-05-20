/**
 * Vitest wrapper — runs the golden vector generator as a test so vitest's
 * module resolver (brotli-wasm alias, wasm plugin) are active.
 *
 * Usage: pnpm -C packages/codec exec vitest run scripts/run-generate-vectors.test.ts
 */
import { test } from 'vitest'

test('generate golden vectors', async () => {
  // Dynamic import picks up vitest's alias resolution for brotli-wasm
  const mod = await import('./generate-vectors.js')
  // The module calls main() at module level via the bottom invocation.
  // If we import it directly it runs. But generate-vectors.ts exports nothing
  // and has a top-level main() call — it already ran on import.
  void mod
}, 120_000)
