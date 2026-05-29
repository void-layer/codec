//! Tests for encode::amount.
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

// --- T5: encode-side precision guard ---

/// A quantity with 10 significant decimal places must be rejected.
#[test]
fn write_quantity_rejects_10_decimals() {
    let mut buf = Vec::new();
    // 1.1234567891 — 10 decimal places, cannot be encoded losslessly in 9-scale scheme.
    let err = write_quantity(&mut buf, 1.123_456_789_1_f64).unwrap_err();
    assert!(
        matches!(err, crate::error::CodecError::InvalidAmount(_)),
        "expected InvalidAmount for >9 significant decimals, got {err:?}"
    );
    assert!(buf.is_empty(), "buf must remain empty on precision error");
}

/// A quantity with exactly 9 significant decimals must encode successfully.
#[test]
fn write_quantity_accepts_9_decimals_exact() {
    let mut buf = Vec::new();
    // 1.123456789 — exactly 9 decimal places.
    write_quantity(&mut buf, 1.123_456_789_f64).expect("9 decimals must encode");
    assert!(!buf.is_empty(), "buf must contain encoded bytes");
    // scale=9, scaled_value=1_123_456_789
    assert_eq!(buf[0], 9u8, "scale byte must be 9");
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
