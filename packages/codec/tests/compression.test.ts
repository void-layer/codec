/**
 * Corpus-driven compression test — T3.
 *
 * Iterates vectors/corpus.json and asserts:
 *   (a) wire roundtrip: decodeInvoiceWire(fromHex(wire_hex)) deepEquals decoded
 *   (b) wire_len <= canonical_len for every entry (shim fallback invariant)
 *   (c) when compressed:true, strictly wire_len < canonical_len
 *   (d) URL-cap gate: ceil(wire_len * 4/3) <= 2000 for medium/full entries
 *   (e) informational: console.table of compression ratio per shape
 */

import { describe, it, expect, afterAll } from 'vitest'
import { decodeInvoiceWire } from '../src/index.js'
import corpus from '../vectors/corpus.json'

type CorpusEntry = (typeof corpus.entries)[number]

function fromHex(hex: string): Uint8Array {
  return new Uint8Array(Buffer.from(hex, 'hex'))
}

// Accumulate ratio data for the informational table emitted in afterAll.
const ratiosByShape: Record<string, number[]> = {}

function recordRatio(shape: string, wireLen: number, canonicalLen: number): void {
  if (!ratiosByShape[shape]) ratiosByShape[shape] = []
  ratiosByShape[shape]!.push(wireLen / canonicalLen)
}

afterAll(() => {
  console.table(
    Object.fromEntries(
      Object.entries(ratiosByShape).map(([shape, ratios]) => {
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
})

describe('corpus: wire roundtrip', () => {
  for (const entry of corpus.entries as CorpusEntry[]) {
    it(`roundtrip: ${entry.name}`, async () => {
      const wire = fromHex(entry.wire_hex)
      const decoded = await decodeInvoiceWire(wire)
      expect(decoded).toEqual(entry.decoded)
    })
  }
})

describe('corpus: wire_len <= canonical_len (shim fallback invariant)', () => {
  for (const entry of corpus.entries as CorpusEntry[]) {
    it(`wire_len <= canonical_len: ${entry.name}`, () => {
      recordRatio(entry.shape, entry.wire_len, entry.canonical_len)
      expect(entry.wire_len).toBeLessThanOrEqual(entry.canonical_len)
    })
  }
})

describe('corpus: compressed entries are strictly smaller', () => {
  const compressedEntries = (corpus.entries as CorpusEntry[]).filter((e) => e.compressed)

  it(`${compressedEntries.length} entries have compressed:true`, () => {
    expect(compressedEntries.length).toBeGreaterThan(0)
  })

  for (const entry of compressedEntries) {
    it(`compressed strictly smaller: ${entry.name}`, () => {
      expect(entry.wire_len).toBeLessThan(entry.canonical_len)
    })
  }
})

describe('corpus: URL-cap gate (medium/full entries)', () => {
  const realisticEntries = (corpus.entries as CorpusEntry[]).filter(
    (e) => e.shape === 'medium' || e.shape === 'full',
  )

  for (const entry of realisticEntries) {
    it(`base64url expansion <= 2000 bytes: ${entry.name}`, () => {
      const b64Expanded = Math.ceil(entry.wire_len * 4 / 3)
      expect(b64Expanded, `${entry.name}: ${b64Expanded}B base64url expansion exceeds 2000B cap`).toBeLessThanOrEqual(2000)
    })
  }
})
