// Regression test for D-B11: BigInt WASM<->JS boundary failure modes.
// Promoted from src/bin/bigint_probe.rs after spike (T-P2-0b, 2026-05-19).
//
// ACTUAL findings — D-B11 AMENDED (spec §4.8 prediction was wrong):
// - Config A (default): serde-wasm-bindgen 0.6 returns Err for ANY u64 value.
//   Error: "<value> can't be represented as a JavaScript number". Does NOT silently
//   truncate — it hard-errors. Even safe values (2^53 = 9007199254740992) return Err.
// - Config B (.serialize_large_number_types_as_bigints(true)): u64 becomes JS BigInt. Exact.
// - Codec decision: amounts stored as decimal strings — safe under both configs.

use js_sys;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_node_experimental);

#[derive(Serialize, Clone)]
struct Probe {
    u64_max: u64,
    above_2_53: u64,
    safe_53: u64,
    string_amount: String,
}

impl Probe {
    fn new() -> Self {
        Probe {
            u64_max: u64::MAX,
            above_2_53: 9_007_199_254_740_993_u64, // 2^53 + 1
            safe_53: 9_007_199_254_740_992_u64,    // 2^53 exact
            string_amount:
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
                    .to_string(), // uint256::MAX decimal string
        }
    }
}

// --- Config A: default serializer ---
// ACTUAL behavior (serde-wasm-bindgen 0.6): returns Err for ANY u64 value.
// Error message: "<value> can't be represented as a JavaScript number".
// This is stricter than spec §4.8 predicted (silent truncation). D-B11 AMENDED.

#[wasm_bindgen_test]
fn config_a_safe_u64_returns_err() {
    let serializer = serde_wasm_bindgen::Serializer::new();
    let result = Probe::new().safe_53.serialize(&serializer);
    // serde-wasm-bindgen 0.6 default rejects ALL u64, even values that fit in f64 mantissa
    assert!(
        result.is_err(),
        "Config A: u64 safe_53 (2^53) must return Err — default serializer rejects all u64"
    );
}

#[wasm_bindgen_test]
fn config_a_above_2_53_returns_err() {
    let serializer = serde_wasm_bindgen::Serializer::new();
    let result = Probe::new().above_2_53.serialize(&serializer);
    assert!(
        result.is_err(),
        "Config A: u64 above 2^53 must return Err — D-B11 failure mode (harder than truncation)"
    );
}

#[wasm_bindgen_test]
fn config_a_u64_max_returns_err() {
    let serializer = serde_wasm_bindgen::Serializer::new();
    let result = Probe::new().u64_max.serialize(&serializer);
    assert!(
        result.is_err(),
        "Config A: u64::MAX must return Err — confirms D-B11 amended failure mode"
    );
}

// --- Config B: BigInt-enabled serializer ---

#[wasm_bindgen_test]
fn config_b_above_2_53_is_bigint() {
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_large_number_types_as_bigints(true);
    let js_val = Probe::new().above_2_53.serialize(&serializer).unwrap();
    assert!(
        js_val.is_bigint(),
        "Config B: u64 above 2^53 must serialize as JS BigInt"
    );
}

#[wasm_bindgen_test]
fn config_b_u64_max_is_exact_bigint() {
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_large_number_types_as_bigints(true);
    let js_val = Probe::new().u64_max.serialize(&serializer).unwrap();
    assert!(js_val.is_bigint(), "Config B: u64::MAX must serialize as JS BigInt");
    let bigint = js_sys::BigInt::from(js_val);
    let bigint_str = String::from(bigint.to_string(10).unwrap());
    assert_eq!(
        bigint_str, "18446744073709551615",
        "Config B: u64::MAX BigInt value must be exact"
    );
}

#[wasm_bindgen_test]
fn config_b_safe_53_is_still_bigint() {
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_large_number_types_as_bigints(true);
    let js_val = Probe::new().safe_53.serialize(&serializer).unwrap();
    assert!(
        js_val.is_bigint(),
        "Config B: even safe u64 becomes JS BigInt (uniform serialization)"
    );
}

// --- String amount path (safe under both configs) ---

#[wasm_bindgen_test]
fn string_amount_survives_config_a() {
    let serializer = serde_wasm_bindgen::Serializer::new();
    let js_val = Probe::new().string_amount.serialize(&serializer).unwrap();
    let back = js_val.as_string().expect("String amount must round-trip as JS string");
    assert_eq!(
        back,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
        "String amount (uint256::MAX) must survive Config A unchanged"
    );
}

#[wasm_bindgen_test]
fn string_amount_survives_config_b() {
    let serializer = serde_wasm_bindgen::Serializer::new()
        .serialize_large_number_types_as_bigints(true);
    let js_val = Probe::new().string_amount.serialize(&serializer).unwrap();
    let back = js_val.as_string().expect("String amount must round-trip as JS string under Config B");
    assert_eq!(
        back,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
        "String amount (uint256::MAX) must survive Config B unchanged"
    );
}

// --- Zod-equivalent: JS BigInt(v) accept/reject cases ---
// Mirrors: z.string().refine(v => { try { BigInt(v); return true; } catch { return false; } })

#[wasm_bindgen_test]
fn zod_refine_accepts_valid_integer_strings() {
    let valid = [
        "0",
        "1",
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
    ];
    for s in &valid {
        let result = js_sys::BigInt::new(&JsValue::from_str(s));
        assert!(result.is_ok(), "Zod refine: '{}' must be accepted by BigInt(v)", s);
    }
}

#[wasm_bindgen_test]
fn zod_refine_rejects_invalid_strings() {
    // JS BigInt() throws for scientific notation, non-numeric, decimals
    let invalid = ["1e18", "abc", "1.5"];
    for s in &invalid {
        let result = js_sys::BigInt::new(&JsValue::from_str(s));
        assert!(result.is_err(), "Zod refine: '{}' must be rejected by BigInt(v)", s);
    }
}
