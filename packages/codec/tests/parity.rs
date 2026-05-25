//! Golden-vector parity test — Rust surface (T-P2-13).
//!
//! Canonical only: Rust has no wire encoder (Brotli lives in the JS shim per B-v C3).
//! Wire parity is covered by tests/parity.test.ts on the TS surface.
//!
//! Reads vectors/v4-codec.json and asserts:
//!   - Non-malformed: encode → canonical_hex matches; decode canonical_hex → decoded payload matches.
//!   - Malformed decode-input (canonical_hex): decode → expected CodecError variant.
//!   - Malformed encode-input (over-u256): encode → CodecError::InvalidAmount.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::{from_hex, load_vectors, to_hex, to_invoice};
use void_layer_codec::{CodecError, decode_invoice_canonical, encode_invoice_canonical};

// ---------------------------------------------------------------------------
// Non-malformed vectors — canonical encode + decode (both directions)
// ---------------------------------------------------------------------------

#[test]
fn parity_canonical_encode_all_non_malformed() {
    let file = load_vectors();
    let mut failures: Vec<String> = Vec::new();

    for v in &file.vectors {
        if v.roundtrip != Some(true) {
            continue;
        }
        let decoded = v
            .decoded
            .as_ref()
            .expect("non-malformed vector has decoded");
        let canonical_hex = v
            .canonical_hex
            .as_deref()
            .expect("non-malformed vector has canonical_hex");

        let invoice = to_invoice(decoded);
        match encode_invoice_canonical(&invoice) {
            Ok(bytes) => {
                let actual = to_hex(&bytes);
                if actual != canonical_hex {
                    failures.push(format!(
                        "ENCODE MISMATCH vector={}\n  expected: {}\n  actual:   {}",
                        v.name, canonical_hex, actual
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("ENCODE ERROR vector={}: {e:?}", v.name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Canonical encode parity failures:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn parity_canonical_decode_all_non_malformed() {
    let file = load_vectors();
    let mut failures: Vec<String> = Vec::new();

    for v in &file.vectors {
        if v.roundtrip != Some(true) {
            continue;
        }
        let expected_decoded = v
            .decoded
            .as_ref()
            .expect("non-malformed vector has decoded");
        let canonical_hex = v
            .canonical_hex
            .as_deref()
            .expect("non-malformed vector has canonical_hex");

        let bytes = from_hex(canonical_hex);
        match decode_invoice_canonical(&bytes) {
            Ok(actual) => {
                let expected = to_invoice(expected_decoded);
                if actual != expected {
                    failures.push(format!(
                        "DECODE MISMATCH vector={}\n  expected: {expected:?}\n  actual:   {actual:?}",
                        v.name
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("DECODE ERROR vector={}: {e:?}", v.name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Canonical decode parity failures:\n{}",
        failures.join("\n\n")
    );
}

// ---------------------------------------------------------------------------
// Malformed decode-input vectors — canonical_hex → expected CodecError variant
// ---------------------------------------------------------------------------

#[test]
fn parity_malformed_varint_overflow() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-varint-overflow")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::VarintOverflow(_)),
        "expected VarintOverflow, got {err:?}"
    );
}

#[test]
fn parity_malformed_checksum_mismatch() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-checksum-mismatch")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::ChecksumMismatch),
        "expected ChecksumMismatch, got {err:?}"
    );
}

#[test]
fn parity_malformed_oversize() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-oversize")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

#[test]
fn parity_malformed_bad_magic() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-bad-magic")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::BadMagic),
        "expected BadMagic, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Malformed encode-input vector — bigint-amount-over-u256 → InvalidAmount
// ---------------------------------------------------------------------------

#[test]
fn parity_malformed_encode_input_over_u256() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "bigint-amount-over-u256")
        .expect("vector must exist");

    let decoded = v.decoded.as_ref().expect("encode-input vector has decoded");
    let invoice = to_invoice(decoded);
    let err = encode_invoice_canonical(&invoice).expect_err("must fail");
    assert!(
        matches!(err, CodecError::InvalidAmount(_)),
        "expected InvalidAmount, got {err:?}"
    );
}
