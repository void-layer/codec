//! Structural codec limits — single source of truth.
//!
//! These caps are shared by the encode and decode paths. Keeping them in one
//! module prevents the encode/decode sides from silently drifting apart.

/// Maximum number of TLV records in a single canonical payload.
pub(crate) const MAX_TLV_COUNT: usize = 64;

/// Maximum byte length of a single TLV value.
pub(crate) const MAX_VALUE_SIZE: usize = 4096;

/// Maximum line items per invoice.
pub(crate) const MAX_ITEMS: usize = 50;

/// Maximum trailing-zero count for a mantissa-encoded amount.
/// A valid U256 has at most 77 decimal digits, so a base-10 value can carry
/// up to 77 trailing zeros (e.g. 10^77 < 2^256). Decode must accept any count
/// a valid U256 can produce — capping lower would reject valid encodings.
pub(crate) const MAX_TRAILING_ZEROS: u32 = 77;

/// Maximum `scale` byte in the packed-items quantity encoding.
/// scale is the number of decimal places: 0..=18. Values above 18 cannot be
/// represented in f64 without precision loss beyond the f64 mantissa domain.
pub(crate) const MAX_QUANTITY_SCALE: u32 = 18;

/// Maximum safe integer for f64 mantissa precision (2^53).
/// scaled_value above this cannot be represented exactly in f64.
pub(crate) const MAX_SAFE_F64_INT: u64 = 9_007_199_254_740_992; // 2^53
