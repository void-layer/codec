/**
 * Vitest config for explicit vector generation only.
 * Used by: pnpm generate-vectors
 * Overrides the main vitest.config.ts exclude so scripts/** is included.
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
    include: ['scripts/run-generate-vectors.test.ts'],
  },
  resolve: {
    alias: {
      'brotli-wasm': require.resolve('brotli-wasm'),
    },
  },
})
