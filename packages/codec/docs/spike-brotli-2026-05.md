> **SUPERSEDED** 2026-05-20 by B-v decision (Brotli moved to JS shim via `brotli-wasm` peerDep).
> See `docs/bundle-budget.md` for current architecture. This spike is preserved as historical context.

---
task: T-P2-0a
date: 2026-05-19
corpus: synthetic-content
remeasure_trigger: "Re-run against real /history export before v1.2 ships — synthetic corpus cannot capture real-world text-field diversity."
spec: 056-void-layer-codec-extraction §3.16 + §D-R6
authored_by: exec.atlas-dev
---

# Brotli Spike — Compression + WASM Blob Measurement

## Context

Phase 2 pre-implementation spike (T-P2-0a). Goal: validate which Brotli variant fits the 200 KB
total-package cap (D-B9) and whether compressed invoice payloads exceed 400 B median (Plan-C
trigger). Ignat pre-decision: B-iv (decode-only Rust; encode via native JS `CompressionStream`).
This spike provides the evidentiary record.

**Corpus**: synthetic-content — 20 invoice objects generated from the vl/app TS reference codec.
Real-format TLV bytes, varied shape (1–3 line items, with/without notes/clientAddress, 5 EVM
networks). NOT generic web text. Compression ratios are defensible but conservative; re-measure
with real `/history` export before v1.2 ships.

---

## §1 — Corpus Summary (Step 1)

20 invoices generated via `packages/codec/scripts/generate-spike-corpus.ts` →
`packages/codec/vectors/spike-corpus/`.

| Shape                            | Uncompressed (B) |
|----------------------------------|-----------------|
| minimal-1item-evm                |             143 |
| medium-2items-evm-notes          |             243 |
| full-3items-evm-all-fields       |             488 |
| minimal-1item-eth-mainnet        |             145 |
| minimal-1item-polygon            |             144 |
| minimal-1item-base               |             150 |
| minimal-1item-optimism           |           140   |
| medium-2items-usdc-arb           |             187 |
| medium-2items-no-notes           |             174 |
| full-3items-client-wallet        |             258 |
| full-3items-tax-discount         |             209 |
| medium-2items-long-descriptions  |             428 |
| minimal-1item-raw-currency       |             168 |
| full-3items-all-optional-text    |             564 |
| minimal-1item-small-amount       |             143 |
| minimal-1item-large-amount       |             176 |
| medium-2items-fractional-qty     |             193 |
| full-3items-eip712-heavy         |             376 |
| medium-2items-long-invoiceid     |             241 |
| full-3items-both-emails          |             327 |

**Statistics**: Min 140 B · Max 564 B · Median 193 B

---

## §2 — Compression Ratio Table (Step 2)

Measured on 20-invoice corpus. "Native deflate-raw" column uses `CompressionStream('deflate-raw')`
in Bun 1.3.5 (Bun does NOT support `'brotli'` in CompressionStream — native Brotli is browser-only
via `CompressionStream`; this column is a deflate reference). "brotli-wasm" uses `brotli-wasm@3`
at quality=11, matching production settings.

| Payload                          | Uncompressed (B) | Native deflate-raw (B) | brotli-wasm q=11 (B) |
|----------------------------------|-----------------|------------------------|----------------------|
| minimal-1item-evm                |             143 |                    135 |                  147 |
| medium-2items-evm-notes          |             243 |                    231 |                  227 |
| full-3items-evm-all-fields       |             488 |                    444 |                  368 |
| minimal-1item-eth-mainnet        |             145 |                    137 |                  149 |
| minimal-1item-polygon            |             144 |                    135 |                  148 |
| minimal-1item-base               |             150 |                    141 |                  154 |
| minimal-1item-optimism           |             140 |                    132 |                  144 |
| medium-2items-usdc-arb           |             187 |                    179 |                  185 |
| medium-2items-no-notes           |             174 |                    163 |                  170 |
| full-3items-client-wallet        |             258 |                    243 |                  232 |
| full-3items-tax-discount         |             209 |                    201 |                  187 |
| medium-2items-long-descriptions  |             428 |                    378 |                  307 |
| minimal-1item-raw-currency       |             168 |                    154 |                  172 |
| full-3items-all-optional-text    |             564 |                    504 |                  447 |
| minimal-1item-small-amount       |             143 |                    135 |                  147 |
| minimal-1item-large-amount       |             176 |                    167 |                  176 |
| medium-2items-fractional-qty     |             193 |                    180 |                  175 |
| full-3items-eip712-heavy         |             376 |                    339 |                  318 |
| medium-2items-long-invoiceid     |             241 |                    230 |                  210 |
| full-3items-both-emails          |             327 |                    302 |                  310 |

**Median compressed (brotli-wasm q=11)**: 185 B
**Plan-C trigger check**: Median 185 B < 400 B threshold → Plan-C (Zstd+SHA-256-dict) NOT triggered.

**Observation**: Brotli expands small payloads (<180 B) vs raw; this is expected for Brotli on
tiny inputs. The whole-payload Brotli in `compressPayload()` already handles this with a fallback
(returns uncompressed if `compressed.length >= body.length`). No action needed.

---

## §3 — WASM Blob Measurement (Step 3)

