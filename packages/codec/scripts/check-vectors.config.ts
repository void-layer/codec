/**
 * Vitest config for the freeze-gate vector checker.
 * Used by: pnpm check-vectors (CI freeze-gate)
 *
 * Includes the run-check-vectors.test.ts wrapper which:
 *   - Runs the positive gate (codec output == frozen oracle)
 *   - Runs the negative test (mutated vector is caught)
 *
 * Coverage is disabled — this is a gate script, not a coverage target.
 */
import { defineConfig } from 'vitest/config'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  test: {
    environment: 'node',
    include: ['scripts/run-check-vectors.test.ts', 'scripts/run-freeze-write.test.ts'],
    coverage: { enabled: false },
  },
  resolve: {
    alias: {
      'brotli-wasm': require.resolve('brotli-wasm'),
    },
  },
})
