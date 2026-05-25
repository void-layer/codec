// Reverse dictionary expansion: chain ID, currency, token address, and
// app-level text substitution.

use crate::dict::chain::CHAIN_DICT;
use crate::error::CodecError;
use crate::varint::read_varint;

use super::hex::bytes_to_address;

/// Reverse app-level dictionary substitution (mirrors reverseDict from app-dict.ts).
///
/// Reuses `encode::APP_DICT_ENTRIES` — the single ordered source of truth — so
/// the encode and decode dict tables cannot silently diverge.
pub(super) fn reverse_dict(bytes: &[u8]) -> Result<String, CodecError> {
    // Decode raw bytes as UTF-8 (matches the TS reference's TextDecoder).
    // Dict-code bytes (0x02–0x0F) are valid single-byte UTF-8 and survive as
    // single chars, so the expansion loop below works unchanged.
    let mut text = String::from_utf8(bytes.to_vec())
        .map_err(|_| CodecError::InvalidData("invalid UTF-8 in dict text".to_string()))?;

    // Apply in reverse order (shortest first for reverse) — mirrors TS [...DICT_ENTRIES].reverse()
    for &(pattern, code) in crate::encode::APP_DICT_ENTRIES.iter().rev() {
        text = text.replace(char::from(code), pattern);
    }

    Ok(text)
}

/// Decode chain ID from TLV value bytes:
///   [0x00, code] → dict lookup
///   [0x01, varint...] → raw chain ID
pub(super) fn decode_chain_id(value: &[u8]) -> Result<u32, CodecError> {
    if value.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    let prefix = value[0];
    if prefix == 0x00 {
        if value.len() < 2 {
            return Err(CodecError::Truncated { needed: 2, had: 1 });
        }
        let code = value[1];
        // Reverse lookup: code → chain_id
        let chain_id = CHAIN_DICT
            .entries()
            .find_map(|(&k, &v)| (v == code).then_some(k))
            .ok_or(CodecError::UnknownExtension(code))?;
        Ok(chain_id)
    } else if prefix == 0x01 {
        let (chain_id_u64, _) = read_varint(value, 1)?;
        // Reject chain IDs > u32::MAX instead of silently truncating.
        let chain_id = u32::try_from(chain_id_u64).map_err(|_| {
            CodecError::InvalidAmount(format!("chain ID {chain_id_u64} overflows u32"))
        })?;
        // T6: reject non-canonical encoding — if this chain_id is in the dict,
        // the encoder must have used dict form [0x00, code]. Raw form for a known
        // chain ID means the payload was not produced by the canonical encoder.
        if CHAIN_DICT.contains_key(&chain_id) {
            return Err(CodecError::InvalidData(format!(
                "non-canonical chain encoding: chain {chain_id} must use dict form"
            )));
        }
        Ok(chain_id)
    } else {
        Err(CodecError::UnknownExtension(prefix))
    }
}

/// Decode currency from TLV value bytes:
///   [0x00, code] → dict lookup
///   [0x01, utf8...] → raw string
pub(super) fn decode_currency(value: &[u8]) -> Result<String, CodecError> {
    if value.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    if value[0] == 0x00 {
        if value.len() < 2 {
            return Err(CodecError::Truncated { needed: 2, had: 1 });
        }
        let code = value[1];
        crate::dict::currency::CURRENCY_DICT
            .iter()
            .find_map(|&(c, s)| (c == code).then_some(s.to_string()))
            .ok_or(CodecError::UnknownExtension(code))
    } else if value[0] == 0x01 {
        let currency = String::from_utf8(value[1..].to_vec())
            .map_err(|_| CodecError::InvalidData("invalid UTF-8 in currency".to_string()))?;
        // T6: reject non-canonical encoding — if this currency is in the dict,
        // the encoder must have used dict form [0x00, code].
        let upper = currency.to_uppercase();
        if crate::dict::currency::CURRENCY_DICT
            .iter()
            .any(|&(_, sym)| sym == upper.as_str())
        {
            return Err(CodecError::InvalidData(format!(
                "non-canonical currency encoding: {currency} must use dict form"
            )));
        }
        Ok(currency)
    } else {
        Err(CodecError::UnknownExtension(value[0]))
    }
}

