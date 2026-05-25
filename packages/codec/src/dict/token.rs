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

use crate::dict::data::v1_tokens::V1_TOKEN_DICT_ENTRIES;

pub(crate) static TOKEN_DICT: &[(u8, &str)] = V1_TOKEN_DICT_ENTRIES;

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
