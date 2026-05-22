// Dictionary substitution + chain/currency dict encoding.
// Mirrors applyDict from app-dict.ts and the chain-dict / CURRENCY_DICT schemes.

use crate::dict::{app::APP_DICT, chain::CHAIN_DICT};
use crate::error::CodecError;
use crate::varint::write_varint;

/// Apply app-level dictionary substitution (mirrors applyDict from app-dict.ts).
/// Replaces known string patterns with 1-byte control codes.
/// Longest match first — iterate entries in length-descending order.
///
/// Returns `Err(CodecError::CompressionFailed)` if the input contains any raw
/// byte equal to an actual dictionary code value. Such bytes would be
/// misinterpreted by `reverse_dict` as dictionary codes on decode, producing a
/// different value. Only the exact `APP_DICT` code values are reserved —
/// non-code control characters such as LF (0x0A) pass through unchanged so
/// multi-line `notes` encode correctly (matches the TS reference).
pub(super) fn apply_dict(input: &str) -> Result<Vec<u8>, CodecError> {
    // Reject only bytes equal to an actual dict code (derived from APP_DICT).
    let is_dict_code = |b: u8| APP_DICT.values().any(|&code| code == b);
    if let Some(c) = input.chars().find(|&c| (c as u32) < 0x100 && is_dict_code(c as u8)) {
        return Err(CodecError::CompressionFailed(format!(
            "field value contains reserved dictionary code byte: 0x{:02x}",
            c as u8
        )));
    }

    // Sorted entries by key length descending (mirrors DICT_ENTRIES order in TS)
    // APP_DICT is a phf map; we must apply longest-match-first manually.
    let mut entries: Vec<(&str, u8)> = APP_DICT.entries().map(|(&k, &v)| (k, v)).collect();
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut text = input.to_string();
    for (pattern, code) in &entries {
        text = text.replace(pattern, &(String::from(char::from(*code))));
    }
    Ok(text.into_bytes())
}

/// Encode chain ID per chain-dict encoding scheme:
///   0x00 <code>   — known chain (dict lookup, 2 bytes)
///   0x01 <varint> — unknown chain (raw varint, 2+ bytes)
pub(super) fn encode_chain_id(network_id: u32) -> Vec<u8> {
    if let Some(&code) = CHAIN_DICT.get(&network_id) {
        vec![0x00, code]
    } else {
        let mut buf = vec![0x01];
        write_varint(network_id as u64, &mut buf);
        buf
    }
}

/// Currency symbol → dict code (mirrors CURRENCY_DICT in tlv-map.ts). Static: zero per-call alloc.
static CURRENCY_SYMBOL_TO_CODE: &[(&str, u8)] = &[
    ("USDC", 1),
    ("USDT", 2),
    ("DAI", 3),
    ("ETH", 4),
    ("WETH", 5),
    ("MATIC", 6),
    ("POL", 7),
    ("WBTC", 8),
    ("USDC.E", 9),
    ("EURC", 10),
    ("USDT0", 11),
];

/// Encode currency per spec §5.1:
///   0x00 <code>  — dict known currency
///   0x01 <utf8>  — raw UTF-8
pub(super) fn encode_currency(currency: &str) -> Vec<u8> {
    let upper = currency.to_uppercase();
    if let Some(&(_, code)) = CURRENCY_SYMBOL_TO_CODE
        .iter()
        .find(|&&(k, _)| k == upper.as_str())
    {
        vec![0x00, code]
    } else {
        let mut val = vec![0x01];
        val.extend_from_slice(currency.as_bytes());
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            matches!(err, crate::error::CodecError::CompressionFailed(_)),
            "expected CompressionFailed for control byte 0x06, got {err:?}"
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
                matches!(err, crate::error::CodecError::CompressionFailed(_)),
                "expected CompressionFailed for dict code 0x{code:02x}, got {err:?}"
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
            matches!(err, crate::error::CodecError::CompressionFailed(_)),
            "expected CompressionFailed for TAB (0x09), got {err:?}"
        );
    }

    /// CR (0x0D) IS a dict code ("development") — must be rejected.
    #[test]
    fn apply_dict_rejects_cr() {
        let err = apply_dict("line\rwrap").unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::CompressionFailed(_)),
            "expected CompressionFailed for CR (0x0D), got {err:?}"
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
            matches!(err, crate::error::CodecError::CompressionFailed(_)),
            "expected CompressionFailed for 0x06, got {err:?}"
        );
    }
}
