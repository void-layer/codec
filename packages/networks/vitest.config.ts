import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    environment: 'node',
    coverage: {
      // enabled: true → coverage is collected + gated on every `vitest run`,
      // so the 80% threshold is enforced by plain `pnpm -r test` in CI.
      enabled: true,
      include: ['src/**/*.ts'],
      exclude: ['src/**/*.test.ts', 'src/**/*.d.ts', '**/node_modules/**', '**/dist/**'],
      thresholds: {
        lines: 80,
        branches: 80,
        functions: 80,
        statements: 80,
      },
    },
  },
})
