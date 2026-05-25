/**
 * Demo invoice vectors sourced from vl/app landing and video demo constants.
 *
 * Source 1 — landing (5 invoices): voidpay/src/widgets/landing/constants/demo-invoices.ts
 * Source 2 — video   (1 invoice):  voidpay/src/video/src/constants/demo-invoice.ts
 *
 * Fields dropped (not in codec v1 Invoice schema):
 *   txHash, txHashValidated, magicDust (total already includes dust for video demo),
 *   any invoiceUrl / createdAt / status / createHash wrappers.
 *
 * Salts: deterministic per-vector hex strings (16 bytes = 32 hex chars) seeded by vector id.
 * Timestamps: fixed UTC midnight values so vectors stay byte-stable across builds.
 */

import { nonMalformed, WIRE_DIAG, type NonMalformedVector } from './non-malformed.js'

// Fixed timestamps — 2026-05-25 00:00:00 UTC
const ISSUED_AT = 1748131200
const DUE_AT_14 = ISSUED_AT + 14 * 86400  // +14 days
const DUE_AT_28 = ISSUED_AT + 28 * 86400  // +28 days
const DUE_AT_30 = ISSUED_AT + 30 * 86400  // +30 days

// Deterministic salts — one per vector (never reused)
const SALT_ETH    = 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6'
const SALT_BASE   = 'b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7'
const SALT_ARB    = 'c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8'
const SALT_OP     = 'd4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9'
const SALT_POLY   = 'e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0'
const SALT_VIDEO  = 'f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1'