/// Decode token address from TLV value bytes:
///   [0x00, code] → dict reverse lookup
///   [0x01, 20 bytes] → raw hex address
pub(super) fn decode_token_address(value: &[u8]) -> Result<String, CodecError> {
    if value.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    if value[0] == 0x00 {
        if value.len() < 2 {
            return Err(CodecError::Truncated { needed: 2, had: 1 });
        }
        let code = value[1];
        crate::dict::token::TOKEN_DICT
            .iter()
            .find_map(|&(c, addr)| (c == code).then_some(addr.to_string()))
            .ok_or(CodecError::UnknownExtension(code))
    } else if value[0] == 0x01 {
        bytes_to_address(&value[1..])
        // NOTE: T6 canonical-aliasing check is NOT applied here.
        // Token addresses may legitimately appear raw even when the address is
        // "known" — e.g. WETH 0x4200…0006 on Base: dict code 24 is OP range,
        // outside Base range → encoder emits raw. Applying a raw→dict rejection
        // here would break valid cross-chain payloads. Chain ID and Currency
        // have clean bijective dict mappings; token addresses do not.
    } else {
        Err(CodecError::UnknownExtension(value[0]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIX #1: non-ASCII text must round-trip through dict layer.
    /// "Café 日本語 ñ" contains no `APP_DICT` pattern, so `apply_dict` would
    /// emit exactly its UTF-8 bytes — fed here directly to `reverse_dict`.
    /// The old `b as char` (Latin-1) path corrupted every multi-byte char.
    #[test]
    fn reverse_dict_roundtrips_non_ascii() {
        let original = "Café 日本語 ñ";
        let encoded = original.as_bytes(); // == apply_dict(original) — no dict match
        let decoded = reverse_dict(encoded).expect("valid UTF-8 must decode");
        assert_eq!(decoded, original, "non-ASCII text must round-trip intact");
    }

    /// FIX #1: invalid UTF-8 input must surface an error, not silent garbage.
    #[test]
    fn reverse_dict_invalid_utf8_errors() {
        // 0xFF is never a valid UTF-8 byte.
        let bad = [b'a', 0xFF, b'b'];
        let err = reverse_dict(&bad).unwrap_err();
        assert!(
            matches!(err, CodecError::InvalidData(_)),
            "expected InvalidData for invalid UTF-8, got {err:?}"
        );
    }

    /// Regression: dict-code expansion still works on a UTF-8-decoded string.
    #[test]
    fn reverse_dict_expands_dict_code() {
        // 0x06 = "Invoice" dict code.
        let decoded = reverse_dict(&[0x06, b' ', b'#', b'1']).unwrap();
        assert_eq!(decoded, "Invoice #1");
    }

    // --- T6: decoder rejects raw-form for dict-known values ---

    /// decode_chain_id must reject raw-varint form for a chain ID that exists in CHAIN_DICT.
    #[test]
    fn decode_chain_id_rejects_raw_for_dict_known() {
        use crate::varint::write_varint;
        // Ethereum (chain 1) is in CHAIN_DICT — must use dict form [0x00, 0x01], not raw.
        let mut value = vec![0x01u8]; // raw prefix
        write_varint(1u64, &mut value); // raw chain_id = 1
        let err = decode_chain_id(&value).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidData(_)),
            "expected InvalidData for raw-encoded dict-known chain, got {err:?}"
        );
    }

    /// decode_chain_id must accept raw-varint form for an unknown chain (not in CHAIN_DICT).
    #[test]
    fn decode_chain_id_accepts_raw_for_unknown_chain() {
        use crate::varint::write_varint;
        // Chain 5 (Goerli) is not in CHAIN_DICT — raw form is correct.
        let mut value = vec![0x01u8];
        write_varint(5u64, &mut value);
        let result = decode_chain_id(&value).unwrap();
        assert_eq!(result, 5);
    }

    /// decode_currency must reject raw UTF-8 form for a currency that exists in the dict.
    #[test]
    fn decode_currency_rejects_raw_for_dict_known() {
        // USDC is in CURRENCY_CODE_TO_SYMBOL — must use dict form [0x00, 0x01], not raw.
        let mut value = vec![0x01u8]; // raw prefix
        value.extend_from_slice(b"USDC");
        let err = decode_currency(&value).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidData(_)),
            "expected InvalidData for raw-encoded dict-known currency, got {err:?}"
        );
    }

    /// P1-F2: decode_currency must reject any prefix that is neither 0x00 nor 0x01.
    #[test]
    fn decode_currency_rejects_unknown_prefix() {
        let value = vec![0x02u8, b'X', b'Y', b'Z'];
        let err = decode_currency(&value).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::UnknownExtension(0x02)),
            "expected UnknownExtension(0x02) for unknown currency prefix, got {err:?}"
        );
    }

    /// P1-F3: decode_token_address must reject any prefix that is neither 0x00 nor 0x01.
    #[test]
    fn decode_token_address_rejects_unknown_prefix() {
        let mut value = vec![0x02u8];
        value.extend_from_slice(&[0u8; 20]); // 20 bytes of zeros (valid address body)
        let err = decode_token_address(&value).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::UnknownExtension(0x02)),
            "expected UnknownExtension(0x02) for unknown token-address prefix, got {err:?}"
        );
    }

    /// decode_token_address must accept raw 20-byte form even for a dict-known address
    /// because the canonical encoder legitimately emits raw when the dict code falls
    /// outside the invoice's chain range (e.g. WETH 0x4200…0006 on Base — code 24 is
    /// OP range, so encoder emits raw). T6 canonical-aliasing is scoped to chain_id
    /// and currency only; token addresses have cross-chain collisions that make a
    /// blanket raw→dict rejection unsound.
    #[test]
    fn decode_token_address_accepts_raw_for_dict_known_cross_chain() {
        // WETH 0x4200…0006 on Base is legitimately raw-encoded by the canonical encoder.
        let addr_bytes: [u8; 20] = [
            0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x06,
        ];
        let mut value = vec![0x01u8];
        value.extend_from_slice(&addr_bytes);
        let result = decode_token_address(&value).unwrap();
        assert_eq!(result, "0x4200000000000000000000000000000000000006");
    }
}
