import { defineConfig, configDefaults } from 'vitest/config'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  test: {
    environment: 'node',
    // Exclude the generator wrapper from the default test run — regeneration
    // is an explicit manual step, not something that should run on every pnpm test.
    exclude: [...configDefaults.exclude, 'scripts/**'],
    coverage: {
      // enabled: true → `vitest run` always collects + gates coverage, so the
      // 80% threshold (Constitution X) is enforced by plain `pnpm -r test` in CI.
      enabled: true,
      include: ['src/**'],
      exclude: [
        'target/**',
        'pkg/**',
        'pkg-node/**',
        'pkg-web/**',
        'dist/**',
        'docs/**',
        'scripts/**',
        // index.node.ts is the Node-target entry (node export condition).
        // It is exercised by pnpm test:pack (pack-and-import smoke test), not
        // by vitest which runs under the bundler resolver path via vite-plugin-wasm.
        'src/index.node.ts',
      ],
      thresholds: {
        lines: 80,
        branches: 80,
        functions: 80,
        statements: 80,
      },
    },
  },
  resolve: {
    alias: {
      // brotli-wasm's ESM condition routes to index.web.js, which loads WASM via
      // fetch() — unavailable in the vitest Node env. The bare specifier resolved
      // through CJS conditions lands on index.node.js (synchronous). The
      // '/index.node.js' subpath is not in the package's exports map, so it must
      // be resolved as the bare specifier, not appended.
      'brotli-wasm': require.resolve('brotli-wasm'),
    },
  },
})
