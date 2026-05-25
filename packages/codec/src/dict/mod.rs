pub(crate) mod app;
pub(crate) mod chain;
pub(crate) mod currency;
pub(crate) mod token;

#[cfg(test)]
mod tests {
    use super::app::APP_DICT;
    use super::chain::CHAIN_DICT;
    use std::fmt::Write as _;

    // ---------------------------------------------------------------------
    // Dict-lock approach (fix #9): `APP_DICT` is a `phf_map!` whose iteration
    // order is hash-order, not insertion-order — a pure hash over the map
    // cannot detect a reordering of the v1 entries. Instead the lock hashes
    // an EXPLICIT hardcoded ordered entry list (`V1_APP_DICT_ENTRIES`) and
    // separately asserts that list matches `APP_DICT` as a set. Any
    // add / remove / reorder / value-change of a v1 entry changes either the
    // ordered hash or the set-equality assertion, so the lock fails loudly.
    // `CHAIN_DICT` gets the same treatment via `V1_CHAIN_DICT_ENTRIES`.
    // ---------------------------------------------------------------------

    /// v1 `APP_DICT` entries in their canonical (length-descending) order.
    /// This is the order-sensitive source of truth the lock hash is taken over.
    const V1_APP_DICT_ENTRIES: &[(&str, u8)] = &[
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

    /// v1 `CHAIN_DICT` entries in canonical order (ascending chain ID).
    const V1_CHAIN_DICT_ENTRIES: &[(u32, u8)] = &[
        (1, 0x01),
        (10, 0x03),
        (137, 0x04),
        (8453, 0x05),
        (42161, 0x02),
    ];

    // Locked hashes over the explicit ordered entry lists above.
    // Two-commit pattern: run tests once with <TBD>, capture actual hashes
    // from failure output, then paste them here and commit again.
    const APP_DICT_HASH: &str = "7e9fe8e27754369ef22a66cd8cb276f1bc938bb4096935ec00483b81cd9ec565";
    const CHAIN_DICT_HASH: &str =
        "6ddf0a04233a8b0b6dffe4658782eb5bd13391b37d202894e4da66efc5b388da";

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    /// Order-sensitive hash over the explicit v1 `APP_DICT` entry list.
    fn hash_app_dict() -> String {
        let mut buf = Vec::new();
        for (key, code) in V1_APP_DICT_ENTRIES {
            buf.extend_from_slice(key.as_bytes());
            buf.push(*code);
        }
        to_hex(&crate::hash::keccak256(&buf))
    }

    /// Order-sensitive hash over the explicit v1 `CHAIN_DICT` entry list.
    fn hash_chain_dict() -> String {
        let mut buf = Vec::new();
        for (chain_id, code) in V1_CHAIN_DICT_ENTRIES {
            buf.extend_from_slice(&chain_id.to_be_bytes());
            buf.push(*code);
        }
        to_hex(&crate::hash::keccak256(&buf))
    }

    #[test]
    fn app_dict_locked() {
        // Honor VOID_DICT_OVERRIDE=1 env var to skip the assert (D-B6).
        if std::env::var("VOID_DICT_OVERRIDE").as_deref() == Ok("1") {
            return;
        }
        let actual = hash_app_dict();
        assert_eq!(
            actual, APP_DICT_HASH,
            "Dictionary changed! Refusing unless VOID_DICT_OVERRIDE=1.\nActual hash: {actual}"
        );
    }

    #[test]
    fn chain_dict_locked() {
        // Honor VOID_DICT_OVERRIDE=1 env var to skip the assert (D-B6).
        if std::env::var("VOID_DICT_OVERRIDE").as_deref() == Ok("1") {
            return;
        }
        let actual = hash_chain_dict();
        assert_eq!(
            actual, CHAIN_DICT_HASH,
            "Dictionary changed! Refusing unless VOID_DICT_OVERRIDE=1.\nActual hash: {actual}"
        );
    }

    /// The explicit v1 entry list must match `APP_DICT` exactly as a set —
    /// guards against the phf map and the lock list silently diverging.
    #[test]
    fn v1_app_dict_entries_match_phf_map() {
        assert_eq!(
            V1_APP_DICT_ENTRIES.len(),
            APP_DICT.len(),
            "V1_APP_DICT_ENTRIES count must match APP_DICT"
        );
        for (key, code) in V1_APP_DICT_ENTRIES {
            assert_eq!(
                APP_DICT.get(key),
                Some(code),
                "APP_DICT entry for {key:?} diverged from V1_APP_DICT_ENTRIES"
            );
        }
    }

    /// The codec's runtime ordered dict slice (`encode::APP_DICT_ENTRIES`,
    /// reused by `decode::dict::reverse_dict`) must be byte-and-order-exact
    /// with the v1 lock list — closes the loop phf map ↔ lock list ↔ codec.
    #[test]
    fn encode_dict_entries_match_v1_lock_list() {
        assert_eq!(
            crate::encode::APP_DICT_ENTRIES,
            V1_APP_DICT_ENTRIES,
            "encode::APP_DICT_ENTRIES diverged from the v1 dict-lock list"
        );
    }

    /// The explicit v1 chain list must match `CHAIN_DICT` exactly as a set.
    #[test]
    fn v1_chain_dict_entries_match_phf_map() {
        assert_eq!(
            V1_CHAIN_DICT_ENTRIES.len(),
            CHAIN_DICT.len(),
            "V1_CHAIN_DICT_ENTRIES count must match CHAIN_DICT"
        );
        for (chain_id, code) in V1_CHAIN_DICT_ENTRIES {
            assert_eq!(
                CHAIN_DICT.get(chain_id),
                Some(code),
                "CHAIN_DICT entry for {chain_id} diverged from V1_CHAIN_DICT_ENTRIES"
            );
        }
    }

    #[test]
    fn app_dict_entry_count() {
        assert_eq!(APP_DICT.len(), 11, "APP_DICT must have exactly 11 entries");
    }

    #[test]
    fn chain_dict_entry_count() {
        assert_eq!(
            CHAIN_DICT.len(),
            5,
            "CHAIN_DICT must have exactly 5 entries"
        );
    }

    #[test]
    fn app_dict_spot_check() {
        assert_eq!(APP_DICT.get("@outlook.com"), Some(&0x02u8));
        assert_eq!(APP_DICT.get("@hotmail.com"), Some(&0x0cu8));
        assert_eq!(APP_DICT.get("INV-"), Some(&0x0fu8));
        assert_eq!(APP_DICT.get(".com"), Some(&0x09u8));
    }

    #[test]
    fn chain_dict_spot_check() {
        assert_eq!(CHAIN_DICT.get(&1u32), Some(&0x01u8)); // Ethereum
        assert_eq!(CHAIN_DICT.get(&42161u32), Some(&0x02u8)); // Arbitrum
        assert_eq!(CHAIN_DICT.get(&8453u32), Some(&0x05u8)); // Base
    }

    #[test]
    fn keccak256_smoke() {
        // Sanity: empty input keccak256 is the well-known value.
        let hash = to_hex(&crate::hash::keccak256(&[]));
        assert_eq!(
            hash,
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }
}
