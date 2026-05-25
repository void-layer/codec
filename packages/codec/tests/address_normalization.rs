//! G-12, G-13: hex/EIP-55 address normalization

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{decode_invoice_canonical, encode_invoice_canonical};

// ---------------------------------------------------------------------------
// G-12: hex_decode_salt with uppercase hex and "0x"-prefixed salt → both Ok
// ---------------------------------------------------------------------------

#[test]
fn g12_hex_decode_salt_uppercase_hex_ok() {
    let mut invoice = minimal_invoice();
    invoice.salt = "DEADBEEFDEADBEEFDEADBEEFDEADBEEF".to_string();
    let result = encode_invoice_canonical(&invoice);
    assert!(
        result.is_ok(),
        "uppercase salt hex must encode without error"
    );
}

#[test]
fn g12_hex_decode_salt_0x_prefixed_ok() {
    let mut invoice = minimal_invoice();
    invoice.salt = "0xdeadbeefdeadbeefdeadbeefdeadbeef".to_string();
    let result = encode_invoice_canonical(&invoice);
    assert!(
        result.is_ok(),
        "0x-prefixed salt hex must encode without error"
    );
}

#[test]
fn g12_uppercase_and_0x_prefixed_decode_same_bytes() {
    let mut inv_upper = minimal_invoice();
    inv_upper.salt = "DEADBEEFDEADBEEFDEADBEEFDEADBEEF".to_string();
    let mut inv_lower = minimal_invoice();
    inv_lower.salt = "deadbeefdeadbeefdeadbeefdeadbeef".to_string();
    let mut inv_0x = minimal_invoice();
    inv_0x.salt = "0xdeadbeefdeadbeefdeadbeefdeadbeef".to_string();

    let bytes_upper = encode_invoice_canonical(&inv_upper).unwrap();
    let bytes_lower = encode_invoice_canonical(&inv_lower).unwrap();
    let bytes_0x = encode_invoice_canonical(&inv_0x).unwrap();

    assert_eq!(
        to_hex(&bytes_upper),
        to_hex(&bytes_lower),
        "uppercase and lowercase salt must produce same canonical bytes"
    );
    assert_eq!(
        to_hex(&bytes_lower),
        to_hex(&bytes_0x),
        "0x-prefixed and lowercase salt must produce same canonical bytes"
    );
}

// ---------------------------------------------------------------------------
// G-13: address_to_bytes mixed-case EIP-55 checksum address → roundtrip, output lowercased
// ---------------------------------------------------------------------------

#[test]
fn g13_eip55_checksum_address_roundtrips_lowercased() {
    let eip55 = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
    let expected_lower = "0xd8da6bf26964af9d7eed9e03e53415d37aa96045";

    let mut invoice = minimal_invoice();
    invoice.from.wallet_address = eip55.to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode EIP-55 address");
    let decoded = decode_invoice_canonical(&bytes).expect("decode EIP-55 address");
    assert_eq!(
        decoded.from.wallet_address, expected_lower,
        "EIP-55 mixed-case address must decode as lowercased"
    );
}
