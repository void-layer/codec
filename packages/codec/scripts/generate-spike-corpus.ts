/**
 * Brotli Spike Corpus Generator
 *
 * Generates 20+ synthetic invoice objects using the vl/app TS reference codec,
 * encodes each to TLV wire bytes (uncompressed), and writes one JSON file per
 * invoice to vectors/spike-corpus/.
 *
 * Usage (run from /Users/ignat/code/vl/app):
 *   npx tsx --tsconfig tsconfig.json \
 *     /Users/ignat/code/vl/codec/packages/codec/scripts/generate-spike-corpus.ts
 *
 * Each output JSON:
 *   { source, generated_at, bytes_hex, uncompressed_length, shape }
 *
 * spike_id: brotli-2026-05
 */

import { writeTlv, sortCanonical, writeVarInt, writeMantissa, writeQuantity } from '/Users/ignat/code/vl/app/src/shared/lib/tlv-codec'
import type { TlvRecord } from '/Users/ignat/code/vl/app/src/shared/lib/tlv-codec'
import type { Invoice } from '/Users/ignat/code/vl/app/src/shared/lib/invoice-types'
import { applyDict } from '/Users/ignat/code/vl/app/src/features/invoice-codec/lib/app-dict'
import { encodeChainId } from '/Users/ignat/code/vl/app/src/features/invoice-codec/lib/chain-dict'
import { TlvType, encodeCurrency, encodeTokenAddress } from '/Users/ignat/code/vl/app/src/features/invoice-codec/lib/tlv-map'
import { generateSalt, computeDomainSeparator } from '/Users/ignat/code/vl/app/src/features/invoice-codec/lib/security'
import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'

// __dirname unavailable in ESM — derive from import.meta.url
const _filename = fileURLToPath(import.meta.url)
const _dirname = path.dirname(_filename)

const CORPUS_DIR = path.resolve(_dirname, '../vectors/spike-corpus')
const NOW_UNIX = Math.floor(Date.now() / 1000)
const ONE_DAY = 86400

// ---- helpers (mirrors encode.ts without brotli/base64url) ------------------

function utf8(s: string): Uint8Array {
  return new TextEncoder().encode(s)
}

function addressToBytes(address: string): Uint8Array {
  const hex = address.startsWith('0x') ? address.slice(2) : address
  const bytes = new Uint8Array(20)
  for (let i = 0; i < 20; i++) bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16)
  return bytes
}

function uint32BE(value: number): Uint8Array {
  const b = new Uint8Array(4)
  b[0] = (value >>> 24) & 0xff; b[1] = (value >>> 16) & 0xff
  b[2] = (value >>> 8) & 0xff;  b[3] = value & 0xff
  return b
}

function varintBytes(value: number): Uint8Array {
  const buf: number[] = []; writeVarInt(buf, value); return new Uint8Array(buf)
}

function mantissaBytes(value: bigint): Uint8Array {
  const buf: number[] = []; writeMantissa(buf, value); return new Uint8Array(buf)
}

function packItems(items: Invoice['items']): Uint8Array {
  const buf: number[] = []
  writeVarInt(buf, items.length)
  for (const item of items) {
    const descBytes = applyDict(utf8(item.description))
    writeVarInt(buf, descBytes.length)
    for (let i = 0; i < descBytes.length; i++) buf.push(descBytes[i]!)
    writeQuantity(buf, item.quantity)
    writeMantissa(buf, BigInt(item.rate || '0'))
  }
  return new Uint8Array(buf)
}

/**
 * Encode invoice to raw TLV bytes (no compression, no base64url).
 * Mirrors encode.ts buildRecords logic exactly.
 */
