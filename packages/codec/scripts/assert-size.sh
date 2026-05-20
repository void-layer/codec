#!/usr/bin/env bash
set -euo pipefail
WASM_PATH="${WASM_PATH:-pkg/void_layer_codec_bg.wasm}"
MAX_WASM_GZIP_BYTES=81920          # 80 KB GZIPPED — spec §3 (B-v)
MAX_PACKAGE_BYTES=204800           # 200 KB tarball
gzip_wasm=$(gzip -c "$WASM_PATH" | wc -c)
echo "WASM gzip: ${gzip_wasm} bytes (cap: ${MAX_WASM_GZIP_BYTES})"
[[ "$gzip_wasm" -le "$MAX_WASM_GZIP_BYTES" ]] || { echo "FAIL: wasm gzip exceeds cap"; exit 1; }
actual_pkg=$(tar czf - pkg/ dist/ | wc -c)
echo "Package tarball: ${actual_pkg} bytes (cap: ${MAX_PACKAGE_BYTES})"
[[ "$actual_pkg" -le "$MAX_PACKAGE_BYTES" ]] || { echo "FAIL: package exceeds cap"; exit 1; }
echo "OK"
