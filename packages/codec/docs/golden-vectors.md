# Golden Vectors — `vectors/v4-codec.json`

> **Append-only forever.** Once a vector is committed, its `name`, `canonical_hex`,
> and `wire_hex` are immutable. The only permitted change is adding new vectors at
> the end of the array. Amending an existing vector is a Constitution IV violation.

---

## Purpose

Golden vectors are the wire-format regression suite for `@void-layer/codec`. They
serve three functions:

1. **Byte-stable reference** — any future codec implementation (Rust, TS, Python,
   Go) must produce identical `canonical_hex` bytes for the same `decoded` input.
2. **Parity gate** — the `vector-parity` CI job (T-P2-13) loads `v4-codec.json`
   and asserts both directions × both forms (canonical + wire) in Rust and TS.
3. **Perpetuity proof** — URLs generated today must decode correctly in any future
   version. The vectors are the machine-readable proof of that contract.

---

## Schema (`schema_version: 1`)

### Non-malformed vector

```jsonc
{
  "name": "minimal-single-tlv",           // stable identifier, kebab-case
  "canonical_hex": "5601...",             // hex of encodeInvoiceCanonical output
  "wire_hex": "5601...",                  // hex of encodeInvoiceWire output
  "decoded": { ... },                     // the Invoice object (source of truth)
  "roundtrip": true,                      // decode(encode(decoded)) === decoded
  "diagnostic": "..."                     // human-readable note
}
```

`wire_hex` is Brotli-compressed (VERSION byte has `0x80` set) when Brotli reduces
the payload size. For small invoices Brotli expands, so `wire_hex === canonical_hex`
and the `COMPRESSED_FLAG` is NOT set — per C4 amendment (2026-05-20). Both fields
are always present regardless.

### Malformed vector — decode-input subtype

```jsonc
{
  "name": "malformed-bad-magic",
  "canonical_hex": "ff01...",        // OR wire_hex — whichever layer the error targets
  "diagnostic": "malformed:canonical",  // or "malformed:wire"
  "expected_error": "BadMagic"
}
```

Decode-input malformed vectors carry one hex field (`canonical_hex` or `wire_hex`)
and no `decoded` field. Feed the bytes to the decoder; assert the named error variant.

### Malformed vector — encode-input subtype

```jsonc
{
  "name": "bigint-amount-over-u256",
  "decoded": { "total": "115792...", ... },  // full Invoice
  "diagnostic": "malformed:encode-input",
  "expected_error": "InvalidAmount"
}
```

Encode-input malformed vectors carry a `decoded` Invoice and no hex fields. The error
fires at encode time — no bytes are produced. Construct the Invoice from `decoded` and
assert that `encodeInvoiceCanonical` throws the named error variant.

`diagnostic` prefix summary:
- `malformed:canonical` — decode `canonical_hex` → expect error
- `malformed:wire` — decode `wire_hex` → expect error
- `malformed:encode-input` — encode `decoded` Invoice → expect error

---

## Starter Set (v4-codec.json, schema_version=1)

18 vectors, regenerated 2026-05-20 for U256 widening (T-P2-12a / C9 amendment) and
corrected malformed vector set (T-P2-12 follow-up, Kai decision 2026-05-20).

| # | Name | Category | wire compressed |
|---|------|----------|----------------|
| 1 | `minimal-single-tlv` | Minimal | false |
| 2 | `chain-ethereum` | Chain selector | false |
| 3 | `chain-base` | Chain selector | false |
| 4 | `chain-arbitrum` | Chain selector | false |
| 5 | `chain-optimism` | Chain selector | false |
| 6 | `chain-polygon` | Chain selector | false |
| 7 | `bigint-amount-zero` | BigInt edge | false |
| 8 | `bigint-amount-one` | BigInt edge | false |
| 9 | `bigint-amount-uint256-max` | BigInt edge | **true** |
| 10 | `bigint-amount-over-u256` | BigInt edge (malformed — InvalidAmount) | — |
| 11 | `malformed-checksum-mismatch` | Malformed | — |
| 12 | `malformed-varint-overflow` | Malformed | — |
| 13 | `extension-magic-dust` | Extension | **true** |
| 14 | `extension-og-param` | Extension | **true** |
| 15 | `extension-sub-invoice-chain` | Extension | false |
| 16 | `malformed-corrupted-brotli` | Malformed | — |
| 17 | `malformed-oversize` | Malformed | — |
| 18 | `malformed-bad-magic` | Malformed | — |

