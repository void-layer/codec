/**
 * Vitest config for vl/app provenance verification.
 * Adds @/ → voidpay/src alias so vl/app's encode.ts can be imported directly.
 * brotli-wasm is aliased to Node variant (same as main vitest.config.ts).
 *
 * LOCAL-ONLY provenance audit — NOT a CI gate.
 * This test requires a local checkout of the vl/app (voidpay) repo.
 * Set VOIDPAY_SRC to the absolute path of voidpay/src before running:
 *   VOIDPAY_SRC=/path/to/voidpay/src pnpm exec vitest run scripts/vlapp-provenance.test.ts \
 *     --config scripts/vlapp-provenance.config.ts
 */
import { defineConfig } from 'vitest/config'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { createRequire } from 'node:module'
import * as path from 'node:path'

const require = createRequire(import.meta.url)

const VOIDPAY_SRC = path.resolve(process.env['VOIDPAY_SRC'] ?? '/Users/ignat/code/voidpay/src')

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
