/// V1 currency dictionary — wire-format data, **APPEND-ONLY**.
///
/// Per Constitution IV (Perpetual + Schema versioning), every entry here is part
/// of the wire format `void-layer/codec` v1. Existing `(code, symbol)` pairs are
/// LOCKED — modifying any entry breaks decoders in the wild.
///
/// Adding a new currency: append to the slice (do not reorder, do not modify).
/// The append must include a bump-and-add commit to the lock-hash test fixture.
///
/// Enforcement: `packages/codec/src/dict/currency.rs::currency_dict_locked`
/// hashes this slice and compares to a snapshot.
pub(crate) const V1_CURRENCY_DICT_ENTRIES: &[(u8, &str)] = &[
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
