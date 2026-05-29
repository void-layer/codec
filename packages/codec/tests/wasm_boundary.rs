// WASM boundary test for receiptHash (compute_content_hash Rust implementation).
// Task T-P2-9b-fix — calls Rust directly, no /pkg/ re-import (2026-05-20).
//
// Tests:
//   - Determinism: same input → identical digest on two calls
//   - Distinctness: distinct inputs → distinct digests (guards constant-return regression)
//   - 32-byte length (explicit assertion for documentation)

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_node_experimental);

/// Determinism: same canonical bytes → identical digest on two independent calls.
#[wasm_bindgen_test]
fn compute_content_hash_is_deterministic() {
    // Hand-crafted canonical TLV: tag=0x01, length=0x03, value=[0xAA, 0xBB, 0xCC]
    let canonical: &[u8] = &[0x01, 0x03, 0xAA, 0xBB, 0xCC];
    let first = void_layer_codec::compute_content_hash(canonical);
    let second = void_layer_codec::compute_content_hash(canonical);
    assert_eq!(
        first, second,
        "compute_content_hash must be deterministic — same input must yield identical digest"
    );
}

/// Non-empty input and empty input must produce different digests
/// (guards against a constant-return regression).
#[wasm_bindgen_test]
fn compute_content_hash_distinct_for_distinct_inputs() {
    let a = void_layer_codec::compute_content_hash(&[0x01, 0x03, 0xAA, 0xBB, 0xCC]);
    let b = void_layer_codec::compute_content_hash(&[]);
    assert_ne!(
        a, b,
        "compute_content_hash of distinct inputs must differ (non-constant function)"
    );
}

/// 32-byte digest length (compile-time [u8; 32], explicit runtime assertion for documentation).
#[wasm_bindgen_test]
fn compute_content_hash_returns_32_bytes() {
    let canonical: &[u8] = &[0x01, 0x03, 0xAA, 0xBB, 0xCC];
    let digest = void_layer_codec::compute_content_hash(canonical);
    assert_eq!(
        digest.len(),
        32,
        "compute_content_hash must return exactly 32 bytes"
    );
}
