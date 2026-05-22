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