**Changes from initial 16-vector set (C9 amendment, 2026-05-20)**:
- `bigint-amount-u128-max` replaced by `bigint-amount-uint256-max` (U256::MAX =
  `115792089237316195423570985008687907853269984665640564039457584007913129639935`).
  After U256 widening this encodes successfully (roundtrip=true, wire compressed).
- `bigint-amount-over-u256` added: amount = 2^256, encode rejects with `InvalidAmount`.
  No canonical_hex field — error fires at encode time, no bytes produced.

**Changes from 17-vector set (T-P2-12 follow-up, Kai decision 2026-05-20)**:
- `malformed-varint-overflow` corrected: the previous hex (`56 01 01 18 0x26 38×0x80`)
  was misidentified — the codec hits `ChecksumMismatch` before any varint overflow path.
  The old hex is preserved as `malformed-checksum-mismatch` (new name, same bytes).
- New `malformed-varint-overflow` added: hex = `56 01 01 18` + 37×`0x80`. The LENGTH
  field of the first TLV record is 37 continuation bytes with no terminal byte. The
  varint decoder fires `VarintOverflow` at `bytes_read == MAX_BYTES (37)` before the
  checksum stage. Empirically confirmed on both WASM and Rust surfaces.

**Why some vectors are uncompressed**: the T-P2-0a Brotli spike measured that
payloads under ~180 bytes expand under Brotli q11. All single-item minimal invoices
fall below this threshold. The `bigint-amount-uint256-max`, `extension-magic-dust`,
and `extension-og-param` vectors are compressed due to larger payloads.

---

## Append-Only Rule

Adding new vectors (at the end of the array) is always safe.

The following operations are FORBIDDEN:
- Changing `name`, `canonical_hex`, or `wire_hex` of any existing vector
- Reordering vectors
- Removing vectors
- Changing `schema_version` (a new schema gets a new file, e.g. `v4-codec-v2.json`)

If you need to correct a vector that has never been published in an npm release,
open a PR, reference the Kai decision that approves the correction, and include a
`BREAKING` note in the changeset.

---

## Regenerating

The generator is `scripts/generate-vectors.ts`. It imports from `pkg-node/`
(nodejs-target WASM build) and mirrors the `src/index.ts` shim wire logic.

```bash
# From packages/codec root:
pnpm build:nodejs      # rebuild pkg-node/ if Rust changed
pnpm generate-vectors  # runs scripts/run-generate-vectors.test.ts via dedicated config
```

`pnpm test` intentionally excludes `scripts/**` — regeneration is always explicit.

Regeneration replaces the file. Diff the output carefully before committing —
any change to an existing vector's hex fields is a perpetuity violation.

---

## CodecError variants (expected_error values)

| Variant | Trigger |
|---------|---------|
| `BadMagic` | First byte is not `0x56` |
| `VarintOverflow` | LEB128 continuation bytes exceed MAX_BYTES (37) |
| `Truncated` | Buffer ends before a TLV value is fully read |
| `CompressionFailed` | Brotli decompression error on a wire payload |
| `UnsupportedVersion` | Version byte signals an unknown codec version |
| `DictionaryMismatch` | Dict hash in payload does not match compiled dict |
| `InvalidAmount` | Amount string exceeds U256::MAX or is not a valid decimal |

See `src/error.rs` for the full 10-variant enum.
