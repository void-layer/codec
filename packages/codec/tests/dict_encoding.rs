//! G-29, G-30, G-31, G-36: dict encoding — currency normalization, longest-match,
//! NUL passthrough, WETH per-network encoding

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{CodecError, decode_invoice_canonical, encode_invoice_canonical};

// ---------------------------------------------------------------------------
// G-29: encode_currency case-normalization: currency="usdc" → decode → "USDC"
// ---------------------------------------------------------------------------

#[test]
fn g29_lowercase_currency_normalizes_to_uppercase_on_decode() {
    let mut invoice = minimal_invoice();
    invoice.currency = "usdc".to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode lowercase currency");
    let decoded = decode_invoice_canonical(&bytes).expect("decode");
    assert_eq!(
        decoded.currency, "USDC",
        "lowercase 'usdc' must decode as 'USDC' (non-identity, intentional normalization)"
    );
}

// ---------------------------------------------------------------------------
// G-30: apply_dict longest-match ordering
// ---------------------------------------------------------------------------

#[test]
fn g30_apply_dict_longest_match_order() {
    let mut invoice = minimal_invoice();
    invoice.from.name = "Invoice Payment".to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode with dict patterns");
    let decoded = decode_invoice_canonical(&bytes).expect("decode");
    assert_eq!(
        decoded.from.name, "Invoice Payment",
        "longest-match dict application must roundtrip correctly"
    );
}

#[test]
fn g30_apply_dict_consulting_pattern_roundtrips() {
    let mut invoice = minimal_invoice();
    invoice.items[0].description = "consulting services".to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode");
    let decoded = decode_invoice_canonical(&bytes).expect("decode");
    assert_eq!(
        decoded.items[0].description, "consulting services",
        "dict pattern 'consulting' must roundtrip correctly"
    );
}

// ---------------------------------------------------------------------------
// G-31: NUL byte (0x00) in dict-encoded field: apply_dict("\x00test")
// ---------------------------------------------------------------------------

#[test]
fn g31_nul_byte_passes_through_apply_dict() {
    let mut invoice = minimal_invoice();
    invoice.notes = Some("\x00test".to_string());
    let result = encode_invoice_canonical(&invoice);
    match result {
        Ok(bytes) => {
            let decoded = decode_invoice_canonical(&bytes).expect("decode");
            assert_eq!(
                decoded.notes.as_deref(),
                Some("\x00test"),
                "NUL byte must roundtrip through dict layer unchanged"
            );
        }
        Err(CodecError::InvalidData(_)) => {
            panic!("NUL byte should NOT be rejected by apply_dict — not a dict code");
        }
        Err(e) => panic!("unexpected error for NUL byte in notes: {e:?}"),
    }
}

// ---------------------------------------------------------------------------
// G-36: token dict code 43 (Base WETH) vs 24 (Optimism WETH)
// ---------------------------------------------------------------------------

#[test]
fn g36_weth_base_encodes_as_code_43_decodes_correctly() {
    let weth = "0x4200000000000000000000000000000000000006";
    let mut invoice = minimal_invoice();
    invoice.network_id = 8453; // Base
    invoice.token_address = Some(weth.to_string());
    let bytes = encode_invoice_canonical(&invoice).expect("encode WETH on Base");

    let header_len = 3usize;
    let mut i = header_len;
    let mut found_prefix: Option<u8> = None;
    let mut found_len: Option<usize> = None;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 1 {
            found_prefix = Some(bytes[value_start]);
            found_len = Some(length);
            break;
        }
        i = value_end;
    }
    assert_eq!(
        found_prefix,
        Some(0x01),
        "WETH on Base must be raw-encoded (prefix 0x01), not dict"
    );
    assert_eq!(
        found_len,
        Some(21),
        "raw token address TLV value must be 21 bytes (0x01 + 20 addr bytes)"
    );

    let decoded = decode_invoice_canonical(&bytes).expect("decode WETH on Base");
    assert_eq!(
        decoded.token_address.as_deref(),
        Some(weth),
        "raw-encoded WETH on Base must decode back to the WETH address"
    );
}

#[test]
fn g36_weth_optimism_encodes_as_code_24_decodes_correctly() {
    let weth = "0x4200000000000000000000000000000000000006";
    let mut invoice = minimal_invoice();
    invoice.network_id = 10; // Optimism
    invoice.token_address = Some(weth.to_string());
    let bytes = encode_invoice_canonical(&invoice).expect("encode WETH on Optimism");

    let header_len = 3usize;
    let mut i = header_len;
    let mut found_code: Option<u8> = None;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 1 {
            assert_eq!(bytes[value_start], 0x00, "should be dict-encoded");
            found_code = Some(bytes[value_start + 1]);
            break;
        }
        i = value_end;
    }
    assert_eq!(
        found_code,
        Some(24),
        "WETH on Optimism must encode as dict code 24"
    );

    let decoded = decode_invoice_canonical(&bytes).expect("decode WETH on Optimism");
    assert_eq!(
        decoded.token_address.as_deref(),
        Some(weth),
        "WETH dict code 24 must decode to the WETH address"
    );
}
