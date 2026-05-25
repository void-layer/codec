//! Golden-vector malformed parity — Rust surface (T-P2-13).
//!
//! Asserts malformed vectors produce the expected CodecError variant on decode or encode.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::{from_hex, load_vectors, to_invoice};
use void_layer_codec::{CodecError, decode_invoice_canonical, encode_invoice_canonical};

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
