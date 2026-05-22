import { defineConfig } from 'vitest/config'

// @void-layer/types is a type-only package — every export is a `type`/`interface`
// that compiles to zero runtime JS, so line/branch coverage is structurally N/A.
// The `expectTypeOf` suite is the gate; no coverage thresholds here by design.
export default defineConfig({
  test: {
    environment: 'node',
  },
})