export async function demoinvoiceVectors(): Promise<NonMalformedVector[]> {
  const results: NonMalformedVector[] = []

  // --- demo-landing-eth-001 (chain 1, ETH, smart contract audit) ---
  results.push(
    await nonMalformed(
      'demo-landing-eth-001',
      {
        invoice_id: 'INV-2026-042',
        issued_at: ISSUED_AT,
        due_at: DUE_AT_14,
        network_id: 1,
        currency: 'ETH',
        decimals: 18,
        from: {
          name: 'EtherScale Solutions',
          wallet_address: '0x5aFe000000000000000000000000000000000001',
          email: 'billing@etherscale.io',
          physical_address: '548 Market St, Suite 23000\nSan Francisco, CA 94104\nUSA',
          phone: '+1 415 555 0142',
          tax_id: 'US 12-3456789',
        },
        client: {
          name: 'DeFi Frontiers DAO',
          wallet_address: '0xbeeF000000000000000000000000000000000002',
          email: 'treasury@defifrontiers.xyz',
          physical_address: 'c/o Legal Entity\n123 Blockchain Ave\nZug, Switzerland',
          phone: '+41 41 555 0198',
          tax_id: 'CHE-123.456.789',
        },
        items: [
          { description: 'Smart Contract Security Audit', quantity: 40, rate: '125000000000000000' },
          { description: 'Gas Optimization Consulting (8 hours)', quantity: 8, rate: '100000000000000000' },
        ],
        discount: '5%',
        total: '5510000000000000000',
        salt: SALT_ETH,
      },
      `Landing demo: Ethereum (chain 1), ETH, smart contract audit. ${WIRE_DIAG}`,
    ),
  )

  // --- demo-landing-base-002 (chain 8453, USDC, smart wallet integration) ---
  results.push(
    await nonMalformed(
      'demo-landing-base-002',
      {
        invoice_id: 'INV-2026-217',
        issued_at: ISSUED_AT,
        due_at: DUE_AT_14,
        network_id: 8453,
        currency: 'USDC',
        token_address: '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913',
        decimals: 6,
        from: {
          name: 'Base Builders Co.',
          wallet_address: '0xdEaD000000000000000000000000000000000009',
          email: 'team@basebuilders.xyz',
          physical_address: '100 Innovation Drive\nSan Francisco, CA 94105\nUSA',
          phone: '+1 628 555 0321',
        },
        client: {
          name: 'Onchain Commerce DAO',
          wallet_address: '0xFeed000000000000000000000000000000000010',
          email: 'finance@onchaincommerce.xyz',
          physical_address: '42 Web3 Street\nBrooklyn, NY 11201\nUSA',
          phone: '+1 718 555 0456',
          tax_id: 'US 98-7654321',
        },
        items: [
          { description: 'Smart Wallet SDK Integration', quantity: 1, rate: '3500000000' },
          { description: 'Passkey Authentication Module', quantity: 1, rate: '2800000000' },
          { description: 'User Onboarding Flow Design', quantity: 1, rate: '1200000000' },
        ],
        notes: 'Passkey wallet integration for mobile dApp. Milestone 2 of 4.',
        tax: '5',
        total: '7875000000',
        salt: SALT_BASE,
      },
      `Landing demo: Base (chain 8453), USDC, smart wallet integration. ${WIRE_DIAG}`,
    ),
  )

  // --- demo-landing-arb-003 (chain 42161, USDC, game asset design) ---
  results.push(
    await nonMalformed(
      'demo-landing-arb-003',
      {
        invoice_id: 'INV-2026-087',
        issued_at: ISSUED_AT,
        due_at: DUE_AT_28,
        network_id: 42161,
        currency: 'USDC',
        token_address: '0xaf88d065e77c8cc2239327c5edb3a432268e5831',
        decimals: 6,
        from: {
          name: 'L2 Design Studio',
          wallet_address: '0xcAFe000000000000000000000000000000000003',
          email: 'invoices@l2design.studio',
          physical_address: '789 Creative Blvd, Unit 4\nAustin, TX 78701\nUSA',
          phone: '+1 512 555 0177',
        },
        client: {
          name: 'ArbGaming Inc.',
          wallet_address: '0xFaCE000000000000000000000000000000000004',
          email: 'payments@arbgaming.io',
          physical_address: '456 Gaming Tower, Floor 12\nSingapore 018956',
          phone: '+65 6555 0234',
        },
        items: [
          { description: 'Character Sprite Set (10 animations)', quantity: 1, rate: '1200000000' },
          { description: 'UI Animation Pack (menus, buttons)', quantity: 1, rate: '800000000' },
          { description: 'Sound Effects Integration', quantity: 1, rate: '400000000' },
        ],
        notes: 'Final delivery includes source files and commercial license.',
        tax: '8',
        discount: '5',
        total: '2472000000',
        salt: SALT_ARB,
      },
      `Landing demo: Arbitrum (chain 42161), USDC, game asset design. ${WIRE_DIAG}`,
    ),
  )

  // --- demo-landing-op-004 (chain 10, OP token, public goods grant) ---
  // OP token address 0x4200...0042 is NOT in the v1 token dict → raw form encoding.
  results.push(
    await nonMalformed(
      'demo-landing-op-004',
      {
        invoice_id: 'INV-2026-135',
        issued_at: ISSUED_AT,
        due_at: DUE_AT_30,
        network_id: 10,
        currency: 'OP',
        token_address: '0x4200000000000000000000000000000000000042',
        decimals: 18,
        from: {
          name: 'Optimistic Builders Collective',
          wallet_address: '0xBABe000000000000000000000000000000000005',
          email: 'grants@optimisticbuilders.org',
          physical_address: '1 Public Goods Way\nOptimism City, OP 10001\nDecentralized',
          phone: '+1 800 555 0100',
          tax_id: 'US 55-1234567',
        },
        client: {
          name: 'RetroPGF Foundation',
          wallet_address: '0xC0DE000000000000000000000000000000000006',
          email: 'disbursements@retropgf.eth',
          physical_address: 'Optimism Foundation\n123 Collective Drive\nRemote',
          phone: '+1 888 555 0100',
        },
        items: [
          { description: 'Public Goods Infrastructure Grant - Phase 1', quantity: 1, rate: '15000000000000000000000' },
          { description: 'Community Tooling Development', quantity: 1, rate: '8000000000000000000000' },
          { description: 'Documentation & Onboarding', quantity: 1, rate: '2000000000000000000000' },
        ],
        notes: 'Thank you for supporting public goods. Milestone 1 of 3.',
        total: '25000000000000000000000',
        salt: SALT_OP,
      },
      `Landing demo: Optimism (chain 10), OP token, public goods grant. ${WIRE_DIAG}`,
    ),
  )

  // --- demo-landing-poly-005 (chain 137, USDC, data analytics) ---
  results.push(
    await nonMalformed(
      'demo-landing-poly-005',
      {
        invoice_id: 'INV-2026-198',
        issued_at: ISSUED_AT,
        due_at: DUE_AT_30,
        network_id: 137,
        currency: 'USDC',
        token_address: '0x3c499c542cef5e3811e1192ce70d8cc03d5c3359',
        decimals: 6,
        from: {
          name: 'PolyMarket Analytics Ltd.',
          wallet_address: '0xf00D000000000000000000000000000000000007',
          email: 'billing@polymarketanalytics.com',
          physical_address: '42 Data Center Road\nMumbai, Maharashtra 400001\nIndia',
          phone: '+91 22 5555 0456',
          tax_id: 'IN GSTIN29ABCDE1234F1Z5',
        },
        client: {
          name: 'Prediction Protocol DAO',
          wallet_address: '0xfEED000000000000000000000000000000000008',
          email: 'finance@predictiondao.io',
          physical_address: 'DAO Multisig\nGlobal Decentralized Network',
          phone: '+44 20 5555 0789',
          tax_id: 'GB 123456789',
        },
        items: [
          { description: 'Market Data Feed - Premium Tier (Q1)', quantity: 3, rate: '1500000000' },
          { description: 'API Access - Unlimited Calls', quantity: 1, rate: '500000000' },
          { description: 'Custom Dashboard Setup', quantity: 1, rate: '750000000' },
        ],
        notes: 'Q1 2026 subscription. Auto-renewal unless cancelled 7 days prior.',
        tax: '18',
        discount: '10',
        total: '6210000000',
        salt: SALT_POLY,
      },
      `Landing demo: Polygon (chain 137), USDC, data analytics. ${WIRE_DIAG}`,
    ),
  )

  // --- demo-video-base-treasury-006 (chain 8453, USDC, VoidPay treasury, with Magic Dust) ---
  // Magic Dust 187 atomic units baked into total: 1000000 + 187 = 1000187.
  // USDC on Base: 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913
  results.push(
    await nonMalformed(
      'demo-video-base-treasury-006',
      {
        invoice_id: 'INV-2026-203',
        issued_at: 1779062400,  // 2026-05-18 00:00:00 UTC (fixed from source)
        due_at: 1810512000,     // 2027-05-17 00:00:00 UTC (fixed from source)
        network_id: 8453,
        currency: 'USDC',
        token_address: '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913',
        decimals: 6,
        from: {
          name: 'VoidPay',
          wallet_address: '0xA8A1F79C4dAa2eC25Af2C91349A6F60c5b41160E',
        },
        client: {
          name: 'You',
        },
        items: [
          { description: 'Support VoidPay', quantity: 1, rate: '1000000' },
        ],
        total: '1000187',  // 1.000000 USDC + 187 atomic units Magic Dust
        salt: SALT_VIDEO,
      },
      `Video demo: Base (chain 8453), USDC, VoidPay treasury, total includes Magic Dust (+187 atomic units). ${WIRE_DIAG}`,
    ),
  )

  return results
}