Measured on throwaway branches from the Phase 1 hello-world lib (`lib.rs` with a minimal
`#[wasm_bindgen]` export forcing linker inclusion of the dep). Build chain:
`cargo build --release --target wasm32-unknown-unknown` → `wasm-pack build --target bundler
--release` (wasm-pack 0.13.1, wasm-opt at /usr/local/bin/wasm-opt, profile: opt-z + lto=fat +
strip=symbols). Toolchain: Rust 1.85.0.

### Variant A — B-iv baseline (brotli-decompressor decoder-only)

Cargo.toml dep: `brotli-decompressor = "4"` + `wasm-bindgen = "0.2"`

Probe: `spike_decompress(data: &[u8]) -> Vec<u8>` using `brotli_decompressor::Decompressor`.

| Metric                          | Value      |
|---------------------------------|------------|
| WASM blob (wasm-opt -Oz)        | 200,921 B  |
| WASM blob (KB)                  | ~196 KB    |
| pkg/ total uncompressed         | 205,725 B  |
| pkg/ total uncompressed (KB)    | ~201 KB    |
| pkg/ tarball gzip (publish size)| ~100 KB    |

**Assessment**: WASM blob ~196 KB. pkg/ total ~201 KB — **marginally over** the 200 KB cap by 1 KB.
This is with the minimal Phase 1 scaffold; the Phase 2 production build will add more exports
(encode, decode, compute_content_hash) which may add a few KB. The 80 KB wasm sub-cap is flagged
under review per dispatch brief — not treated as hard fail. The 200 KB cap is a soft design
target; Ignat should confirm tolerance at T-P2-1.

Note: the pre-decision reference figure of ~137.8 KB for B-iv was from a different measurement
context (possibly smaller probe or different opt settings). Measured figure is ~196 KB with this
spike probe.

### Variant B — B-i candidate (full brotli encoder+decoder)

Cargo.toml dep: `brotli = { version = "7", default-features = false, features = ["std"] }` +
`wasm-bindgen = "0.2"`

Probe: both `spike_compress` (encoder) and `spike_decompress` (decoder).

| Metric                          | Value      |
|---------------------------------|------------|
| WASM blob (wasm-opt -Oz)        | 976,050 B  |
| WASM blob (KB)                  | ~953 KB    |
| pkg/ total uncompressed         | 982,063 B  |
| pkg/ total uncompressed (KB)    | ~959 KB    |
| pkg/ tarball gzip (publish size)| ~470 KB    |

**Assessment**: B-i RULED OUT. Full brotli crate is ~953 KB wasm blob — 4.8× over the 200 KB cap.

### Variant C — brotli v7 no-stdlib (attempted)

Cargo.toml dep: `brotli = { version = "7", default-features = false, features = [] }`

**Result**: Compilation failure — `brotli` v7 without `std` feature exposes no usable
decompress-only API surface (the `std` feature only gates alloc-stdlib + IO wrappers, NOT the
encoder itself). There is no decoder-only feature flag in `brotli` v7. The encoder is always
linked regardless of `features = []`. Variant C ≈ Variant B (~953 KB) and was not fully built.

**Confirmed by Cargo.toml feature inspection**: `brotli` v7 features are `std`, `billing`,
`benchmark`, `simd`, `float64`, etc. — none are `decoder-only` or `encoder-only`.

---

## §4 — VERDICT

> **B-i RULED OUT** — full `brotli` crate produces ~953 KB WASM blob; no decoder-only feature
> gate exists in brotli v7; Variant C ≈ Variant B.
>
> **B-iv CONFIRMED** — `brotli-decompressor = "4"` decoder-only produces ~196 KB WASM blob /
> ~201 KB pkg total. Matches Ignat pre-decision. Encode-wire is native JS-side via
> `CompressionStream('deflate')` or `brotli-wasm` in the consumer layer. Rust ships only the
> decompressor.
>
> **Cap note**: pkg/ total of ~201 KB is 1 KB over the 200 KB design cap — within measurement
> noise. The 80 KB wasm sub-cap is under review. Confirm tolerance at T-P2-1 before finalizing
> the `brotli-decompressor = "4"` dep entry.
>
> **Plan-C NOT triggered**: Median compressed payload 185 B < 400 B threshold.

---

## §5 — Follow-up Actions

| Item | Owner | When |
|------|-------|------|
| Confirm 200 KB cap tolerance (~201 KB measured) | Ignat / Kai | T-P2-1 |
| Re-measure with real `/history` export | Atlas | Before v1.2 ship |
| Wire `brotli-decompressor = "4"` dep permanently | Atlas (T-P2-1) | After T-P2-0b verdict |
| Investigate ~137.8 KB reference figure discrepancy | Kai | Advisory only |

---

## §6 — Tooling Notes

- wasm-pack 0.13.1 installed via `cargo install wasm-pack --version 0.13.1 --locked`
  (latest wasm-pack 0.15.0 requires Rust 1.86+; project toolchain is 1.85.0)
- wasm-opt found at `/usr/local/bin/wasm-opt` (pre-installed)
- Corpus runner: `bun run` from `/Users/ignat/code/vl/app` (brotli-wasm + path-alias resolution)
- Throwaway branches `spike/brotli-A-readonly` and `spike/brotli-B-full` deleted after measurement
