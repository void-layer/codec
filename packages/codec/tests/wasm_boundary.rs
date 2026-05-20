// WASM boundary test for receiptHash (compute_content_hash JS export).
// Task T-P2-9b — Phase 2B hotfix (2026-05-20).
//
// Tests:
//   - 32-byte digest length
//   - Determinism: same input → identical output across two calls

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_node_experimental);

#[wasm_bindgen(module = "/pkg/void_layer_codec.js")]
extern "C" {
    #[wasm_bindgen(js_name = receiptHash, catch)]
    fn receipt_hash_js(bytes: &[u8]) -> Result<Vec<u8>, JsValue>;
}

/// 32-byte digest: receiptHash over a hand-crafted canonical TLV fixture
/// must return exactly 32 bytes.
#[wasm_bindgen_test]
fn receipt_hash_returns_32_bytes() {
    // Hand-crafted canonical TLV: tag=0x01, length=0x03, value=[0xAA, 0xBB, 0xCC]
    let canonical: &[u8] = &[0x01, 0x03, 0xAA, 0xBB, 0xCC];
    let digest = receipt_hash_js(canonical).expect("receiptHash must not trap");
    assert_eq!(digest.len(), 32, "receiptHash must return exactly 32 bytes");
}

/// Determinism: same canonical bytes → identical digest on two independent calls.
#[wasm_bindgen_test]
fn receipt_hash_is_deterministic() {
    let canonical: &[u8] = &[0x01, 0x03, 0xAA, 0xBB, 0xCC];
    let first = receipt_hash_js(canonical).expect("first call must not trap");
    let second = receipt_hash_js(canonical).expect("second call must not trap");
    assert_eq!(
        first, second,
        "receiptHash must be deterministic — same input must yield identical digest"
    );
}

/// Non-empty input and empty input must produce different digests
/// (guards against a constant-return regression).
#[wasm_bindgen_test]
fn receipt_hash_distinct_for_distinct_inputs() {
    let a = receipt_hash_js(&[0x01, 0x03, 0xAA, 0xBB, 0xCC]).expect("call A must not trap");
    let b = receipt_hash_js(&[]).expect("call B (empty) must not trap");
    assert_ne!(
        a, b,
        "receiptHash of distinct inputs must differ (non-constant function)"
    );
}