function encodeToTlvBytes(invoice: Invoice, salt: Uint8Array): Uint8Array {
  const records: TlvRecord[] = []

  const chainBuf: number[] = []
  encodeChainId(chainBuf, invoice.networkId)
  records.push({ type: TlvType.CHAIN_ID, value: new Uint8Array(chainBuf) })
  records.push({ type: TlvType.ISSUED_AT, value: uint32BE(invoice.issuedAt) })
  records.push({ type: TlvType.DUE_AT, value: varintBytes(invoice.dueAt - invoice.issuedAt) })
  records.push({ type: TlvType.DECIMALS, value: new Uint8Array([invoice.decimals]) })
  records.push({ type: TlvType.FROM_WALLET, value: addressToBytes(invoice.from.walletAddress) })

  const currCode = encodeCurrency(invoice.currency)
  if (currCode !== null) {
    records.push({ type: TlvType.CURRENCY, value: new Uint8Array([0x00, currCode]) })
  } else {
    const rawCurr = utf8(invoice.currency)
    const val = new Uint8Array(1 + rawCurr.length)
    val[0] = 0x01; val.set(rawCurr, 1)
    records.push({ type: TlvType.CURRENCY, value: val })
  }

  records.push({ type: TlvType.ITEMS, value: packItems(invoice.items) })
  records.push({ type: TlvType.INVOICE_ID, value: utf8(invoice.invoiceId) })
  records.push({ type: TlvType.SALT, value: salt })
  records.push({ type: TlvType.FROM_NAME, value: applyDict(utf8(invoice.from.name)) })
  records.push({ type: TlvType.CLIENT_NAME, value: applyDict(utf8(invoice.client.name)) })

  if (invoice.notes) records.push({ type: TlvType.NOTES, value: applyDict(utf8(invoice.notes)) })
  if (invoice.from.email) records.push({ type: TlvType.FROM_EMAIL, value: applyDict(utf8(invoice.from.email)) })
  if (invoice.from.phone) records.push({ type: TlvType.FROM_PHONE, value: applyDict(utf8(invoice.from.phone)) })
  if (invoice.from.physicalAddress) records.push({ type: TlvType.FROM_ADDRESS, value: applyDict(utf8(invoice.from.physicalAddress)) })
  if (invoice.from.taxId) records.push({ type: TlvType.FROM_TAX_ID, value: applyDict(utf8(invoice.from.taxId)) })
  if (invoice.client.email) records.push({ type: TlvType.CLIENT_EMAIL, value: applyDict(utf8(invoice.client.email)) })
  if (invoice.client.phone) records.push({ type: TlvType.CLIENT_PHONE, value: applyDict(utf8(invoice.client.phone)) })
  if (invoice.client.physicalAddress) records.push({ type: TlvType.CLIENT_ADDRESS, value: applyDict(utf8(invoice.client.physicalAddress)) })
  if (invoice.client.taxId) records.push({ type: TlvType.CLIENT_TAX_ID, value: applyDict(utf8(invoice.client.taxId)) })

  if (invoice.tokenAddress) {
    const tokenEntry = encodeTokenAddress(invoice.tokenAddress, invoice.networkId)
    if (tokenEntry) {
      records.push({ type: TlvType.TOKEN_ADDRESS, value: new Uint8Array([0x00, tokenEntry.code]) })
    } else {
      const rawAddr = addressToBytes(invoice.tokenAddress)
      const val = new Uint8Array(1 + 20); val[0] = 0x01; val.set(rawAddr, 1)
      records.push({ type: TlvType.TOKEN_ADDRESS, value: val })
    }
  }

  if (invoice.client.walletAddress) {
    records.push({ type: TlvType.CLIENT_WALLET, value: addressToBytes(invoice.client.walletAddress) })
  }
  if (invoice.tax) records.push({ type: TlvType.TAX, value: utf8(invoice.tax) })
  if (invoice.discount) records.push({ type: TlvType.DISCOUNT, value: utf8(invoice.discount) })

  const total = BigInt(invoice.total ?? '0')
  records.push({ type: TlvType.TOTAL, value: mantissaBytes(total) })

  const sorted = sortCanonical(records)
  const domainSep = computeDomainSeparator(sorted)
  sorted.push({ type: TlvType.DOMAIN_SEPARATOR, value: domainSep })
  const finalRecords = sortCanonical(sorted)

  return writeTlv(finalRecords)
}

// ---- invoice fixtures -------------------------------------------------------

