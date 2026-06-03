#!/usr/bin/env node
/**
 * Pack-and-import smoke test — B2 acceptance gate.
 *
 * Proves the *published* package imports cleanly in plain Node ESM
 * (no bundler, no vitest resolver magic).
 *
 * Steps:
 *   1. pnpm pack → tarball
 *   2. Install tarball into a clean tmp dir (type:module) via pnpm
 *   3. node --input-type=module runs the import + encode→decode round-trip
 *   4. Assert round-trip equality; exit 1 on any failure
 *
 * Run: node scripts/test-pack.mjs  (from packages/codec/)
 */

import { execSync, execFileSync } from 'node:child_process'
import { mkdtempSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'

const __dirname = fileURLToPath(new URL('.', import.meta.url))
const PACKAGE_DIR = resolve(__dirname, '..')

console.log('=== B2 pack-and-import smoke test ===')
console.log(`Package dir: ${PACKAGE_DIR}`)

// 1. Build + pack (pnpm rewrites workspace:^ → ^x.y.z)
console.log('\n[1] pnpm pack ...')
execSync('pnpm pack', {
  cwd: PACKAGE_DIR,
  stdio: 'inherit',
})
const tarballName = 'void-layer-codec-0.1.0.tgz'
const tarballPath = join(PACKAGE_DIR, tarballName)
console.log(`    tarball: ${tarballName}`)

// 2. Set up clean tmp consumer dir
const tmpDir = mkdtempSync(join(tmpdir(), 'vl-codec-smoke-'))
console.log(`\n[2] Installing tarball into ${tmpDir} ...`)

// @void-layer/types is import-type-only in the codec source; not needed at runtime.
// brotli-wasm is a peerDep of @void-layer/codec that the consumer must provide.
writeFileSync(
  join(tmpDir, 'package.json'),
  JSON.stringify({
    name: 'smoke-consumer',
    version: '1.0.0',
    type: 'module',
    dependencies: {
      '@void-layer/codec': `file:${tarballPath}`,
      'brotli-wasm': '3.0.1',
    },
  }),
)

execSync('pnpm install --no-frozen-lockfile 2>&1', {
  cwd: tmpDir,
  stdio: 'inherit',
})

// 3. Run the round-trip import in plain Node ESM
console.log('\n[3] Running Node ESM import + encode→decode round-trip ...')

const smokeScript = /* js */ `
import { encodeInvoiceWire, decodeInvoiceWire, encodeInvoiceCanonical, decodeInvoiceCanonical } from '@void-layer/codec';

const invoice = {
  invoice_id: 'B2-SMOKE',
  issued_at: 1_700_000_000,
  due_at: 1_700_086_400,
  network_id: 8453,
  currency: 'USDC',
  decimals: 6,
  from: { name: 'Alice', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' },
  client: { name: 'Bob' },
  items: [{ description: 'Smoke test', quantity: 1.0, rate: '1000000' }],
  total: '1000000',
  salt: 'deadbeefdeadbeefdeadbeefdeadbeef',
};

// Canonical round-trip
const canonicalBytes = encodeInvoiceCanonical(invoice);
if (!(canonicalBytes instanceof Uint8Array)) throw new Error('encodeInvoiceCanonical did not return Uint8Array');
if (canonicalBytes[0] !== 0x56) throw new Error('Missing MAGIC byte 0x56');

const canonicalDecoded = decodeInvoiceCanonical(canonicalBytes);
if (canonicalDecoded.invoice_id !== 'B2-SMOKE') throw new Error('canonical round-trip invoice_id mismatch: ' + canonicalDecoded.invoice_id);
if (canonicalDecoded.currency !== 'USDC') throw new Error('canonical round-trip currency mismatch');
console.log('  canonical round-trip: OK');

// Wire round-trip
const wireBytes = await encodeInvoiceWire(invoice);
if (!(wireBytes instanceof Uint8Array)) throw new Error('encodeInvoiceWire did not return Uint8Array');

const wireDecoded = await decodeInvoiceWire(wireBytes);
if (wireDecoded.invoice_id !== 'B2-SMOKE') throw new Error('wire round-trip invoice_id mismatch: ' + wireDecoded.invoice_id);
if (wireDecoded.currency !== 'USDC') throw new Error('wire round-trip currency mismatch');
console.log('  wire round-trip: OK');

console.log('');
console.log('B2 SMOKE PASS');
`

let smokePassed = false
try {
  execFileSync(process.execPath, ['--input-type=module'], {
    cwd: tmpDir,
    input: smokeScript,
    stdio: ['pipe', 'inherit', 'inherit'],
    env: { ...process.env, NODE_OPTIONS: '' },
  })
  smokePassed = true
} catch {
  // execFileSync throws on non-zero exit; the child's stderr is already
  // forwarded via stdio:'inherit' above, so no additional logging needed.
  console.error('\nB2 SMOKE FAIL — Node ESM import failed (see above)')
} finally {
  // Always clean up tarball; ignore errors (e.g. already deleted)
  try {
    execSync(`rm -f "${tarballPath}"`)
  } catch {
    // intentional: cleanup failure must not mask smoke failure
  }
}

// Clean up tmp dir
try {
  execSync(`rm -rf "${tmpDir}"`)
} catch {
  // intentional: cleanup failure must not mask smoke failure
}

if (!smokePassed) process.exit(1)

console.log('\n=== B2 DONE ===')
