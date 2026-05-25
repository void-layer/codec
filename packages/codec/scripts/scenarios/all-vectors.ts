/**
 * Full vector corpus (non-malformed + malformed), preserving original order.
 *
 * Order: minimal → chains → bigints → [early malformed] → extensions →
 *        unicode → [late malformed]
 *
 * Demo-invoice vectors are appended by the top-level generator.
 */

import { base } from '../lib/invoice-base.js'
import { nonMalformed, WIRE_DIAG, type NonMalformedVector } from './non-malformed.js'
import {
  buildEarlyMalformedVectors,
  buildLateMalformedVectors,
  type MalformedVector,
} from './malformed.js'

export type AnyVector = NonMalformedVector | MalformedVector

export async function buildAllVectors(): Promise<AnyVector[]> {
  const vectors: AnyVector[] = []

  // 1. Minimal
  vectors.push(
    await nonMalformed(
      'minimal-single-tlv',
      base({}),
      `Smallest valid invoice — all required fields, one item, no optional fields. ${WIRE_DIAG}`,
    ),
  )

  // 2. Chain selectors (5)
  const chains: Array<[number, string]> = [
    [1, 'ethereum'],
    [8453, 'base'],
    [42161, 'arbitrum'],
    [10, 'optimism'],
    [137, 'polygon'],
  ]
  for (const [network_id, chainName] of chains) {
    vectors.push(
      await nonMalformed(
        `chain-${chainName}`,
        base({ network_id, invoice_id: `INV-CHAIN-${network_id}` }),
        `Chain selector: ${chainName} (network_id=${network_id}). ${WIRE_DIAG}`,
      ),
    )
  }

  // 3. BigInt edges — non-malformed subset (a, b, c)

  // 3a. amount = 0
  vectors.push(
    await nonMalformed(
      'bigint-amount-zero',
      base({
        invoice_id: 'INV-BIGINT-ZERO',
        items: [{ description: 'Zero payment', quantity: 1.0, rate: '0' }],
        total: '0',
      }),
      `BigInt edge: total = 0 (LEB128 single 0x00 byte). ${WIRE_DIAG}`,
    ),
  )

  // 3b. amount = 1
  vectors.push(
    await nonMalformed(
      'bigint-amount-one',
      base({
        invoice_id: 'INV-BIGINT-ONE',
        items: [{ description: 'One atomic unit', quantity: 1.0, rate: '1' }],
        total: '1',
      }),
      `BigInt edge: total = 1 (smallest nonzero, no trailing zeros). ${WIRE_DIAG}`,
    ),
  )

  // 3c. U256::MAX
  const U256_MAX = '115792089237316195423570985008687907853269984665640564039457584007913129639935'
  vectors.push(
    await nonMalformed(
      'bigint-amount-uint256-max',
      base({
        invoice_id: 'INV-BIGINT-U256MAX',
        currency: 'ETH',
        decimals: 18,
        items: [{ description: 'Max uint256 payment', quantity: 1.0, rate: U256_MAX }],
        total: U256_MAX,
      }),
      `BigInt edge: total = U256::MAX (${U256_MAX}) — largest encodable value after U256 widening. ${WIRE_DIAG}`,
    ),
  )

  // 3d–3f. Early malformed: over-u256, checksum-mismatch, varint-overflow
  for (const v of buildEarlyMalformedVectors()) {
    vectors.push(v)
  }

  // 4. Extensions (3)

  // 4a. magic-dust
  vectors.push(
    await nonMalformed(
      'extension-magic-dust',
      base({
        invoice_id: 'INV-EXT-DUST',
        total: '1000042',
        notes: 'Magic dust applied: +0.000042 for unique matching',
        items: [{ description: 'Consulting', quantity: 1.0, rate: '1000042' }],
      }),
      `Extension: magic-dust (micro-amount uniquifier in total + notes field). ${WIRE_DIAG}`,
    ),
  )

  // 4b. OG-param
  vectors.push(
    await nonMalformed(
      'extension-og-param',
      base({
        invoice_id: 'INV-EXT-OG',
        from: { name: 'Alice Dev Studio', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045', email: 'alice@dev.io' },
        client: { name: 'Acme Corp', wallet_address: '0x70997970C51812dc3A010C7d01b50e0d17dc79C8' },
        notes: 'Please pay within 30 days',
        total: '5000000',
        items: [{ description: 'Design work', quantity: 1.0, rate: '5000000' }],
      }),
      `Extension: OG-param fields (from.email, client.wallet_address, notes) for social preview. ${WIRE_DIAG}`,
    ),
  )

  // 4c. sub-invoice-chain
  vectors.push(
    await nonMalformed(
      'extension-sub-invoice-chain',
      base({
        invoice_id: 'INV-EXT-SUBCHAIN',
        network_id: 42161,
        currency: 'ETH',
        decimals: 18,
        total: '500000000000000000',
        items: [{ description: 'Cross-chain consulting', quantity: 1.0, rate: '500000000000000000' }],
        tax: '10',
        discount: '5',
      }),
      `Extension: sub-invoice chain — ETH on Arbitrum with tax and discount fields. ${WIRE_DIAG}`,
    ),
  )

  // 5. Unicode vectors (5)

  // 5a. Cyrillic
  vectors.push(
    await nonMalformed(
      'unicode-cyrillic',
      base({
        invoice_id: 'INV-UNI-CYR',
        from: { name: 'Алиса Разработчик', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' },
        client: { name: 'Боб Клиент' },
        items: [{ description: 'Консультационные услуги', quantity: 1.0, rate: '2000000' }],
        total: '2000000',
        notes: 'Оплата в течение 30 дней',
      }),
      `Unicode: Cyrillic (2-byte UTF-8) in from.name, client.name, item.description, notes. ${WIRE_DIAG}`,
    ),
  )

  // 5b. CJK
  vectors.push(
    await nonMalformed(
      'unicode-cjk',
      base({
        invoice_id: 'INV-UNI-CJK',
        from: { name: 'Alice', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' },
        client: { name: 'Bob' },
        items: [{ description: '软件开发咨询服务', quantity: 1.0, rate: '3000000' }],
        total: '3000000',
        notes: '請在30天內付款。感謝您的支持。',
      }),
      `Unicode: CJK (3-byte UTF-8) in item.description and notes. ${WIRE_DIAG}`,
    ),
  )

  // 5c. Emoji
  vectors.push(
    await nonMalformed(
      'unicode-emoji',
      base({
        invoice_id: 'INV-UNI-EMJ',
        from: { name: 'Alice 🚀', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' },
        client: { name: 'Bob' },
        items: [{ description: 'Premium consulting', quantity: 1.0, rate: '5000000' }],
        total: '5000000',
        notes: '✅ Payment confirmed 🎉 Thank you! 💎',
      }),
      `Unicode: emoji (4-byte UTF-8 surrogate pairs) in from.name and notes. Codec treats as bytes — no normalization. ${WIRE_DIAG}`,
    ),
  )

  // 5d. RTL
  vectors.push(
    await nonMalformed(
      'unicode-rtl',
      base({
        invoice_id: 'INV-UNI-RTL',
        from: { name: 'أليس المطور', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' },
        client: { name: 'Bob' },
        items: [{ description: 'خدمات استشارية', quantity: 1.0, rate: '1500000' }],
        total: '1500000',
        notes: 'يرجى الدفع خلال 30 يوماً',
      }),
      `Unicode: Arabic RTL (2-4 byte UTF-8) in from.name, description, notes. Codec treats as opaque bytes — no reorder or normalize. ${WIRE_DIAG}`,
    ),
  )

  // 5e. Mixed
  vectors.push(
    await nonMalformed(
      'unicode-mixed',
      base({
        invoice_id: 'INV-UNI-MIX',
        from: { name: 'Alice 🌍', wallet_address: '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' },
        client: { name: 'Боб / 鲍勃' },
        items: [
          { description: '咨询服务 / Consulting / Консультации', quantity: 1.0, rate: '4000000' },
        ],
        total: '4000000',
        notes: 'Mixed: Кириллица + 中文 + العربية + emoji 🎯',
      }),
      `Unicode: mixed scripts (ASCII + Cyrillic + CJK + Arabic + emoji) across all text fields. ${WIRE_DIAG}`,
    ),
  )

  // 6–8. Late malformed: corrupted-brotli, oversize, bad-magic, unknown-tlv-tag,
  //       duplicate-tlv-tag, non-canonical-varint, unknown-content-tag
  for (const v of buildLateMalformedVectors()) {
    vectors.push(v)
  }

  return vectors
}