type Shape =
  | 'minimal-1item-evm'
  | 'medium-2items-evm-notes'
  | 'full-3items-evm-all-fields'
  | 'minimal-1item-eth-mainnet'
  | 'minimal-1item-polygon'
  | 'minimal-1item-base'
  | 'minimal-1item-optimism'
  | 'medium-2items-usdc-arb'
  | 'medium-2items-no-notes'
  | 'full-3items-client-wallet'
  | 'full-3items-tax-discount'
  | 'medium-2items-long-descriptions'
  | 'minimal-1item-raw-currency'
  | 'full-3items-all-optional-text'
  | 'minimal-1item-small-amount'
  | 'minimal-1item-large-amount'
  | 'medium-2items-fractional-qty'
  | 'full-3items-eip712-heavy'
  | 'medium-2items-long-invoiceid'
  | 'full-3items-both-emails'

interface CorpusEntry {
  source: 'synthetic-via-ts-codec'
  generated_at: string
  bytes_hex: string
  uncompressed_length: number
  shape: Shape
}

const SALT_FIXED = new Uint8Array(16).fill(0x42) // deterministic for audit

const FROM_ETH = '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' as const
const CLIENT_ETH = '0x70997970C51812dc3A010C7d01b50e0d17dc79C8' as const

function makeInvoice(overrides: Partial<Invoice> & Pick<Invoice, 'invoiceId' | 'items' | 'from' | 'client' | 'currency' | 'networkId' | 'decimals' | 'total'>): Invoice {
  return {
    issuedAt: NOW_UNIX,
    dueAt: NOW_UNIX + 30 * ONE_DAY,
    ...overrides,
  }
}

