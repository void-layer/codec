/**
 * Parametric corpus generator — @void-layer/codec corpus.json
 *
 * Tier-2 regenerable corpus: curated combinatorial sampling across 4 dimensions:
 *   chain      : {1, 8453, 42161, 10, 137}
 *   fill_level : {minimal, medium, full}
 *   language   : {ascii, cyrillic, cjk, emoji, rtl, high-entropy}
 *   amount_edge: {zero, one, typical, large, u256-max}
 *
 * Target: 60-120 entries via deliberate sampling, not full cross-product (450).
 * DETERMINISM: fixed timestamps, fixed salt, seeded PRNG — running twice
 * must produce byte-identical corpus.json.
 *
 * Run (from packages/codec root):
 *   pnpm -C packages/codec exec vite-node scripts/generate-corpus.ts
 *
 * Or via the vitest wrapper:
 *   pnpm -C packages/codec exec vitest run scripts/run-generate-corpus.test.ts \
 *     --config scripts/generate-vectors.config.ts
 */

import * as fs from 'node:fs'
import * as path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  encodeInvoiceCanonical,
  decodeInvoiceCanonical,
} from '../pkg-node/void_layer_codec.js'
import brotliWasmInit from 'brotli-wasm'

const _filename = fileURLToPath(import.meta.url)
const _dirname = path.dirname(_filename)
const VECTORS_DIR = path.resolve(_dirname, '../vectors')
const OUT_PATH = path.join(VECTORS_DIR, 'corpus.json')

const COMPRESSED_FLAG = 0x80

// ---------------------------------------------------------------------------
// Fixed constants — MUST NOT change (determinism)
// ---------------------------------------------------------------------------

const ISSUED_AT = 1_700_000_000
const DUE_AT = 1_700_086_400
const FROM_WALLET = '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045'
const CLIENT_WALLET = '0x70997970C51812dc3A010C7d01b50e0d17dc79C8'
const SALT = 'deadbeefdeadbeefdeadbeefdeadbeef'

const U256_MAX =
  '115792089237316195423570985008687907853269984665640564039457584007913129639935'

// ---------------------------------------------------------------------------
// Seeded PRNG — xorshift32, deterministic, NOT crypto-random
// ---------------------------------------------------------------------------

function xorshift32(seed: number): () => number {
  let s = seed >>> 0
  return function next(): number {
    s ^= s << 13
    s ^= s >>> 17
    s ^= s << 5
    return (s >>> 0) / 0x100000000
  }
}

/** Generate a deterministic "high-entropy" string of given byte length.
 *  Uses xorshift32 seeded by (index * 0x9e3779b9) to ensure each entry
 *  gets a unique but reproducible sequence. */
function highEntropyString(byteLen: number, seed: number): string {
  const rng = xorshift32(seed * 0x9e3779b9 + 1)
  // Printable ASCII range 0x21-0x7e (94 chars) — high entropy, incompressible
  const chars = '!"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~'
  const arr: string[] = []
  for (let i = 0; i < byteLen; i++) {
    arr.push(chars[Math.floor(rng() * chars.length)]!)
  }
  return arr.join('')
}

// ---------------------------------------------------------------------------
// Language text fixtures
// ---------------------------------------------------------------------------

type Language = 'ascii' | 'cyrillic' | 'cjk' | 'emoji' | 'rtl' | 'high-entropy'

interface LangTexts {
  fromName: string
  clientName: string
  description: string
  notes: string
}

// CJK notes padded to exactly 280 chars (Unicode code points, not bytes)
// Each CJK char = 3 UTF-8 bytes; 280 chars = up to 840 bytes.
const CJK_280_CHARS =
  '软件开发咨询服务，包括架构设计、代码审查、部署支持和事故响应，按月计费。本发票适用于2026年第二季度服务合同。感谢您的信任与合作。请在到期日前完成付款，否则将收取逾期费用。如有疑问请联系我们的财务部门。服务范围涵盖前端开发、后端API、数据库设计及持续集成。我们致力于提供高质量的技术解决方案以满足您的业务需求。'

