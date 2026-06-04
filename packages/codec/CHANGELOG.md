# Changelog

All notable changes to `@void-layer/codec` will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## 0.1.0

### Added

- **B-v codec architecture** — Rust WASM exposes canonical encode/decode + `receiptHash`; Brotli compression lives in the JS shim (`dist/index.js`) via `brotli-wasm` peer dependency. Wire format: `[MAGIC 0x56][VERSION | 0x80][brotli body]`, falls back to uncompressed when Brotli would expand the payload.
- **U256 amount domain** — full `uint256` range via `ruint` crate; amounts encoded as `[mantissa_varint][zeros_u8]` pairs. Encode rejects amounts exceeding `U256::MAX` with `InvalidAmount`.
- **27 golden vectors** (`vectors/v4-codec.json`, `schema_version = 1`) covering minimal, chain-selector, BigInt edge, extension, unicode coverage, and malformed decode paths. Bidirectional Rust ↔ TS parity asserted by `ts-rust-parity` CI job.
- **54-entry parametric corpus** (`vectors/corpus.json`) — deterministic cross-product of chain × fill-level × language × amount-edge; checked by `tests/compression.test.ts` and `tests/corpus.rs`.
- **Content hash** — `receiptHash(canonical_bytes)` returns `keccak256` (32-byte `Uint8Array`); suitable as ERC-3009 nonce. Callers must pass output of `encodeInvoiceCanonical`, never received bytes.
- **TLV Registry** (`REGISTRY.md`) — BOLT-style federated governance; vendor namespace 1000–9999 FCFS via GitHub PR.
- **CI scaffold** — `ci.yml` (lint + test + wasm-size-gate), `ts-rust-parity` job, `ci-gate` meta-job.

### Changed (T6 — decoder hardening, 4 strictness gates)

- Reject raw-form encoding of any chain ID that exists in `CHAIN_DICT` → `InvalidData("non-canonical chain encoding: …")`.
- Reject raw-form encoding of any currency symbol that exists in `CURRENCY_DICT` → `InvalidData("non-canonical currency encoding: …")`.
- Reject unknown prefix byte (≠ `0x00`/`0x01`) on currency and token-address TLV fields → `UnknownExtension(prefix)`.
- Reject per-item quantity scale > `MAX_CANONICAL_QUANTITY_SCALE` (9) → `InvalidData("non-canonical quantity scale …")`.

### Changed (fix-batch-6 — 7 code-review fixes)

- Dict reverse-lookup unified via `lookup_by_code` helper (eliminates dual `find_map` pattern).
- `decode_prefixed` helper centralises prefix-dispatch for chain/currency/token-address TLV fields.
- `read_optional` helper collapses optional-field reads via `Option::map`/`transpose`.
- `utf8_or` helper extracts UTF-8 decode + error tagging.
- `hex_decode_fixed` shared helper for address and salt decoding.
- `is_none_or` combinator for chain-range varint guards.
- Named quantity constants replace magic literals in encoder.

### Changed (R1-R9 — intra-codec DRY refactor, zero net size impact)

- R1: `CURRENCY_DICT` extracted to `dict/currency.rs`.
- R2: `TOKEN_DICT` extracted to `dict/token.rs`.
- R3: `canonical.rs` holds shared encode/decode canonical-form constants.
- R4: `DICT_FORM`/`RAW_FORM` prefix constants centralised in `dict/mod.rs`.
- R5: `MAX_CANONICAL_QUANTITY_SCALE` constant in `encode/limits.rs`.
- R6: `read_optional` helper in `decode/mod.rs`.
- R7: `utf8_or` helper in `decode/mod.rs`.
- R8: `lookup_by_code` generic helper in `decode/dict.rs`.
- R9: `decode_prefixed` helper in `decode/dict.rs`.

### Test growth

- Unit tests: ~135 → 211 (post R1-R9 + T6 hardening).
- Golden vectors: 27 (Tier 1 frozen) + 54 corpus entries (Tier 2 parametric).
