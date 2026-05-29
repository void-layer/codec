//! G-21, G-22, G-23, G-24, G-26, G-28, G-34: structural decode errors
//! (truncated fields, count mismatch, compressed flag, missing zeros byte)

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{CodecError, decode_invoice_canonical, encode_invoice_canonical};

// ---------------------------------------------------------------------------
// G-21: TLV_SALT present but < 16 bytes → Err(ChecksumMismatch or Truncated)
// ---------------------------------------------------------------------------

#[test]
fn g21_salt_shorter_than_16_bytes_errors_checksum() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let length_pos = i + 1;
        let (length, varint_n) = read_varint_from(&bytes, length_pos);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 20 {
            assert_eq!(varint_n, 1, "salt length must be single varint byte");
            bytes[length_pos] = 8;
            let mut rebuilt: Vec<u8> = bytes[..value_start].to_vec();
            rebuilt.extend_from_slice(&bytes[value_start..value_start + 8]);
            rebuilt.extend_from_slice(&bytes[value_end..]);
            bytes = rebuilt;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::Truncated { .. }
        ),
        "salt < 16 bytes must error with ChecksumMismatch or Truncated, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-22: TLV_ISSUED_AT < 4 bytes → Err(Truncated)
// ---------------------------------------------------------------------------

#[test]
fn g22_issued_at_shorter_than_4_bytes_errors_truncated() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let length_pos = i + 1;
        let (length, varint_n) = read_varint_from(&bytes, length_pos);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 4 {
            assert_eq!(length, 4, "issued_at TLV must be 4 bytes");
            let mut rebuilt: Vec<u8> = bytes[..length_pos].to_vec();
            rebuilt.push(2);
            rebuilt.extend_from_slice(&bytes[value_start..value_start + 2]);
            rebuilt.extend_from_slice(&bytes[value_end..]);
            bytes = rebuilt;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::Truncated { .. } | CodecError::ChecksumMismatch
        ),
        "issued_at < 4 bytes must error Truncated or ChecksumMismatch, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-23: TLV_DECIMALS empty value → Err(Truncated)
// ---------------------------------------------------------------------------

#[test]
fn g23_decimals_empty_value_errors_truncated() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let length_pos = i + 1;
        let (length, varint_n) = read_varint_from(&bytes, length_pos);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 8 {
            assert_eq!(length, 1, "decimals TLV must be 1 byte");
            let mut rebuilt: Vec<u8> = bytes[..length_pos].to_vec();
            rebuilt.push(0);
            rebuilt.extend_from_slice(&bytes[value_end..]);
            bytes = rebuilt;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::Truncated { .. } | CodecError::ChecksumMismatch
        ),
        "empty decimals TLV must error Truncated or ChecksumMismatch, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-24: header count=20, body has 1 record → Err(Truncated or Overflow)
// ---------------------------------------------------------------------------

#[test]
fn g24_count_mismatch_header_20_body_1_errors_truncated() {
    let payload: Vec<u8> = vec![
        0x56, // MAGIC
        0x01, // VERSION
        20,   // COUNT = 20
        // one TLV record: type=0x02 (chain_id), length=2, value=[0x00, 0x01]
        0x02, 0x02, 0x00, 0x01,
    ];

    let err = decode_invoice_canonical(&payload).expect_err("must fail");
    assert!(
        matches!(err, CodecError::Truncated { .. } | CodecError::Overflow(_)),
        "count=20 with 1 record must error Truncated or Overflow, got {err:?}"
    );
    let _ = payload;
}

// ---------------------------------------------------------------------------
// G-26: append one extra TLV byte beyond the stream → Err(Truncated or ChecksumMismatch)
// ---------------------------------------------------------------------------

#[test]
fn g26_extra_trailing_byte_errors() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");
    bytes.push(0xAB);
    bytes[2] += 1;

    let err = decode_invoice_canonical(&bytes).expect_err("must fail with extra byte");
    assert!(
        matches!(
            err,
            CodecError::Truncated { .. } | CodecError::ChecksumMismatch
        ),
        "extra trailing byte must error Truncated or ChecksumMismatch, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-28: COMPRESSED_FLAG byte fed to decode_invoice_canonical → Err(InvalidData)
// ---------------------------------------------------------------------------

#[test]
fn g28_compressed_flag_in_decode_canonical_errors_invalid_data() {
    let payload = vec![0x56u8, 0x81, 0x00];
    let err = decode_invoice_canonical(&payload).expect_err("must fail");
    assert!(
        matches!(err, CodecError::InvalidData(_)),
        "COMPRESSED_FLAG in decode_invoice_canonical must return InvalidData, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// G-34: decode_mantissa([0x00]) — mantissa byte present but no zeros byte → Err(Truncated)
// ---------------------------------------------------------------------------

#[test]
fn g34_decode_mantissa_missing_zeros_byte_errors_truncated() {
    let invoice = minimal_invoice();
    let mut bytes = encode_invoice_canonical(&invoice).expect("encode");

    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let length_pos = i + 1;
        let (length, varint_n) = read_varint_from(&bytes, length_pos);
        let value_start = i + 1 + varint_n;
        let value_end = value_start + length;

        if tlv_type == 24 {
            let mut rebuilt: Vec<u8> = bytes[..length_pos].to_vec();
            rebuilt.push(1);
            rebuilt.push(0x00);
            rebuilt.extend_from_slice(&bytes[value_end..]);
            bytes = rebuilt;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(
            err,
            CodecError::ChecksumMismatch | CodecError::Truncated { .. }
        ),
        "missing zeros byte must error ChecksumMismatch or Truncated, got {err:?}"
    );
}