const LANG_TEXTS: Record<Language, (seed: number) => LangTexts> = {
  ascii: (_seed) => ({
    fromName: 'Alice Developer',
    clientName: 'Bob Client',
    description: 'Software consulting services',
    notes: 'Payment due within 30 days. Thank you for your business.',
  }),
  cyrillic: (_seed) => ({
    fromName: 'Алиса Разработчик',
    clientName: 'Боб Клиент',
    description: 'Консультационные услуги по разработке',
    notes: 'Оплата в течение 30 дней. Спасибо за сотрудничество.',
  }),
  cjk: (_seed) => ({
    fromName: 'Alice',
    clientName: '鲍勃客户',
    description: '软件开发咨询服务',
    notes: '請在30天內付款。感謝您的支持與合作。',
  }),
  emoji: (_seed) => ({
    fromName: 'Alice 🚀',
    clientName: 'Bob 💎',
    description: 'Premium consulting ✅',
    notes: '✅ Payment confirmed 🎉 Thank you! 💎 Invoice #️⃣',
  }),
  rtl: (_seed) => ({
    fromName: 'أليس المطور',
    clientName: 'بوب العميل',
    description: 'خدمات استشارية للبرمجيات',
    notes: 'يرجى الدفع خلال 30 يوماً. شكراً لتعاملكم معنا.',
  }),
  'high-entropy': (seed) => ({
    fromName: highEntropyString(20, seed),
    clientName: highEntropyString(15, seed + 1),
    description: highEntropyString(40, seed + 2),
    notes: highEntropyString(60, seed + 3),
  }),
}

// ---------------------------------------------------------------------------
// Amount edges
// ---------------------------------------------------------------------------

type AmountEdge = 'zero' | 'one' | 'typical' | 'large' | 'u256-max'

function amountForEdge(edge: AmountEdge): string {
  switch (edge) {
    case 'zero':    return '0'
    case 'one':     return '1'
    case 'typical': return '1000000'
    case 'large':   return '1000000000000000000'  // 1e18 (1 ETH or 1M USDC with 18 decimals)
    case 'u256-max': return U256_MAX
  }
}

// ---------------------------------------------------------------------------
// Fill levels
// ---------------------------------------------------------------------------

type FillLevel = 'minimal' | 'medium' | 'full'

interface InvoiceShape {
  fill: FillLevel
  lang: Language
  chain: number
  amountEdge: AmountEdge
}

function buildInvoice(shape: InvoiceShape, seed: number): Record<string, unknown> {
  const texts = LANG_TEXTS[shape.lang](seed)
  const amount = amountForEdge(shape.amountEdge)

  const base: Record<string, unknown> = {
    invoice_id: `CORP-${seed.toString(36).toUpperCase().padStart(6, '0')}`,
    issued_at: ISSUED_AT,
    due_at: DUE_AT,
    network_id: shape.chain,
    currency: shape.chain === 137 ? 'MATIC' : 'USDC',
    decimals: shape.chain === 137 ? 18 : 6,
    from: { name: texts.fromName, wallet_address: FROM_WALLET },
    client: { name: texts.clientName },
    items: [{ description: texts.description, quantity: 1.0, rate: amount }],
    total: amount,
    salt: SALT,
  }

  if (shape.fill === 'medium' || shape.fill === 'full') {
    base['notes'] = texts.notes
    // second item
    const secondAmt = shape.amountEdge === 'zero' ? '0' : '500000'
    ;(base['items'] as unknown[]).push({
      description: texts.description + ' (phase 2)',
      quantity: 2.0,
      rate: secondAmt,
    })
  }

  if (shape.fill === 'full') {
    base['from'] = {
      name: texts.fromName,
      wallet_address: FROM_WALLET,
      email: 'alice@example.com',
    }
    base['client'] = {
      name: texts.clientName,
      wallet_address: CLIENT_WALLET,
      email: 'bob@example.com',
    }
    // third item
    const thirdAmt = shape.amountEdge === 'zero' ? '0' : '250000'
    ;(base['items'] as unknown[]).push({
      description: texts.description + ' (phase 3)',
      quantity: 0.5,
      rate: thirdAmt,
    })
    base['tax'] = '10'
    base['discount'] = '5'
  }

  return base
}

// ---------------------------------------------------------------------------
// Wire encode/decode (mirrors generate-vectors.ts exactly)
// ---------------------------------------------------------------------------

async function wireEncode(invoice: unknown): Promise<Uint8Array> {
  const brotli = await brotliWasmInit
  const canonical: Uint8Array = encodeInvoiceCanonical(invoice)
  if (canonical.length < 3) return canonical
  const body = canonical.slice(2)
  const compressed = brotli.compress(body, { quality: 11 })
  if (compressed.length >= body.length) return canonical
  const result = new Uint8Array(2 + compressed.length)
  result[0] = canonical[0]!
  result[1] = canonical[1]! | COMPRESSED_FLAG
  result.set(compressed, 2)
  return result
}

function toHex(bytes: Uint8Array): string {
  return Buffer.from(bytes).toString('hex')
}

function isCompressed(wireHex: string): boolean {
  if (wireHex.length < 4) return false
  return (parseInt(wireHex.slice(2, 4), 16) & COMPRESSED_FLAG) !== 0
}

