//! Currency symbol ↔ TLV dict code mapping. Single source of truth for encode + decode.
//! Locked at codec v1.0 (per Constitution IV — append-only forever).
//! Layout: (code, symbol) — iterate for either direction.

pub(crate) static CURRENCY_DICT: &[(u8, &str)] = &[
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    /// v1 ordered entry list — order-sensitive for the lock hash.
    const V1_CURRENCY_DICT_ENTRIES: &[(u8, &str)] = &[
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

    const CURRENCY_DICT_HASH: &str =
        "e86c58a5c44f34c7a48ea79f7417d11b31867781952c9366939fc6956be2ba80";

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    fn hash_currency_dict() -> String {
        let mut buf = Vec::new();
        for (code, sym) in V1_CURRENCY_DICT_ENTRIES {
            buf.push(*code);
            buf.extend_from_slice(sym.as_bytes());
        }
        to_hex(&crate::hash::keccak256(&buf))
    }

    #[test]
    fn currency_dict_locked() {
        if std::env::var("VOID_DICT_OVERRIDE").as_deref() == Ok("1") {
            return;
        }
        let actual = hash_currency_dict();
        assert_eq!(
            actual, CURRENCY_DICT_HASH,
            "CURRENCY_DICT changed! Refusing unless VOID_DICT_OVERRIDE=1.\nActual hash: {actual}"
        );
    }

    #[test]
    fn currency_dict_matches_v1_entries() {
        assert_eq!(
            CURRENCY_DICT.len(),
            V1_CURRENCY_DICT_ENTRIES.len(),
            "CURRENCY_DICT count must match V1 list"
        );
        for (code, sym) in V1_CURRENCY_DICT_ENTRIES {
            assert!(
                CURRENCY_DICT.iter().any(|&(c, s)| c == *code && s == *sym),
                "CURRENCY_DICT missing entry ({code}, {sym:?})"
            );
        }
    }

    #[test]
    fn currency_dict_entry_count() {
        assert_eq!(
            CURRENCY_DICT.len(),
            11,
            "CURRENCY_DICT must have exactly 11 entries"
        );
    }
}
