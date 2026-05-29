// Quantity / mantissa / U256 / numeric encoding.
// Mirrors writeMantissa / writeQuantity from varint.ts.

use crate::error::CodecError;
use crate::limits::{MAX_CANONICAL_QUANTITY_SCALE, MAX_TRAILING_ZEROS};
use crate::varint::{write_bigint_varint, write_varint};

/// Encode a u32 as 4-byte big-endian.
pub(super) fn uint32_be(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Encode a u64 as LEB128 varint bytes.
pub(super) fn varint_bytes(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    write_varint(value, &mut buf);
    buf
}

/// Encode a decimal integer string (BigInt) as mantissa + trailing-zeros.
/// Mirrors `writeMantissa` from varint.ts.
/// Amount domain is U256 — matches the on-chain uint256 domain and the TS BigInt reference.
pub(super) fn mantissa_bytes(value_str: &str) -> Result<Vec<u8>, CodecError> {
    use ruint::aliases::U256;

    let value: U256 = U256::from_str_radix(value_str, 10)
        .map_err(|_| CodecError::InvalidAmount(value_str.to_string()))?;

    let mut buf = Vec::new();
    if value == U256::ZERO {
        // mantissa = 0 (single 0x00 byte), zeros = 0
        write_bigint_varint(&[0], &mut buf);
        buf.push(0);
        return Ok(buf);
    }

    // A U256 value has at most 77 decimal digits, so at most 77 trailing zeros.
    // Decode accepts a zeros byte in 0..=MAX_TRAILING_ZEROS; encode must never emit more.
    let ten = U256::from(10u64);
    let mut mantissa = value;
    let mut zeros: u32 = 0;
    while mantissa % ten == U256::ZERO {
        mantissa /= ten;
        zeros += 1;
        if zeros > MAX_TRAILING_ZEROS {
            return Err(CodecError::InvalidAmount(format!(
                "trailing-zero count exceeds U256 domain max {MAX_TRAILING_ZEROS}"
            )));
        }
    }
    // Write mantissa as big-endian bytes via bigint_varint
    let mantissa_be: [u8; 32] = mantissa.to_be_bytes();
    write_bigint_varint(&mantissa_be, &mut buf);
    buf.push(zeros as u8);
    Ok(buf)
}

const QTY_EPS: f64 = 1e-9;
const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;

/// Encode a fractional quantity as [scale: u8][scaled_value: varint].
/// Mirrors writeQuantity from varint.ts.
pub(super) fn write_quantity(buf: &mut Vec<u8>, qty: f64) -> Result<(), CodecError> {
    if !qty.is_finite() {
        return Err(CodecError::InvalidAmount(format!(
            "quantity must be finite, got {qty}"
        )));
    }
    // A negative quantity has no representable encoding (`scaled_int` is u64).
    // Without this guard `-5.0 as u64` saturates to 0 — a silent data corruption.
    if qty < 0.0 {
        return Err(CodecError::InvalidAmount(format!(
            "quantity must be non-negative, got {qty}"
        )));
    }
    let mut scale = 0u8;
    let mut scaled = qty;
    while scale < MAX_CANONICAL_QUANTITY_SCALE && (scaled.round() - scaled).abs() > QTY_EPS {
        scale += 1;
        scaled = qty * 10f64.powi(scale as i32);
    }
    // If scale exhausted (==MAX_SCALE) and residual > tolerance, the value has more than
    // 9 significant decimals — reject instead of silently rounding.
    if scale == MAX_CANONICAL_QUANTITY_SCALE && (scaled.round() - scaled).abs() > QTY_EPS {
        return Err(CodecError::InvalidAmount(format!(
            "quantity {qty} has more than 9 significant decimals; encode would lose precision"
        )));
    }
    let rounded = scaled.round();
    // Explicit range check before the cast: `f64 as u64` saturates a value above
    // u64::MAX silently. u64::MAX is not exactly representable as f64, so guard
    // against `2^64` (the smallest f64 strictly above the u64 range).
    if !(0.0..TWO_POW_64).contains(&rounded) {
        return Err(CodecError::InvalidAmount(format!(
            "quantity {qty} scaled to {rounded} exceeds u64 range"
        )));
    }
    let scaled_int = rounded as u64;
    buf.push(scale);
    write_varint(scaled_int, buf);
    Ok(())
}

#[cfg(test)]
mod tests;