// ---------------------------------------------------------------------------
// Corpus sampling plan — curated, not full cross-product
//
// Strategy:
//   A) All 5 chains × minimal × ascii × typical  (5)
//   B) All 5 chains × medium × ascii × typical   (5)
//   C) All 5 chains × full   × ascii × typical   (5)
//   D) Chain=1 × all 3 fills × all 6 languages × typical  (18)
//   E) Chain=1 × minimal × ascii × all 5 amount-edges     (5)
//   F) Chain=8453 × medium × {cyrillic,cjk,emoji,rtl,high-entropy} × typical  (5)
//   G) Chain=42161 × full × {ascii,cyrillic,cjk} × {large,u256-max}  (6)
//   H) Chain=137 × medium × {ascii,cjk} × typical  (2)
//   I) Chain=10 × full × {emoji,rtl} × large       (2)
//   J) CJK notes at 280-char boundary               (1, special)
//
// Total: 5+5+5+18+5+5+6+2+2+1 = 54 entries
// ---------------------------------------------------------------------------

interface CorpusEntry {
  name: string
  shape: FillLevel
  language: Language
  chain: number
  amount_edge: AmountEdge
  decoded: unknown
  canonical_hex: string
  wire_hex: string
  canonical_len: number
  wire_len: number
  compressed: boolean
}

const CHAINS = [1, 8453, 42161, 10, 137] as const

async function buildEntry(
  name: string,
  shape: InvoiceShape,
  invoice: Record<string, unknown>,
): Promise<CorpusEntry> {
  const canonical: Uint8Array = encodeInvoiceCanonical(invoice)
  const wire = await wireEncode(invoice)
  const canonical_hex = toHex(canonical)
  const wire_hex = toHex(wire)
  return {
    name,
    shape: shape.fill,
    language: shape.lang,
    chain: shape.chain,
    amount_edge: shape.amountEdge,
    decoded: decodeInvoiceCanonical(canonical),
    canonical_hex,
    wire_hex,
    canonical_len: canonical.length,
    wire_len: wire.length,
    compressed: isCompressed(wire_hex),
  }
}

let _seed = 0
function nextSeed(): number {
  return ++_seed
}

