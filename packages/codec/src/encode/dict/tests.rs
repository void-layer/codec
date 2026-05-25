//! Tests for encode::dict.
use super::*;
use crate::dict::app::APP_DICT;

#[test]
fn encode_chain_id_known_ethereum() {
    let b = encode_chain_id(1);
    assert_eq!(b, vec![0x00, 0x01]);
}

#[test]
fn encode_chain_id_unknown() {
    let b = encode_chain_id(999999);
    assert_eq!(b[0], 0x01, "unknown chain prefix must be 0x01");
    assert!(b.len() > 1, "must include varint after prefix");
}

#[test]
fn encode_currency_known_usdc() {
    let b = encode_currency("USDC");
    assert_eq!(b, vec![0x00, 0x01]);
}

#[test]
fn encode_currency_unknown() {
    let b = encode_currency("XYZ");
    assert_eq!(b[0], 0x01);
    assert_eq!(&b[1..], b"XYZ");
}

#[test]
fn apply_dict_substitutes_pattern() {
    let result = apply_dict("Invoice total").unwrap();
    // "Invoice" → 0x06
    assert_eq!(result[0], 0x06);
}

#[test]
fn apply_dict_no_match_passthrough() {
    let result = apply_dict("Hello world").unwrap();
    assert_eq!(result, b"Hello world");
}

// --- R3: dict control-byte injection ---

/// A field value containing raw byte 0x06 ("Invoice" dict code) must be
/// rejected. Old code let it pass through apply_dict unchanged, then
/// reverse_dict on decode expanded it: "\x06Acme" → "InvoiceAcme".
#[test]
fn r3_control_byte_0x06_in_field_value_errors() {
    let hostile = "\x06Acme"; // 0x06 = dict code for "Invoice"
    let err = apply_dict(hostile).unwrap_err();
    assert!(
        matches!(err, crate::error::CodecError::InvalidData(_)),
        "expected InvalidData for control byte 0x06, got {err:?}"
    );
}

/// Verify that a value with no control bytes still round-trips correctly
/// (regression guard — apply_dict must not break clean input).
#[test]
fn r3_normal_value_still_roundtrips() {
    let normal = "Acme Corp";
    let encoded = apply_dict(normal).unwrap();
    // Must not contain any raw control bytes in the dict range.
    assert!(
        !encoded.iter().any(|&b| matches!(b, 0x02..=0x1F)),
        "clean input must not produce reserved control bytes"
    );
}

/// Every actual `APP_DICT` code value must be rejected as a raw byte.
#[test]
fn r3_all_dict_code_bytes_rejected() {
    for &code in APP_DICT.values() {
        let hostile = format!("{}", char::from(code));
        let err = apply_dict(&hostile).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidData(_)),
            "expected InvalidData for dict code 0x{code:02x}, got {err:?}"
        );
    }
}

// --- #4: exact-set rejection (match TS reference) ---

/// LF (0x0A) is NOT a dict code — multi-line `notes` must encode fine.
#[test]
fn apply_dict_accepts_lf_multiline_notes() {
    let multiline = "Line one\nLine two\nLine three";
    let encoded = apply_dict(multiline).expect("LF must be accepted");
    assert!(
        encoded.contains(&0x0A),
        "LF byte must survive into the encoded output"
    );
}

/// TAB (0x09) IS a dict code (".com") — must be rejected.
#[test]
fn apply_dict_rejects_tab() {
    let err = apply_dict("col1\tcol2").unwrap_err();
    assert!(
        matches!(err, crate::error::CodecError::InvalidData(_)),
        "expected InvalidData for TAB (0x09), got {err:?}"
    );
}

/// CR (0x0D) IS a dict code ("development") — must be rejected.
#[test]
fn apply_dict_rejects_cr() {
    let err = apply_dict("line\rwrap").unwrap_err();
    assert!(
        matches!(err, crate::error::CodecError::InvalidData(_)),
        "expected InvalidData for CR (0x0D), got {err:?}"
    );
}

/// FIX #1 (encode half): non-ASCII text must pass `apply_dict` and emit
/// its exact UTF-8 bytes — `reverse_dict` round-trips it (see decode tests).
#[test]
fn apply_dict_preserves_non_ascii_utf8() {
    let original = "Café 日本語 ñ";
    let encoded = apply_dict(original).expect("non-ASCII must be accepted");
    assert_eq!(
        encoded,
        original.as_bytes(),
        "non-ASCII input must emit its UTF-8 bytes unchanged"
    );
}

/// A raw 0x06 byte ("Invoice" dict code) must still be rejected.
#[test]
fn apply_dict_rejects_raw_0x06() {
    let err = apply_dict("\x06Acme").unwrap_err();
    assert!(
        matches!(err, crate::error::CodecError::InvalidData(_)),
        "expected InvalidData for 0x06, got {err:?}"
    );
}
