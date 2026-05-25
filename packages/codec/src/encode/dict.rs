// Dictionary substitution + chain/currency dict encoding.
// Mirrors applyDict from app-dict.ts and the chain-dict / CURRENCY_DICT schemes.

use crate::dict::chain::CHAIN_DICT;
use crate::error::CodecError;
use crate::varint::write_varint;

/// Compile-time-ordered `APP_DICT` entries, longest pattern first.
///
/// `APP_DICT` is a `phf_map!` whose iteration order is hash-order, not the
/// length-descending order `apply_dict` requires for correct longest-match.
/// This slice hardcodes that order so the hot path needs zero per-call sorting
/// or allocation. It is the single ordered source of truth — `decode::dict`
/// reuses it for `reverse_dict` so the two sides cannot diverge. The dict-lock
/// test in `dict::tests` asserts it matches `APP_DICT` (same set of pairs).
pub(crate) static APP_DICT_ENTRIES: &[(&str, u8)] = &[
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

/// Lookup table: `true` at index `b` iff byte `b` is a reserved `APP_DICT` code.
/// Built once at compile time — zero per-call allocation.
const fn build_dict_code_set() -> [bool; 256] {
    let mut set = [false; 256];
    let mut i = 0;
    while i < APP_DICT_ENTRIES.len() {
        set[APP_DICT_ENTRIES[i].1 as usize] = true;
        i += 1;
    }
    set
}
static DICT_CODE_SET: [bool; 256] = build_dict_code_set();

/// Apply app-level dictionary substitution (mirrors applyDict from app-dict.ts).
/// Replaces known string patterns with 1-byte control codes.
/// Longest match first — iterate entries in length-descending order.
///
/// Returns `Err(CodecError::InvalidData)` if the input contains any raw byte equal
/// to an actual dictionary code value. Such bytes would be misinterpreted by
/// `reverse_dict` as dictionary codes on decode, producing a different value.
/// Only the exact `APP_DICT` code values are reserved — non-code control
/// characters such as LF (0x0A) pass through unchanged so multi-line `notes`
/// encode correctly (matches the TS reference).
pub(super) fn apply_dict(input: &str) -> Result<Vec<u8>, CodecError> {
    // Reject only bytes equal to an actual dict code (derived from APP_DICT).
    if let Some(c) = input
        .chars()
        .find(|&c| (c as u32) < 0x100 && DICT_CODE_SET[c as usize])
    {
        return Err(CodecError::InvalidData(format!(
            "field value contains reserved dictionary code byte: 0x{:02x}",
            c as u8
        )));
    }

    let mut text = input.to_string();
    for (pattern, code) in APP_DICT_ENTRIES {
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
mod tests;
