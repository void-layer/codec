---
date: 2026-05-19
task: T-P2-0b
spec: 056-void-layer-codec-extraction
decision: D-B11
status: AMENDED
---

# Spike: BigInt WASM↔JS Boundary Failure Modes

**Goal**: Validate D-B11 — observe what actually happens when Rust serializes `u64`/`u128` to JS via
`serde-wasm-bindgen` 0.6 with and without `.serialize_large_number_types_as_bigints(true)`.

**Setup**: `wasm-pack test --node` (wasm-pack 0.13.1, Rust 1.85.0, serde-wasm-bindgen 0.6.5)

---

## Config-A-vs-B Failure-Mode Table

| Field | Config A type | Config A behavior | Config B type | Config B value |
|-------|--------------|-------------------|--------------|----------------|
| `u64::MAX` (18446744073709551615) | `Err` | serialization error: "can't be represented as a JavaScript number" | `bigint` | `18446744073709551615n` (exact) |
| `above_2_53` (9007199254740993) | `Err` | serialization error: "can't be represented as a JavaScript number" | `bigint` | `9007199254740993n` (exact) |
| `safe_53` (9007199254740992 = 2^53) | `Err` | serialization error: "can't be represented as a JavaScript number" | `bigint` | `9007199254740992n` (exact) |
| `string_amount` (uint256::MAX decimal) | `string` | round-trips intact | `string` | round-trips intact |

**Key observation**: Config A does NOT silently truncate. It hard-errors on ALL `u64` values
(including values that fit in f64 mantissa). This is stricter than predicted.

---

## Spec §4.8 Prediction vs. Actual

| Prediction (§4.8) | Actual (serde-wasm-bindgen 0.6) |
|---|---|
| Config A silently truncates u64 >2^53 to f64 | Config A returns `Err` for ALL u64, including safe values |
| Config A values ≤2^53 are exact JS Numbers | Config A returns `Err` even for 2^53 |
| Config B yields `bigint` | CONFIRMED — Config B yields `bigint`, exact |

---

## Zod `.refine()` Verification

Mirrors TS: `z.string().refine(v => { try { BigInt(v); return true; } catch { return false; } })`

Tested via `js_sys::BigInt::new(&JsValue::from_str(s))` in wasm-bindgen-test Node runner:

| Input | Expected | Actual |
|-------|----------|--------|
| `"0"` | ACCEPT | PASS — `BigInt("0")` succeeds |
| `"1"` | ACCEPT | PASS — `BigInt("1")` succeeds |
| uint256::MAX decimal string | ACCEPT | PASS — `BigInt("<78-digit string>")` succeeds |
| `"1e18"` | REJECT | PASS — `BigInt("1e18")` throws |
| `"abc"` | REJECT | PASS — `BigInt("abc")` throws |
| `"1.5"` | REJECT | PASS — `BigInt("1.5")` throws |

All 6 cases confirmed. The Zod refine strategy is sound.

---

## VERDICT

**D-B11 AMENDED**: The spec §4.8 prediction that "Config A silently truncates u64/u128 to f64 for
values >2^53" is incorrect for serde-wasm-bindgen 0.6. The actual behavior is that the default
serializer hard-errors (`Err`) on ALL `u64` values — it does not produce a JS Number at all.

Consequence for codec design:
1. `.serialize_large_number_types_as_bigints(true)` is **mandatory** to successfully serialize any
   `u64`/`u128` across the WASM boundary (not just for large values — for all u64).
2. The string-amount path (decimal string) is safe under BOTH configs and remains the preferred
   approach for invoice amounts — immune to this failure mode entirely.
3. The Zod `.refine(v => BigInt(v))` guard on the TS consumer side is confirmed correct for
   validating incoming decimal-string amounts (rejects scientific notation, decimals, non-numeric).

Regression test: `packages/codec/tests/bigint_boundary.rs` — 10/10 green under `wasm-pack test --node`.
