/**
 * Vitest wrapper — runs the parametric corpus generator as a test so vitest's
 * module resolver (brotli-wasm alias, wasm plugin) are active.
 *
 * Usage: pnpm -C packages/codec exec vitest run scripts/run-generate-corpus.test.ts \
 *          --config scripts/generate-vectors.config.ts
 */
import { test } from 'vitest'

test('generate parametric corpus', async () => {
  const mod = await import('./generate-corpus.js')
  void mod
}, 120_000)
