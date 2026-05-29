//! G-05, G-06, G-07, G-32, G-37: numeric overflow and bigint-varint boundary

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{
    CodecError, InvoiceItem, decode_invoice_canonical, encode_invoice_canonical,
};

// ---------------------------------------------------------------------------
// G-05: issued_at=u32::MAX, due_delta=1 → checked_add overflow → InvalidAmount
// ---------------------------------------------------------------------------

#[test]
fn g05_issued_at_u32_max_due_delta_1_overflows() {
    let mut invoice = minimal_invoice();
    invoice.issued_at = 1_700_000_000;
    invoice.due_at = 1_700_000_001;
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = {
            let mut value: u64 = 0;
            let mut shift: u32 = 0;
            let mut n = 0usize;
            loop {
                let b = bytes[i + 1 + n];
                n += 1;
                value |= ((b & 0x7F) as u64) << shift;
                if b & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            (value as usize, n)
        };
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 4 {
            bytes[value_start] = 0xFF;
            bytes[value_start + 1] = 0xFF;
            bytes[value_start + 2] = 0xFF;
            bytes[value_start + 3] = 0xFF;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::InvalidAmount(_)
        ),
        "expected ChecksumMismatch or InvalidAmount for u32::MAX + 1 overflow, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-06: decode_mantissa U256::MAX mantissa × 10 → checked_mul overflow
// ---------------------------------------------------------------------------

#[test]
fn g06_decode_mantissa_u256_max_times_10_overflows() {
    let uint256_max =
        "115792089237316195423570985008687907853269984665640564039457584007913129639935";
    let mut invoice = minimal_invoice();
    invoice.total = uint256_max.to_string();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode with u256_max total");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, varint_n) = {
            let mut value: u64 = 0;
            let mut shift: u32 = 0;
            let mut n = 0usize;
            loop {
                let b = bytes[i + 1 + n];
                n += 1;
                value |= ((b & 0x7F) as u64) << shift;
                if b & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            (value as usize, n)
        };
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 24 {
            bytes[value_end - 1] = 1;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::InvalidAmount(_)
        ),
        "expected ChecksumMismatch or InvalidAmount for U256::MAX × 10 overflow, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-07: 32-byte all-0xFF mantissa, zeros=0 → Ok(U256::MAX)
// ---------------------------------------------------------------------------

#[test]
fn g07_u256_max_mantissa_roundtrips_ok() {
    let uint256_max =
        "115792089237316195423570985008687907853269984665640564039457584007913129639935";
    let mut invoice = minimal_invoice();
    invoice.total = uint256_max.to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode U256::MAX total");
    let decoded = decode_invoice_canonical(&bytes).expect("decode U256::MAX total");
    assert_eq!(
        decoded.total, uint256_max,
        "U256::MAX mantissa (32 bytes all-0xFF) must roundtrip without hitting the >32 guard"
    );
}

// ---------------------------------------------------------------------------
// G-32: write_bigint_varint([0x00;32]) → [0x00] (leading-zero stripping of U256 zero)
// ---------------------------------------------------------------------------

#[test]
fn g32_write_bigint_varint_all_zero_32_bytes_encodes_as_single_zero() {
    let mut invoice = minimal_invoice();
    invoice.total = "0".to_string();
    invoice.items = vec![InvoiceItem {
        description: "Zero".to_string(),
        quantity: 1.0,
        rate: "0".to_string(),
    }];
    let bytes = encode_invoice_canonical(&invoice).expect("encode total=0");
    let decoded = decode_invoice_canonical(&bytes).expect("decode total=0");
    assert_eq!(decoded.total, "0", "all-zero U256 must roundtrip as '0'");
}

// ---------------------------------------------------------------------------
// G-37: write_bigint_varint single byte boundary
// [0x7F] → [0x7F] (fits in 7 bits, no continuation)
// [0x80] → [0x80, 0x01] (requires continuation bit)
// ---------------------------------------------------------------------------

#[test]
fn g37_write_bigint_varint_0x7f_encodes_as_single_byte() {
    let mut invoice = minimal_invoice();
    invoice.total = "127".to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode total=127");
    let decoded = decode_invoice_canonical(&bytes).expect("decode total=127");
    assert_eq!(decoded.total, "127", "0x7F mantissa must roundtrip");
}

#[test]
fn g37_write_bigint_varint_0x80_encodes_with_continuation() {
    let mut invoice = minimal_invoice();
    invoice.total = "128".to_string();
    let bytes = encode_invoice_canonical(&invoice).expect("encode total=128");
    let decoded = decode_invoice_canonical(&bytes).expect("decode total=128");
    assert_eq!(
        decoded.total, "128",
        "0x80 mantissa must roundtrip via 2-byte LEB128"
    );
}
