// Reverse dictionary expansion: chain ID, currency, token address, and
// app-level text substitution.

use crate::dict::chain::CHAIN_DICT;
use crate::error::CodecError;
use crate::varint::read_varint;

use super::hex::bytes_to_address;

/// Reverse app-level dictionary substitution (mirrors reverseDict from app-dict.ts).
pub(super) fn reverse_dict(bytes: &[u8]) -> Result<String, CodecError> {
    // Decode raw bytes as UTF-8 (matches the TS reference's TextDecoder).
    // Dict-code bytes (0x02–0x0F) are valid single-byte UTF-8 and survive as
    // single chars, so the expansion loop below works unchanged.
    let mut text = String::from_utf8(bytes.to_vec())
        .map_err(|_| CodecError::CompressionFailed("invalid UTF-8 in dict text".to_string()))?;

    // Reverse entries longest-pattern-first (same order as apply_dict)
    let entries: &[(&str, u8)] = &[
        ("@outlook.com", 0x02),
        ("@hotmail.com", 0x0c),
        ("development", 0x0d),
        ("consulting", 0x0e),
        ("@gmail.com", 0x03),
        ("@yahoo.com", 0x04),
        ("https://", 0x05),
        ("Invoice", 0x06),
        ("Payment", 0x07),
        (".com", 0x09),
        ("INV-", 0x0f),
    ];

    // Apply in reverse order (shortest first for reverse) — mirrors TS [...DICT_ENTRIES].reverse()
    for &(pattern, code) in entries.iter().rev() {
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
            .find(|&(&_k, &v)| v == code)
            .map(|(&k, _)| k)
            .ok_or(CodecError::UnknownExtension(code))?;
        Ok(chain_id)
    } else if prefix == 0x01 {
        let (chain_id, _) = read_varint(value, 1)?;
        // Reject chain IDs > u32::MAX instead of silently truncating.
        u32::try_from(chain_id)
            .map_err(|_| CodecError::InvalidAmount(format!("chain ID {chain_id} overflows u32")))
    } else {
        Err(CodecError::UnknownExtension(prefix))
    }
}

/// Currency code → symbol (mirrors CURRENCY_DICT_REVERSE in tlv-map.ts). Static: zero per-call alloc.
static CURRENCY_CODE_TO_SYMBOL: &[(u8, &str)] = &[
    (1, "USDC"),
    (2, "USDT"),
    (3, "DAI"),
    (4, "ETH"),
    (5, "WETH"),
    (6, "MATIC"),
    (7, "POL"),
    (8, "WBTC"),
    (9, "USDC.E"),
    (10, "EURC"),
    (11, "USDT0"),
];

/// Token dict code → lowercase address (mirrors TOKEN_DICT_REVERSE in tlv-map.ts). Static: zero per-call alloc.
/// Code 43 = Base WETH (same address as Optimism code 24, different chain context).
static TOKEN_CODE_TO_ADDRESS: &[(u8, &str)] = &[
    (1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    (2, "0xdac17f958d2ee523a2206206994597c13d831ec7"),
    (3, "0x6b175474e89094c44da98b954eedeac495271d0f"),
    (4, "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
    (5, "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599"),
    (6, "0x1abaea1f7c830bd89acc67ec4af516284b1bc33c"),
    (7, "0x6c96de32cea08842dcc4058c14d3aaad7fa41dee"),
    (10, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"),
    (11, "0xff970a61a04b1ca14834a43f5de4533ebddb5cc8"),
    (12, "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9"),
    (13, "0xda10009cbd5d07dd0cecc66161fc93d7c9000da1"),
    (14, "0x82af49447d8a07e3bd95bd0d56f35241523fbab1"),
    (15, "0x2f2a2543b76a4166549f7aab2e75bef0aefc5b0f"),
    (20, "0x0b2c639c533813f4aa9d7837caf62653d097ff85"),
    (21, "0x7f5c764cbc14f9669b88837ca1490cca17c31607"),
    (22, "0x94b008aa00579c1307b0ef2c499ad98a8ce58e58"),
    (24, "0x4200000000000000000000000000000000000006"),
    (25, "0x68f180fcce6836688e9084f035309e29bf0a2095"),
    (30, "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359"),
    (31, "0x2791bca1f2de4661ed88a30c99a7a9449aa84174"),
    (32, "0xc2132d05d31c914a87c6611c10748aeb04b58e8f"),
    (33, "0x8f3cf7ad23cd3cadbd9735aff958023239c6a063"),
    (34, "0x7ceb23fd6bc0add59e62ac25578270cff1b9f619"),
    (35, "0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6"),
    (40, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
    (41, "0xd9aaec86b65d86f6a7b5b1b0c42ffa531710b6ca"),
    (42, "0x50c5725949a6f0c72e6c4a641f24049a917db0cb"),
    (43, "0x4200000000000000000000000000000000000006"),
    (44, "0x0555e30da8f98308edb960aa94c0ed47230d2b9c"),
    (45, "0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42"),
];

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
        CURRENCY_CODE_TO_SYMBOL
            .iter()
            .find(|&&(c, _)| c == code)
            .map(|&(_, s)| s.to_string())
            .ok_or(CodecError::UnknownExtension(code))
    } else {
        String::from_utf8(value[1..].to_vec())
            .map_err(|_| CodecError::CompressionFailed("invalid UTF-8 in currency".to_string()))
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
        TOKEN_CODE_TO_ADDRESS
            .iter()
            .find(|&&(c, _)| c == code)
            .map(|&(_, addr)| addr.to_string())
            .ok_or(CodecError::UnknownExtension(code))
    } else {
        bytes_to_address(&value[1..])
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
            matches!(err, CodecError::CompressionFailed(_)),
            "expected CompressionFailed for invalid UTF-8, got {err:?}"
        );
    }

    /// Regression: dict-code expansion still works on a UTF-8-decoded string.
    #[test]
    fn reverse_dict_expands_dict_code() {
        // 0x06 = "Invoice" dict code.
        let decoded = reverse_dict(&[0x06, b' ', b'#', b'1']).unwrap();
        assert_eq!(decoded, "Invoice #1");
    }
}
