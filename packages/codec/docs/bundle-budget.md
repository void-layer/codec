# Bundle Budget — @void-layer/codec v0.1.0 (Phase 2)

> Architecture: B-v — Brotli lives in the JS shim (`dist/index.js`), NOT in the WASM.
> The WASM exposes only canonical encode/decode + receiptHash.
> Gzip is the gated metric (spec §3).

| Component | Bytes | Cap | Margin |
|-----------|-------|-----|--------|
| `void_layer_codec_bg.wasm` raw | 180,042 | — | — |
| `void_layer_codec_bg.wasm` gzip | 78,060 | 81,920 (80 KB) | ~4.7% |
| Package tarball (`pkg/` + `dist/`) | 92,160 | 204,800 (200 KB) | ~55% |

> Measured post fix-batch-4 (2026-05-22). gzip figure uses `gzip -c` (the
> `scripts/assert-size.sh` gate method); `gzip -9` yields ~77,283 bytes.

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

| Gate | Cap |
|------|-----|
| WASM gzip | 81,920 bytes (80 KB) |
| Package tarball | 204,800 bytes (200 KB) |
