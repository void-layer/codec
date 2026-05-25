/**
 * T3 — Cross-impl parity: Rust WASM encoder vs TS reference encoder (vl/app).
 *
 * Runs each non-malformed golden vector through both encoders and asserts
 * byte-identical canonical output.  Fails loud at the first mismatch with
 * the full invoice JSON + both hex outputs so the diff is immediately visible.
 *
 * Usage (local, after `pnpm build` in packages/codec):
 *   VL_APP_PATH=/path/to/vl/app pnpm run test:ts-rust-parity
 *
 * Usage (CI, via ts-rust-parity job):
 *   vl-app is checked out at ./vl-app relative to the codec repo root.
 *   VL_APP_PATH is set by the CI job step.
 */

import { createRequire } from 'module'
import { readFileSync } from 'fs'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const repoRoot = resolve(__dirname, '../../..')

// ---------------------------------------------------------------------------
// Load vl/app encoder
// ---------------------------------------------------------------------------

const vlAppPath = process.env['VL_APP_PATH']
if (!vlAppPath) {
  console.error('ERROR: VL_APP_PATH env var is required.')
  console.error('  Local: VL_APP_PATH=/path/to/vl/app pnpm run test:ts-rust-parity')
  process.exit(1)
}

// Use dynamic import for ESM compatibility
const encodeModulePath = resolve(vlAppPath, 'src/features/invoice-codec/lib/encode.ts')

// vl/app uses TypeScript source directly — we need tsx to run this script.
// The encodeInvoiceCanonical from vl/app takes the same Invoice shape.
let encodeInvoiceCanonical_TS: (invoice: unknown) => Uint8Array

try {
  const mod = await import(encodeModulePath)
  encodeInvoiceCanonical_TS = mod.encodeInvoiceCanonical
  if (typeof encodeInvoiceCanonical_TS !== 'function') {
    throw new Error('encodeInvoiceCanonical is not a function in vl/app encode.ts')
  }
} catch (err) {
  console.error(`ERROR: Failed to import vl/app encoder from ${encodeModulePath}`)
  console.error(err)
  process.exit(1)
}

// ---------------------------------------------------------------------------
// Load Rust WASM encoder (built pkg)
// ---------------------------------------------------------------------------

const pkgPath = resolve(__dirname, '../pkg/void_layer_codec.js')
let encodeInvoiceCanonical_Rust: (invoice: unknown) => Uint8Array

try {
  const mod = await import(pkgPath)
  encodeInvoiceCanonical_Rust = mod.encodeInvoiceCanonical
  if (typeof encodeInvoiceCanonical_Rust !== 'function') {
    throw new Error('encodeInvoiceCanonical is not a function in Rust WASM pkg')
  }
} catch (err) {
  console.error(`ERROR: Failed to import Rust WASM encoder from ${pkgPath}`)
  console.error('  Run: pnpm -C packages/codec build')
  console.error(err)
  process.exit(1)
}

// ---------------------------------------------------------------------------
// Load golden vectors
// ---------------------------------------------------------------------------

const vectorsPath = resolve(__dirname, '../vectors/v4-codec.json')
const { vectors } = JSON.parse(readFileSync(vectorsPath, 'utf-8')) as {
  vectors: Array<{
    name: string
    canonical_hex?: string
    decoded?: unknown
    roundtrip?: boolean
    expected_error?: string
  }>
}

// ---------------------------------------------------------------------------
// Run parity check
// ---------------------------------------------------------------------------

let passed = 0
let skipped = 0
let failed = 0

for (const vec of vectors) {
  // Skip malformed vectors (no decoded invoice to encode)
  if (vec.expected_error || !vec.decoded || !vec.canonical_hex) {
    skipped++
    continue
  }

  const invoice = vec.decoded

  let rustHex: string
  let tsHex: string

  try {
    const rustBytes = encodeInvoiceCanonical_Rust(invoice)
    rustHex = Buffer.from(rustBytes).toString('hex')
  } catch (err) {
    console.error(`FAIL [${vec.name}]: Rust encoder threw:`, err)
    failed++
    continue
  }

  try {
    const tsBytes = encodeInvoiceCanonical_TS(invoice)
    tsHex = Buffer.from(tsBytes).toString('hex')
  } catch (err) {
    console.error(`FAIL [${vec.name}]: TS encoder threw:`, err)
    failed++
    continue
  }

  if (rustHex !== tsHex) {
    console.error(`FAIL [${vec.name}]: encoder output mismatch`)
    console.error('  Invoice:', JSON.stringify(invoice, null, 2))
    console.error('  Rust:   ', rustHex)
    console.error('  TS:     ', tsHex)
    console.error('  Golden: ', vec.canonical_hex)
    failed++
    continue
  }

  // Also verify both match the golden vector
  if (rustHex !== vec.canonical_hex) {
    console.error(`FAIL [${vec.name}]: both encoders agree but differ from golden vector`)
    console.error('  Encoded:', rustHex)
    console.error('  Golden: ', vec.canonical_hex)
    failed++
    continue
  }

  passed++
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

console.log(`ts-rust-parity: ${passed} passed, ${skipped} skipped (malformed), ${failed} failed`)

if (failed > 0) {
  console.error(`ERROR: ${failed} parity failure(s) — Rust WASM and TS reference encoders diverge`)
  process.exit(1)
}

console.log('OK: Rust WASM and TS reference encoders produce identical canonical bytes for all vectors.')