const fixtures: Array<{ shape: Shape; invoice: Invoice }> = [
  {
    shape: 'minimal-1item-evm',
    invoice: makeInvoice({
      invoiceId: 'INV-001',
      networkId: 42161, // Arbitrum
      currency: 'USDC',
      decimals: 6,
      total: '1250000000',
      from: { name: 'Alice', walletAddress: FROM_ETH },
      client: { name: 'Bob' },
      items: [{ description: 'Consulting', quantity: 1, rate: '1250000000' }],
    }),
  },
  {
    shape: 'medium-2items-evm-notes',
    invoice: makeInvoice({
      invoiceId: 'INV-002',
      networkId: 42161,
      currency: 'USDC',
      decimals: 6,
      total: '3500000000',
      notes: 'Net 30 payment terms. Thank you for your business.',
      from: { name: 'Alice Dev Studio', walletAddress: FROM_ETH, email: 'alice@example.com' },
      client: { name: 'Acme Corp' },
      items: [
        { description: 'Backend development', quantity: 20, rate: '150000000' },
        { description: 'Code review', quantity: 5, rate: '100000000' },
      ],
    }),
  },
  {
    shape: 'full-3items-evm-all-fields',
    invoice: makeInvoice({
      invoiceId: 'INV-003-FULL',
      networkId: 1, // Ethereum mainnet
      currency: 'USDC',
      decimals: 6,
      total: '5600000000',
      notes: 'Please include invoice number in payment reference. VAT registered business.',
      from: {
        name: 'Alice Dev Studio Ltd',
        walletAddress: FROM_ETH,
        email: 'billing@alicedev.io',
        phone: '+1-555-0100',
        physicalAddress: '123 Main St, San Francisco, CA 94105',
        taxId: 'US-TAX-123456',
      },
      client: {
        name: 'Acme Corporation',
        walletAddress: CLIENT_ETH,
        email: 'ap@acme.com',
        phone: '+1-555-0200',
        physicalAddress: '456 Corp Ave, New York, NY 10001',
        taxId: 'US-TAX-789012',
      },
      items: [
        { description: 'Smart contract audit', quantity: 1, rate: '3000000000' },
        { description: 'Frontend development', quantity: 16, rate: '150000000' },
        { description: 'Technical documentation', quantity: 8, rate: '100000000' },
      ],
    }),
  },
  {
    shape: 'minimal-1item-eth-mainnet',
    invoice: makeInvoice({
      invoiceId: 'INV-004',
      networkId: 1,
      currency: 'ETH',
      decimals: 18,
      total: '1000000000000000000',
      from: { name: 'Carol', walletAddress: FROM_ETH },
      client: { name: 'Dave' },
      items: [{ description: 'Design work', quantity: 1, rate: '1000000000000000000' }],
    }),
  },
  {
    shape: 'minimal-1item-polygon',
    invoice: makeInvoice({
      invoiceId: 'INV-005',
      networkId: 137,
      currency: 'USDC',
      decimals: 6,
      total: '500000000',
      from: { name: 'Eve', walletAddress: FROM_ETH },
      client: { name: 'Frank' },
      items: [{ description: 'Logo design', quantity: 1, rate: '500000000' }],
    }),
  },
  {
    shape: 'minimal-1item-base',
    invoice: makeInvoice({
      invoiceId: 'INV-006',
      networkId: 8453,
      currency: 'USDC',
      decimals: 6,
      total: '750000000',
      from: { name: 'Grace', walletAddress: FROM_ETH },
      client: { name: 'Henry' },
      items: [{ description: 'API integration', quantity: 1, rate: '750000000' }],
    }),
  },
  {
    shape: 'minimal-1item-optimism',
    invoice: makeInvoice({
      invoiceId: 'INV-007',
      networkId: 10,
      currency: 'USDC',
      decimals: 6,
      total: '200000000',
      from: { name: 'Iris', walletAddress: FROM_ETH },
      client: { name: 'Jack' },
      items: [{ description: 'Bug fix', quantity: 2, rate: '100000000' }],
    }),
  },
  {
    shape: 'medium-2items-usdc-arb',
    invoice: makeInvoice({
      invoiceId: 'INV-008',
      networkId: 42161,
      currency: 'USDC',
      decimals: 6,
      total: '2250000000',
      from: { name: 'Karl Blockchain', walletAddress: FROM_ETH },
      client: { name: 'Luna Protocol' },
      items: [
        { description: 'DeFi integration', quantity: 10, rate: '200000000' },
        { description: 'Testing & QA', quantity: 5, rate: '50000000' },
      ],
    }),
  },
  {
    shape: 'medium-2items-no-notes',
    invoice: makeInvoice({
      invoiceId: 'INV-009',
      networkId: 42161,
      currency: 'DAI',
      decimals: 18,
      total: '1800000000000000000000',
      from: { name: 'Mia Studio', walletAddress: FROM_ETH },
      client: { name: 'Nova Corp' },
      items: [
        { description: 'UI/UX design', quantity: 12, rate: '100000000000000000000' },
        { description: 'Design system', quantity: 6, rate: '100000000000000000000' },
      ],
    }),
  },
  {
    shape: 'full-3items-client-wallet',
    invoice: makeInvoice({
      invoiceId: 'INV-010',
      networkId: 1,
      currency: 'USDC',
      decimals: 6,
      total: '4500000000',
      from: { name: 'Oscar Dev', walletAddress: FROM_ETH, email: 'oscar@dev.io' },
      client: { name: 'Pam Finance', walletAddress: CLIENT_ETH, email: 'pam@finance.io' },
      items: [
        { description: 'Architecture review', quantity: 1, rate: '2000000000' },
        { description: 'Implementation', quantity: 20, rate: '100000000' },
        { description: 'Deployment support', quantity: 5, rate: '100000000' },
      ],
    }),
  },
  {
    shape: 'full-3items-tax-discount',
    invoice: makeInvoice({
      invoiceId: 'INV-011',
      networkId: 42161,
      currency: 'USDC',
      decimals: 6,
      total: '4720000000',
      tax: '10',
      discount: '5',
      from: { name: 'Quinn Agency', walletAddress: FROM_ETH },
      client: { name: 'Ross Industries' },
      items: [
        { description: 'Strategy consulting', quantity: 1, rate: '2000000000' },
        { description: 'Market research', quantity: 1, rate: '1500000000' },
        { description: 'Report writing', quantity: 1, rate: '500000000' },
      ],
    }),
  },
  {
    shape: 'medium-2items-long-descriptions',
    invoice: makeInvoice({
      invoiceId: 'INV-012',
      networkId: 42161,
      currency: 'USDC',
      decimals: 6,
      total: '6000000000',
      notes: 'Extended engagement for Q2 2026 product development sprint covering all milestones.',
      from: { name: 'Sam Engineering', walletAddress: FROM_ETH },
      client: { name: 'Terra Startup' },
      items: [
        {
          description: 'Full-stack web application development including backend API, database schema design, and React frontend',
          quantity: 1,
          rate: '4000000000',
        },
        {
          description: 'CI/CD pipeline setup, Docker containerization, AWS deployment, monitoring and alerting configuration',
          quantity: 1,
          rate: '2000000000',
        },
      ],
    }),
  },
  {
    shape: 'minimal-1item-raw-currency',
    invoice: makeInvoice({
      invoiceId: 'INV-013',
      networkId: 42161,
      currency: 'WBTC',
      decimals: 8,
      total: '1000000',
      from: { name: 'Uma Bitcoin', walletAddress: FROM_ETH },
      client: { name: 'Victor Fund' },
      items: [{ description: 'Bitcoin custody setup', quantity: 1, rate: '1000000' }],
    }),
  },
  {
    shape: 'full-3items-all-optional-text',
    invoice: makeInvoice({
      invoiceId: 'INV-014-LONG-ID-FOR-TESTING',
      networkId: 1,
      currency: 'USDC',
      decimals: 6,
      total: '9500000000',
      notes: 'Payment due within 30 days. Late fees of 1.5% per month apply after due date.',
      from: {
        name: 'Wendy Tech Solutions',
        walletAddress: FROM_ETH,
        email: 'wendy@techsolutions.io',
        phone: '+44-20-7946-0958',
        physicalAddress: '10 Downing St, London, UK SW1A 2AA',
        taxId: 'GB-VAT-123456789',
      },
      client: {
        name: 'Xavier Enterprises',
        walletAddress: CLIENT_ETH,
        email: 'xavier@enterprises.com',
        phone: '+1-212-555-0150',
        physicalAddress: '1 World Trade Center, New York, NY 10007',
        taxId: 'US-EIN-12-3456789',
      },
      items: [
        { description: 'Enterprise software license', quantity: 1, rate: '5000000000' },
        { description: 'Implementation & onboarding', quantity: 1, rate: '3000000000' },
        { description: 'First year support contract', quantity: 1, rate: '1500000000' },
      ],
    }),
  },
  {
    shape: 'minimal-1item-small-amount',
    invoice: makeInvoice({
      invoiceId: 'INV-015',
      networkId: 137,
      currency: 'USDC',
      decimals: 6,
      total: '5000000',
      from: { name: 'Yara', walletAddress: FROM_ETH },
      client: { name: 'Zoe' },
      items: [{ description: 'Translation', quantity: 1, rate: '5000000' }],
    }),
  },
  {
    shape: 'minimal-1item-large-amount',
    invoice: makeInvoice({
      invoiceId: 'INV-016',
      networkId: 1,
      currency: 'USDC',
      decimals: 6,
      total: '500000000000',
      from: { name: 'Atlas Capital', walletAddress: FROM_ETH },
      client: { name: 'Nexus DAO' },
      items: [{ description: 'Protocol acquisition advisory', quantity: 1, rate: '500000000000' }],
    }),
  },
  {
    shape: 'medium-2items-fractional-qty',
    invoice: makeInvoice({
      invoiceId: 'INV-017',
      networkId: 8453,
      currency: 'USDC',
      decimals: 6,
      total: '875000000',
      from: { name: 'Blake Design', walletAddress: FROM_ETH },
      client: { name: 'Cyan Media' },
      items: [
        { description: 'Brand identity design', quantity: 1.5, rate: '400000000' },
        { description: 'Social media assets', quantity: 2.5, rate: '70000000' },
      ],
    }),
  },
  {
    shape: 'full-3items-eip712-heavy',
    invoice: makeInvoice({
      invoiceId: 'INV-018-EIP712',
      networkId: 42161,
      currency: 'USDC',
      decimals: 6,
      total: '7300000000',
      notes: 'EIP-712 signed invoice for on-chain payment verification.',
      from: {
        name: 'Drew Protocol Labs',
        walletAddress: FROM_ETH,
        email: 'drew@protocollabs.xyz',
      },
      client: {
        name: 'Ember DAO Treasury',
        walletAddress: CLIENT_ETH,
        email: 'treasury@emberdao.xyz',
      },
      items: [
        { description: 'Protocol design & tokenomics', quantity: 1, rate: '3000000000' },
        { description: 'Smart contract development', quantity: 1, rate: '3000000000' },
        { description: 'Security audit coordination', quantity: 1, rate: '1300000000' },
      ],
    }),
  },
  {
    shape: 'medium-2items-long-invoiceid',
    invoice: makeInvoice({
      invoiceId: 'INVOICE-2026-Q2-DEVELOPMENT-SPRINT-042',
      networkId: 10,
      currency: 'USDC',
      decimals: 6,
      total: '2600000000',
      from: { name: 'Faye Studio', walletAddress: FROM_ETH },
      client: { name: 'Gale Ventures' },
      items: [
        { description: 'Sprint planning & execution', quantity: 1, rate: '2000000000' },
        { description: 'Retrospective & documentation', quantity: 1, rate: '600000000' },
      ],
    }),
  },
  {
    shape: 'full-3items-both-emails',
    invoice: makeInvoice({
      invoiceId: 'INV-020',
      networkId: 42161,
      currency: 'USDC',
      decimals: 6,
      total: '3350000000',
      from: {
        name: 'Hank Consulting',
        walletAddress: FROM_ETH,
        email: 'hank@consulting.dev',
        taxId: 'DE-USt-123456789',
      },
      client: {
        name: 'Ivy Solutions GmbH',
        walletAddress: CLIENT_ETH,
        email: 'billing@ivy-solutions.de',
        taxId: 'DE-USt-987654321',
      },
      items: [
        { description: 'Web3 integration consulting', quantity: 15, rate: '150000000' },
        { description: 'Technical due diligence', quantity: 8, rate: '100000000' },
        { description: 'Workshop facilitation', quantity: 3, rate: '200000000' },
      ],
    }),
  },
]

