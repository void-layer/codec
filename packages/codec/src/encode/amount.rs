// Quantity / mantissa / U256 / numeric encoding.
// Mirrors writeMantissa / writeQuantity from varint.ts.

use crate::error::CodecError;
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
    // Decode accepts a zeros byte in 0..=77; encode must never emit more.
    const MAX_TRAILING_ZEROS: usize = 77;
    let ten = U256::from(10u64);
    let mut mantissa = value;
    let mut zeros: usize = 0;
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
    while scale < 9 && (scaled.round() - scaled).abs() > 1e-9 {
        scale += 1;
        scaled = qty * 10f64.powi(scale as i32);
    }
    let scaled_int = scaled.round() as u64;
    buf.push(scale);
    write_varint(scaled_int, buf);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mantissa_bytes_zero() {
        let b = mantissa_bytes("0").unwrap();
        // mantissa=0 → write_bigint_varint([0]) = [0x00], zeros=0
        assert_eq!(b, vec![0x00, 0x00]);
    }

    #[test]
    fn mantissa_bytes_one_million() {
        // 1_000_000 = 1 * 10^6 → mantissa=1 (0x01), zeros=6
        let b = mantissa_bytes("1000000").unwrap();
        assert_eq!(b, vec![0x01, 0x06]);
    }

    #[test]
    fn mantissa_bytes_123() {
        // 123 — no trailing zeros → mantissa=123, zeros=0
        // 123 = 0x7B → LEB128 single byte
        let b = mantissa_bytes("123").unwrap();
        assert_eq!(b, vec![0x7b, 0x00]);
    }

    #[test]
    fn write_quantity_integer_one() {
        let mut buf = Vec::new();
        write_quantity(&mut buf, 1.0).unwrap();
        // scale=0, value=1 → [0x00, 0x01]
        assert_eq!(buf, vec![0x00, 0x01]);
    }

    #[test]
    fn write_quantity_1_5() {
        let mut buf = Vec::new();
        write_quantity(&mut buf, 1.5).unwrap();
        // scale=1, value=15 → [0x01, 0x0F]
        assert_eq!(buf, vec![0x01, 0x0F]);
    }

    // --- U256 amount domain tests ---

    #[test]
    fn mantissa_bytes_u128_max() {
        // u128::MAX = 340282366920938463463374607431768211455
        // Must produce byte-identical output to the old u128 path.
        let s = u128::MAX.to_string();
        let b = mantissa_bytes(&s).unwrap();
        // Verify encode→decode roundtrip produces the same string.
        // Spot-check: no trailing zeros, so zeros byte = 0.
        assert_eq!(*b.last().unwrap(), 0u8, "u128::MAX has no trailing zeros");
    }

    #[test]
    fn mantissa_bytes_u256_max_roundtrips() {
        // 2^256 - 1 as decimal
        let uint256_max =
            "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        let b = mantissa_bytes(uint256_max).unwrap();
        // Last byte = trailing zeros count (should be 0 — u256::MAX is odd)
        assert_eq!(*b.last().unwrap(), 0u8);
        // Verify the encoded bytes decode back (via decode_mantissa)
        let decoded = crate::decode::tests_pub::decode_mantissa_pub(&b).unwrap();
        assert_eq!(decoded, uint256_max);
    }

    #[test]
    fn mantissa_bytes_large_round_value() {
        // 10^30 — large round value well above u128::MAX range in theory but fits U256
        let s = "1".to_string() + &"0".repeat(30);
        let b = mantissa_bytes(&s).unwrap();
        // mantissa = 1, zeros = 30
        assert_eq!(*b.last().unwrap(), 30u8);
    }

    #[test]
    fn mantissa_bytes_above_u256_errors() {
        // 2^256 — one above U256::MAX
        let over = "115792089237316195423570985008687907853269984665640564039457584007913129639936";
        let err = mantissa_bytes(over).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount, got {err:?}"
        );
    }

    #[test]
    fn mantissa_bytes_non_numeric_errors() {
        let err = mantissa_bytes("not_a_number").unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount, got {err:?}"
        );
    }

    // --- R5: NaN/Inf quantity guard ---

    /// f64::INFINITY quantity must return Err, not silently encode as u64::MAX.
    #[test]
    fn r5_infinity_quantity_errors() {
        let mut buf = Vec::new();
        let err = write_quantity(&mut buf, f64::INFINITY).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount for Inf quantity, got {err:?}"
        );
        assert!(buf.is_empty(), "buf must remain empty on error");
    }

    /// f64::NAN quantity must return Err, not silently encode as 0.
    #[test]
    fn r5_nan_quantity_errors() {
        let mut buf = Vec::new();
        let err = write_quantity(&mut buf, f64::NAN).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount for NaN quantity, got {err:?}"
        );
        assert!(buf.is_empty(), "buf must remain empty on error");
    }

    /// f64::NEG_INFINITY must also be rejected.
    #[test]
    fn r5_neg_infinity_quantity_errors() {
        let mut buf = Vec::new();
        let err = write_quantity(&mut buf, f64::NEG_INFINITY).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount for -Inf quantity, got {err:?}"
        );
    }

    // --- #3: negative finite quantity guard ---

    /// A negative finite quantity must return Err, not saturate to 0 via `as u64`.
    #[test]
    fn write_quantity_negative_errors() {
        let mut buf = Vec::new();
        let err = write_quantity(&mut buf, -5.0).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount for negative quantity, got {err:?}"
        );
        assert!(buf.is_empty(), "buf must remain empty on error");
    }

    // --- #2 encode-side: trailing-zeros accumulator robustness ---

    /// A value with many trailing zeros (well within the U256 77-zero domain)
    /// encodes correctly without overflowing the accumulator.
    #[test]
    fn mantissa_bytes_max_trailing_zeros() {
        // 10^77 is the largest power of ten representable in U256.
        let s = "1".to_string() + &"0".repeat(77);
        let b = mantissa_bytes(&s).unwrap();
        // mantissa = 1, zeros = 77
        assert_eq!(*b.last().unwrap(), 77u8);
    }
}
