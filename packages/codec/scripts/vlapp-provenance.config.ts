/**
 * Vitest config for vl/app provenance verification.
 * Adds @/ → voidpay/src alias so vl/app's encode.ts can be imported directly.
 * brotli-wasm is aliased to Node variant (same as main vitest.config.ts).
 */
import { defineConfig } from 'vitest/config'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { createRequire } from 'node:module'
import * as path from 'node:path'

const require = createRequire(import.meta.url)

const VOIDPAY_SRC = path.resolve('/Users/ignat/code/voidpay/src')

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  test: {
    environment: 'node',
    include: ['scripts/vlapp-provenance.test.ts'],
    coverage: { enabled: false },
  },
  resolve: {
    alias: {
      'brotli-wasm': require.resolve('brotli-wasm'),
      '@': VOIDPAY_SRC,
    },
  },
})