// ---- main ------------------------------------------------------------------

async function main(): Promise<void> {
  fs.mkdirSync(CORPUS_DIR, { recursive: true })

  let count = 0
  const summary: Array<{ shape: Shape; uncompressed_length: number; file: string }> = []

  for (const { shape, invoice } of fixtures) {
    const tlvBytes = encodeToTlvBytes(invoice, SALT_FIXED)
    const entry: CorpusEntry = {
      source: 'synthetic-via-ts-codec',
      generated_at: new Date().toISOString(),
      bytes_hex: Buffer.from(tlvBytes).toString('hex'),
      uncompressed_length: tlvBytes.length,
      shape,
    }

    const filename = `${String(count + 1).padStart(2, '0')}-${shape}.json`
    const filepath = path.join(CORPUS_DIR, filename)
    fs.writeFileSync(filepath, JSON.stringify(entry, null, 2) + '\n')
    summary.push({ shape, uncompressed_length: tlvBytes.length, file: filename })
    count++
  }

  console.log(`\nGenerated ${count} corpus entries to ${CORPUS_DIR}\n`)
  console.log('Shape                               | Uncompressed (B)')
  console.log('------------------------------------|------------------')
  for (const s of summary) {
    console.log(`${s.shape.padEnd(35)} | ${s.uncompressed_length}`)
  }

  const sizes = summary.map((s) => s.uncompressed_length)
  const min = Math.min(...sizes)
  const max = Math.max(...sizes)
  const median = sizes.sort((a, b) => a - b)[Math.floor(sizes.length / 2)]!
  console.log(`\nMin: ${min} B  Max: ${max} B  Median: ${median} B`)
}

main().catch((err) => {
  console.error('Corpus generation failed:', err)
  process.exit(1)
})