async function main(): Promise<void> {
  const entries: CorpusEntry[] = []

  // A) All 5 chains × minimal × ascii × typical
  for (const chain of CHAINS) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'minimal', lang: 'ascii', chain, amountEdge: 'typical' }
    entries.push(await buildEntry(`A-chain${chain}-min-ascii-typical`, shape, buildInvoice(shape, s)))
  }

  // B) All 5 chains × medium × ascii × typical
  for (const chain of CHAINS) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'medium', lang: 'ascii', chain, amountEdge: 'typical' }
    entries.push(await buildEntry(`B-chain${chain}-med-ascii-typical`, shape, buildInvoice(shape, s)))
  }

  // C) All 5 chains × full × ascii × typical
  for (const chain of CHAINS) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'full', lang: 'ascii', chain, amountEdge: 'typical' }
    entries.push(await buildEntry(`C-chain${chain}-full-ascii-typical`, shape, buildInvoice(shape, s)))
  }

  // D) Chain=1 × all 3 fills × all 6 languages × typical
  const fills: FillLevel[] = ['minimal', 'medium', 'full']
  const langs: Language[] = ['ascii', 'cyrillic', 'cjk', 'emoji', 'rtl', 'high-entropy']
  for (const fill of fills) {
    for (const lang of langs) {
      const s = nextSeed()
      const shape: InvoiceShape = { fill, lang, chain: 1, amountEdge: 'typical' }
      entries.push(await buildEntry(`D-ch1-${fill}-${lang}-typical`, shape, buildInvoice(shape, s)))
    }
  }

  // E) Chain=1 × minimal × ascii × all 5 amount-edges
  const amountEdges: AmountEdge[] = ['zero', 'one', 'typical', 'large', 'u256-max']
  for (const amountEdge of amountEdges) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'minimal', lang: 'ascii', chain: 1, amountEdge }
    entries.push(await buildEntry(`E-ch1-min-ascii-${amountEdge}`, shape, buildInvoice(shape, s)))
  }

  // F) Chain=8453 × medium × {cyrillic,cjk,emoji,rtl,high-entropy} × typical
  for (const lang of (['cyrillic', 'cjk', 'emoji', 'rtl', 'high-entropy'] as Language[])) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'medium', lang, chain: 8453, amountEdge: 'typical' }
    entries.push(await buildEntry(`F-ch8453-med-${lang}-typical`, shape, buildInvoice(shape, s)))
  }

  // G) Chain=42161 × full × {ascii,cyrillic,cjk} × {large,u256-max}
  for (const lang of (['ascii', 'cyrillic', 'cjk'] as Language[])) {
    for (const amountEdge of (['large', 'u256-max'] as AmountEdge[])) {
      const s = nextSeed()
      const shape: InvoiceShape = { fill: 'full', lang, chain: 42161, amountEdge }
      entries.push(await buildEntry(`G-ch42161-full-${lang}-${amountEdge}`, shape, buildInvoice(shape, s)))
    }
  }

  // H) Chain=137 × medium × {ascii,cjk} × typical
  for (const lang of (['ascii', 'cjk'] as Language[])) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'medium', lang, chain: 137, amountEdge: 'typical' }
    entries.push(await buildEntry(`H-ch137-med-${lang}-typical`, shape, buildInvoice(shape, s)))
  }

  // I) Chain=10 × full × {emoji,rtl} × large
  for (const lang of (['emoji', 'rtl'] as Language[])) {
    const s = nextSeed()
    const shape: InvoiceShape = { fill: 'full', lang, chain: 10, amountEdge: 'large' }
    entries.push(await buildEntry(`I-ch10-full-${lang}-large`, shape, buildInvoice(shape, s)))
  }

  // J) Special: CJK notes at 280-char boundary (each char ≤3 bytes; codec stores bytes)
  // Record outcome: accepted / truncated / rejected — do NOT fix the codec.
  {
    const cjkBoundaryInvoice = {
      invoice_id: 'CORP-CJK280',
      issued_at: ISSUED_AT,
      due_at: DUE_AT,
      network_id: 1,
      currency: 'USDC',
      decimals: 6,
      from: { name: 'Alice', wallet_address: FROM_WALLET },
      client: { name: 'Bob' },
      items: [{ description: '软件开发', quantity: 1.0, rate: '1000000' }],
      total: '1000000',
      salt: SALT,
      notes: CJK_280_CHARS,
    }
    const cjkNoteCodePoints = [...CJK_280_CHARS].length
    const cjkNoteBytes = new TextEncoder().encode(CJK_280_CHARS).length

    let outcome: string
    let entry: CorpusEntry | null = null
    try {
      const shape: InvoiceShape = { fill: 'full', lang: 'cjk', chain: 1, amountEdge: 'typical' }
      entry = await buildEntry('J-cjk-notes-280chars', shape, cjkBoundaryInvoice)
      outcome = `accepted: ${cjkNoteCodePoints} code-points / ${cjkNoteBytes} bytes stored`
      entries.push(entry)
    } catch (err: unknown) {
      outcome = `rejected/truncated: ${String(err)} (${cjkNoteCodePoints} code-points / ${cjkNoteBytes} bytes)`
    }

    console.log(`\n[J] CJK 280-char boundary outcome: ${outcome}`)
    console.log(`    note char count: ${cjkNoteCodePoints}, byte count: ${cjkNoteBytes}`)
  }

  // Write output
  fs.mkdirSync(VECTORS_DIR, { recursive: true })
  const output = {
    schema_version: 1,
    generated_by: '@void-layer/codec v0.1.0',
    generated_at: '2026-05-22',
    entry_count: entries.length,
    entries,
  }
  fs.writeFileSync(OUT_PATH, JSON.stringify(output, null, 2) + '\n')

  console.log(`\nGenerated ${entries.length} corpus entries → ${OUT_PATH}`)

  // Compression ratio table per shape
  const shapeStats: Record<string, { ratios: number[]; overCap: string[] }> = {}
  for (const e of entries) {
    if (!shapeStats[e.shape]) shapeStats[e.shape] = { ratios: [], overCap: [] }
    const ratio = e.wire_len / e.canonical_len
    shapeStats[e.shape]!.ratios.push(ratio)
    // URL-cap check: base64url expansion ceil(wire_len * 4/3) <= 2000
    const b64expanded = Math.ceil(e.wire_len * 4 / 3)
    if ((e.shape === 'medium' || e.shape === 'full') && b64expanded > 2000) {
      shapeStats[e.shape]!.overCap.push(`${e.name} (${b64expanded}B b64)`)
    }
  }

  console.log('\nCompression ratio per shape (wire_len / canonical_len):')
  console.table(
    Object.fromEntries(
      Object.entries(shapeStats).map(([shape, { ratios }]) => {
        const sorted = [...ratios].sort((a, b) => a - b)
        return [
          shape,
          {
            count: ratios.length,
            best: sorted[0]!.toFixed(3),
            median: sorted[Math.floor(sorted.length / 2)]!.toFixed(3),
            worst: sorted[sorted.length - 1]!.toFixed(3),
          },
        ]
      }),
    ),
  )

  const allOverCap = Object.values(shapeStats).flatMap((s) => s.overCap)
  if (allOverCap.length > 0) {
    console.error('\n[URL-CAP OVERFLOW] These medium/full entries exceed 2000-byte base64url cap:')
    for (const name of allOverCap) console.error(`  ${name}`)
    process.exit(1)
  } else {
    console.log('\n[URL-CAP] All medium/full entries within 2000-byte base64url cap.')
  }
}

main().catch((err) => {
  console.error('Corpus generation failed:', err)
  process.exit(1)
})
