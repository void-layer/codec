// Dead-code lint suppressed: pub(crate) dict statics consumed by encode/decode in Phase 2B;
// #[expect] incompatible with inline-test target (lint never fires on test binary → unfulfilled_lint_expectations).
#![allow(dead_code)]

pub(crate) mod app;
pub(crate) mod chain;

#[cfg(test)]
mod tests {
    use super::app::APP_DICT;
    use super::chain::CHAIN_DICT;
    use std::fmt::Write as _;
    use tiny_keccak::{Hasher, Keccak};

    // Two-commit pattern: run tests once with <TBD>, capture actual hashes from
    // failure output, then paste them here and commit again.
    const APP_DICT_HASH: &str = "8abb746c2f968c2bde2b450aee01ce88aabe9df4bb8938bd6d02b587b4954b2e";
    const CHAIN_DICT_HASH: &str =
        "6ddf0a04233a8b0b6dffe4658782eb5bd13391b37d202894e4da66efc5b388da";

    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    fn keccak256_hex(data: &[u8]) -> String {
        let mut k = Keccak::v256();
        let mut out = [0u8; 32];
        k.update(data);
        k.finalize(&mut out);
        to_hex(&out)
    }

    /// Hash all APP_DICT entries: for each entry iterate sorted keys,
    /// feed (key_bytes || value_byte) into keccak256.
    fn hash_app_dict() -> String {
        let mut keys: Vec<&'static str> = APP_DICT.keys().copied().collect();
        keys.sort_unstable();
        let mut k = Keccak::v256();
        for key in &keys {
            k.update(key.as_bytes());
            k.update(&[*APP_DICT.get(key).unwrap()]);
        }
        let mut out = [0u8; 32];
        k.finalize(&mut out);
        to_hex(&out)
    }

    /// Hash all CHAIN_DICT entries: iterate sorted keys,
    /// feed (key_be_bytes || value_byte) into keccak256.
    fn hash_chain_dict() -> String {
        let mut keys: Vec<u32> = CHAIN_DICT.keys().copied().collect();
        keys.sort_unstable();
        let mut k = Keccak::v256();
        for key in &keys {
            k.update(&key.to_be_bytes());
            k.update(&[*CHAIN_DICT.get(key).unwrap()]);
        }
        let mut out = [0u8; 32];
        k.finalize(&mut out);
        to_hex(&out)
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
        let hash = keccak256_hex(&[]);
        assert_eq!(
            hash,
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
    }
}
