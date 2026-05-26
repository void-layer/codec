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

/// Y1 forward-compat: a full invoice wire that embeds unknown odd TLV tag 39 must decode
/// successfully. The domain separator is computed over all TLV bytes including the odd-tag
/// bytes (excluding type 31). Tag 39 is silently ignored; all invoice fields are intact.
/// Decision: codec-bolt12-odd-even-forward-compat (P1 fix, re-derived 2026-05-26).
#[test]
fn parity_y1_odd_tag_in_full_invoice_decodes_successfully() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "decode_unknown_odd_tag_in_full_invoice")
        .expect("vector must exist");

    let canonical_hex = v.canonical_hex.as_deref().expect("has canonical_hex");
    let bytes = from_hex(canonical_hex);

    let invoice = decode_invoice_canonical(&bytes)
        .expect("unknown odd tag 39 must be silently ignored — decode must succeed");

    let expected = to_invoice(v.decoded.as_ref().expect("has decoded"));
    assert_eq!(
        invoice, expected,
        "decoded invoice must match vector decoded block"
    );
}
