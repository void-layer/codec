//! Token address ↔ TLV dict code mapping. Single source of truth for encode + decode.
//! Locked at codec v1.0 (per Constitution IV — append-only forever).
//! Layout: (code, lowercase_address) — iterate for either direction.
//!
//! # WETH cross-chain asymmetry
//! Address 0x4200…0006 appears twice: code 24 (Optimism) and code 43 (Base).
//! Encode iterates by address → finds code 24 first, then CHAIN_CODE_RANGES
//! upgrades to code 43 when `network_id == 8453` (Base). Decode iterates by
//! code → returns the address directly (both entries map to the same bytes).
//! This asymmetry is intentional and must not be collapsed.

pub(crate) static TOKEN_DICT: &[(u8, &str)] = &[
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
    (24, "0x4200000000000000000000000000000000000006"), // Optimism WETH; Base WETH = code 43
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
    (43, "0x4200000000000000000000000000000000000006"), // Base WETH alias (same addr, different chain range)
    (44, "0x0555e30da8f98308edb960aa94c0ed47230d2b9c"),
    (45, "0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42"),
];

/// Chain ID → (code_min, code_max) range for token dict chain-range validation.
/// Co-located here because it is a codec-internal disambiguation rule that
/// determines which dict code to use for cross-chain tokens (e.g. Base WETH code 43
/// vs Optimism WETH code 24). Per Audit B: stays in codec, not networks.
pub(crate) static CHAIN_CODE_RANGES: &[(u32, u8, u8)] = &[
    (1, 1, 9),
    (42161, 10, 19),
    (10, 20, 29),
    (137, 30, 39),
    (8453, 40, 49),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    /// v1 ordered entry list for lock hash (code-ascending order).
    const V1_TOKEN_DICT_ENTRIES: &[(u8, &str)] = &[
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

    const TOKEN_DICT_HASH: &str =
        "342309ddb694efe0f56396f316c0f462327f706c0104344d7662e236a70a2c31";

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    fn hash_token_dict() -> String {
        let mut buf = Vec::new();
        for (code, addr) in V1_TOKEN_DICT_ENTRIES {
            buf.push(*code);
            buf.extend_from_slice(addr.as_bytes());
        }
        to_hex(&crate::hash::keccak256(&buf))
    }

    #[test]
    fn token_dict_locked() {
        if std::env::var("VOID_DICT_OVERRIDE").as_deref() == Ok("1") {
            return;
        }
        let actual = hash_token_dict();
        assert_eq!(
            actual, TOKEN_DICT_HASH,
            "TOKEN_DICT changed! Refusing unless VOID_DICT_OVERRIDE=1.\nActual hash: {actual}"
        );
    }

    #[test]
    fn token_dict_matches_v1_entries() {
        assert_eq!(
            TOKEN_DICT.len(),
            V1_TOKEN_DICT_ENTRIES.len(),
            "TOKEN_DICT count must match V1 list"
        );
        for (code, addr) in V1_TOKEN_DICT_ENTRIES {
            assert!(
                TOKEN_DICT.iter().any(|&(c, a)| c == *code && a == *addr),
                "TOKEN_DICT missing entry ({code}, {addr:?})"
            );
        }
    }

    #[test]
    fn token_dict_entry_count() {
        assert_eq!(
            TOKEN_DICT.len(),
            30,
            "TOKEN_DICT must have exactly 30 entries"
        );
    }
}
