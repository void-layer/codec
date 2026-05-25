// TLV type registry constants mirror tlv-map.ts TlvType enum.

// ---------------------------------------------------------------------------
// TLV type numbers (mirrors tlv-map.ts TlvType)
// ---------------------------------------------------------------------------

// Optional (odd) types
pub(crate) const TLV_TOKEN_ADDRESS: u8 = 1;
pub(crate) const TLV_CLIENT_WALLET: u8 = 3;
pub(crate) const TLV_NOTES: u8 = 5;
pub(crate) const TLV_FROM_EMAIL: u8 = 7;
pub(crate) const TLV_FROM_PHONE: u8 = 9;
pub(crate) const TLV_FROM_ADDRESS: u8 = 11;
pub(crate) const TLV_CLIENT_EMAIL: u8 = 13;
pub(crate) const TLV_CLIENT_PHONE: u8 = 15;
pub(crate) const TLV_CLIENT_ADDRESS: u8 = 17;
pub(crate) const TLV_TAX: u8 = 19;
pub(crate) const TLV_DISCOUNT: u8 = 21;
pub(crate) const TLV_DOMAIN_SEPARATOR: u8 = 31;
pub(crate) const TLV_FROM_TAX_ID: u8 = 35;
pub(crate) const TLV_CLIENT_TAX_ID: u8 = 37;

// Required (even) types
pub(crate) const TLV_CHAIN_ID: u8 = 2;
pub(crate) const TLV_ISSUED_AT: u8 = 4;
pub(crate) const TLV_DUE_AT: u8 = 6;
pub(crate) const TLV_DECIMALS: u8 = 8;
pub(crate) const TLV_FROM_WALLET: u8 = 10;
pub(crate) const TLV_CURRENCY: u8 = 12;
pub(crate) const TLV_ITEMS: u8 = 14;
pub(crate) const TLV_FROM_NAME: u8 = 16;
pub(crate) const TLV_CLIENT_NAME: u8 = 18;
pub(crate) const TLV_SALT: u8 = 20;
pub(crate) const TLV_INVOICE_ID: u8 = 22;
pub(crate) const TLV_TOTAL: u8 = 24;

// Wire format constants
pub(crate) const MAGIC: u8 = 0x56; // 'V'
pub(crate) const VERSION: u8 = 0x01;
/// High bit of VERSION byte signals whole-payload Brotli compression (set by JS shim).
pub(crate) const COMPRESSED_FLAG: u8 = 0x80;

/// Single source of truth for all v1 known TLV tags.
///
/// This list is the canonical registry: the decoder imports it directly so the
/// encode and decode sides cannot silently diverge when new tags are added.
///
/// Content tags (25) + TLV_DOMAIN_SEPARATOR (31) + TLV_FROM_TAX_ID (35) + TLV_CLIENT_TAX_ID (37) = 28 total.
pub(crate) const KNOWN_TAGS: &[u8] = &[
    TLV_TOKEN_ADDRESS,    // 1
    TLV_CHAIN_ID,         // 2
    TLV_CLIENT_WALLET,    // 3
    TLV_ISSUED_AT,        // 4
    TLV_NOTES,            // 5
    TLV_DUE_AT,           // 6
    TLV_FROM_EMAIL,       // 7
    TLV_DECIMALS,         // 8
    TLV_FROM_PHONE,       // 9
    TLV_FROM_WALLET,      // 10
    TLV_FROM_ADDRESS,     // 11
    TLV_CURRENCY,         // 12
    TLV_CLIENT_EMAIL,     // 13
    TLV_ITEMS,            // 14
    TLV_CLIENT_PHONE,     // 15
    TLV_FROM_NAME,        // 16
    TLV_CLIENT_ADDRESS,   // 17
    TLV_CLIENT_NAME,      // 18
    TLV_TAX,              // 19
    TLV_SALT,             // 20
    TLV_DISCOUNT,         // 21
    TLV_INVOICE_ID,       // 22
    TLV_TOTAL,            // 24
    TLV_DOMAIN_SEPARATOR, // 31
    TLV_FROM_TAX_ID,      // 35
    TLV_CLIENT_TAX_ID,    // 37
];

// ---------------------------------------------------------------------------
// T7 tag-contract tests — KNOWN_TAGS must cover all encoder-emitted tags
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// All TLV_* constants that encode/mod.rs may insert into the BTreeMap.
    const ALL_EMITTED_TAGS: &[u8] = &[
        TLV_TOKEN_ADDRESS,
        TLV_CHAIN_ID,
        TLV_CLIENT_WALLET,
        TLV_ISSUED_AT,
        TLV_NOTES,
        TLV_DUE_AT,
        TLV_FROM_EMAIL,
        TLV_DECIMALS,
        TLV_FROM_PHONE,
        TLV_FROM_WALLET,
        TLV_FROM_ADDRESS,
        TLV_CURRENCY,
        TLV_CLIENT_EMAIL,
        TLV_ITEMS,
        TLV_CLIENT_PHONE,
        TLV_FROM_NAME,
        TLV_CLIENT_ADDRESS,
        TLV_CLIENT_NAME,
        TLV_TAX,
        TLV_SALT,
        TLV_DISCOUNT,
        TLV_INVOICE_ID,
        TLV_TOTAL,
        TLV_DOMAIN_SEPARATOR,
        TLV_FROM_TAX_ID,
        TLV_CLIENT_TAX_ID,
    ];

    /// Every tag the encoder can emit must appear in KNOWN_TAGS.
    /// Prevents adding a TLV_* constant without updating the decoder's accept-set.
    #[test]
    fn all_emitted_tags_are_in_known_tags() {
        for &tag in ALL_EMITTED_TAGS {
            assert!(
                KNOWN_TAGS.contains(&tag),
                "TLV tag {tag} is emitted by the encoder but missing from KNOWN_TAGS — \
                 the decoder would reject all payloads using this tag"
            );
        }
    }

    /// KNOWN_TAGS must not contain duplicates.
    #[test]
    fn known_tags_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for &tag in KNOWN_TAGS {
            assert!(seen.insert(tag), "KNOWN_TAGS contains duplicate tag {tag}");
        }
    }

    /// KNOWN_TAGS and ALL_EMITTED_TAGS must have the same cardinality.
    #[test]
    fn known_tags_cardinality_matches_emitted() {
        assert_eq!(
            KNOWN_TAGS.len(),
            ALL_EMITTED_TAGS.len(),
            "KNOWN_TAGS has {} entries but ALL_EMITTED_TAGS has {}",
            KNOWN_TAGS.len(),
            ALL_EMITTED_TAGS.len()
        );
    }
}
