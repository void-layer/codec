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

pub(super) const MAX_TLV_COUNT: usize = 64;
pub(super) const MAX_VALUE_SIZE: usize = 4096;
pub(super) const MAX_PAYLOAD_SIZE: usize = 1481; // (2000 - 25 prefix) / 1.333 Base64url ratio

/// Maximum line items per invoice — must match decode::MAX_ITEMS (50).
pub(super) const MAX_ITEMS: usize = 50;
