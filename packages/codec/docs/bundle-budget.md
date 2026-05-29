# Bundle Budget — @void-layer/codec v0.1.0 (Phase 2)

> Architecture: B-v — Brotli lives in the JS shim (`dist/index.js`), NOT in the WASM.
> The WASM exposes only canonical encode/decode + receiptHash.
> Gzip is the gated metric (spec §3).

| Component | Bytes | Cap | Margin |
|-----------|-------|-----|--------|
| `void_layer_codec_bg.wasm` raw | 180,017 | — | — |
| `void_layer_codec_bg.wasm` gzip | 78,412 | 81,920 (80 KB) | ~4.3% |
| Package tarball (`pkg/` + `dist/`) | 92,160 | 204,800 (200 KB) | ~55% |

> Measured 2026-05-25 post R1-R9 DRY refactor. gzip figure uses `gzip -c` (the
> `scripts/assert-size.sh` gate method).

## Recent Deltas

| Change | gzip delta |
|--------|-----------|
| U256 widening (ruint, D-B8) | +6 KB |
| T6 decoder strictness gates (4 checks) | +~0.7 KB |
| R1-R9 intra-codec DRY refactor | ~0 net |

## Notes

- **gzip figure vs earlier ~73 KB**: the increase is due to the U256/ruint widening
  added for full BigInt support (spec §D-B8). ruint brings additional lookup tables
  and arithmetic paths that add ~6 KB gzip.
- **No brotli-decompressor row**: Brotli decompression is NOT in the WASM (B-v
  decision, 2026-05-20). The JS shim (`dist/index.js`) imports `brotli-wasm` as a
  peer dependency and handles compression/decompression outside the WASM boundary.
- **Anti-stop guard**: if a future change pushes gzip over 81,920 bytes, halt and
  report to Kai. Do NOT raise the cap unilaterally.

## Caps (spec §3)

| Gate | Cap | Enforcement |
|------|-----|-------------|
| WASM gzip | 81,920 bytes (80 KB) | Hard — CI exits 1 on breach |
| Package tarball | 204,800 bytes (200 KB) | Advisory — CI logs warning, does not fail (Phase 2 amend) |

> **200 KB cap doctrine** (Phase 2 amend, Kai decision 2026-05-20): the 200 KB
> package-tarball cap was demoted from hard-exit to advisory. CI logs the measurement
> but does not block merges on tarball size alone. The 80 KB WASM gzip cap remains
> hard. See `scripts/assert-size.sh` for the gate implementation.
