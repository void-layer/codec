//! G-20: invalid UTF-8 in text fields → Err(ChecksumMismatch or InvalidData)

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{CodecError, decode_invoice_canonical, encode_invoice_canonical};

#[test]
fn g20_invalid_utf8_in_invoice_id_errors() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 22 {
            bytes[value_start] = 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::InvalidData(_)
        ),
        "invalid UTF-8 in invoice_id must error, got {err:?}"
    );
}

#[test]
fn g20_invalid_utf8_in_tax_errors() {
    let mut invoice = minimal_invoice();
    invoice.tax = Some("10".to_string());
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode with tax");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 19 {
            bytes[value_start] = 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::InvalidData(_)
        ),
        "invalid UTF-8 in tax must error, got {err:?}"
    );
}

#[test]
fn g20_invalid_utf8_in_discount_errors() {
    let mut invoice = minimal_invoice();
    invoice.discount = Some("5".to_string());
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode with discount");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 21 {
            bytes[value_start] = 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::InvalidData(_)
        ),
        "invalid UTF-8 in discount must error, got {err:?}"
    );
}
