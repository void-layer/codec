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

/// Maximum safe integer for f64 mantissa precision (2^53).
/// scaled_value above this cannot be represented exactly in f64.
pub(crate) const MAX_SAFE_F64_INT: u64 = 9_007_199_254_740_992; // 2^53

/// Canonical quantity scale cap (wire: 1 byte). Encoder enforces; decoder rejects
/// any scale above this as non-canonical (per D-Bx canonical contract — T6 family).
pub(crate) const MAX_CANONICAL_QUANTITY_SCALE: u8 = 9;
