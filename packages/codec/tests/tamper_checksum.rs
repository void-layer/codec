//! G-25: programmatic tamper — flip one byte, decode → Err(ChecksumMismatch)

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{CodecError, decode_invoice_canonical, encode_invoice_canonical};

#[test]
fn g25_tamper_total_tlv_errors_checksum() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 24 {
            bytes[value_start] ^= 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail after tamper");
    assert!(
        matches!(err, CodecError::ChecksumMismatch),
        "tampered TLV_TOTAL must produce ChecksumMismatch, got {err:?}"
    );
}

#[test]
fn g25_tamper_from_wallet_tlv_errors_checksum() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 10 {
            bytes[value_start] ^= 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail after tamper");
    assert!(
        matches!(err, CodecError::ChecksumMismatch),
        "tampered TLV_FROM_WALLET must produce ChecksumMismatch, got {err:?}"
    );
}

#[test]
fn g25_tamper_salt_tlv_errors_checksum() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = read_varint_from(&bytes, i + 1);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 20 {
            bytes[value_start + 8] ^= 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail after tamper");
    assert!(
        matches!(err, CodecError::ChecksumMismatch),
        "tampered TLV_SALT must produce ChecksumMismatch, got {err:?}"
    );
}
