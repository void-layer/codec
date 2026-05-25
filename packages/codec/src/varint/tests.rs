//! Tests for varint.
use super::*;

#[test]
fn writes_zero_as_single_byte_zero() {
    let mut buf = Vec::new();
    write_varint(0, &mut buf);
    assert_eq!(buf, &[0x00]);
}

#[test]
fn writes_127_as_single_byte() {
    let mut buf = Vec::new();
    write_varint(127, &mut buf);
    assert_eq!(buf, &[0x7F]);
}

#[test]
fn writes_128_with_continuation_bit() {
    let mut buf = Vec::new();
    write_varint(128, &mut buf);
    // 128 = 0b10000000 → LEB128: [0x80, 0x01]
    assert_eq!(buf, &[0x80, 0x01]);
}

#[test]
fn returns_truncated_error_on_short_buffer() {
    // A byte with continuation bit set but no following byte.
    let buf = &[0x80u8];
    let err = read_varint(buf, 0).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

#[test]
fn returns_overflow_error_past_max_bytes() {
    // Craft MAX_BYTES+1 bytes each with continuation bit set.
    let buf: Vec<u8> = (0..=MAX_BYTES).map(|_| 0x80u8).collect();
    let err = read_varint(&buf, 0).unwrap_err();
    assert!(
        matches!(err, CodecError::VarintOverflow(_)),
        "expected VarintOverflow, got {err:?}"
    );
}

#[test]
fn max_bytes_constant_equals_37() {
    assert_eq!(MAX_BYTES, 37);
}

#[test]
fn bigint_uint256_max_roundtrips() {
    // 32 bytes of 0xFF — the maximum uint256 value.
    let uint256_max = vec![0xFFu8; 32];
    let mut buf = Vec::new();
    write_bigint_varint(&uint256_max, &mut buf);
    let (decoded, bytes_consumed) = read_bigint_varint(&buf, 0).unwrap();
    assert_eq!(decoded, uint256_max, "roundtrip value mismatch");
    assert_eq!(
        bytes_consumed,
        buf.len(),
        "bytes_consumed must equal full buffer"
    );
}

#[test]
fn known_u64_wire_bytes() {
    // Verify against TS reference values.
    let cases: &[(u64, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (127, &[0x7F]),
        (128, &[0x80, 0x01]),
        (16384, &[0x80, 0x80, 0x01]),
        (4_294_967_295, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]), // max uint32
    ];
    for (value, expected) in cases {
        let mut buf = Vec::new();
        write_varint(*value, &mut buf);
        assert_eq!(&buf[..], *expected, "write_varint({value}) wire mismatch");
        let (decoded, n) = read_varint(&buf, 0).unwrap();
        assert_eq!(decoded, *value, "read_varint roundtrip failed for {value}");
        assert_eq!(n, expected.len());
    }
}

#[test]
fn read_bounded_len_accepts_value_within_max() {
    // varint(100), max = 200 → Ok((100, 1))
    let mut buf = Vec::new();
    write_varint(100, &mut buf);
    let (len, consumed) = read_bounded_len(&buf, 0, 200).unwrap();
    assert_eq!(len, 100);
    assert_eq!(consumed, buf.len());
}

#[test]
fn read_bounded_len_accepts_value_equal_to_max() {
    let mut buf = Vec::new();
    write_varint(200, &mut buf);
    let (len, _) = read_bounded_len(&buf, 0, 200).unwrap();
    assert_eq!(len, 200);
}

#[test]
fn read_bounded_len_rejects_value_exceeding_max() {
    // varint(201), max = 200 → Err(Truncated)
    let mut buf = Vec::new();
    write_varint(201, &mut buf);
    let err = read_bounded_len(&buf, 0, 200).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

#[test]
fn read_bounded_len_rejects_huge_varint_before_cast() {
    // A varint encoding a value far above any plausible usize on wasm32
    // (2^40) must be rejected, not truncated.
    let mut buf = Vec::new();
    write_varint(1u64 << 40, &mut buf);
    let err = read_bounded_len(&buf, 0, 4096).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated for oversized length, got {err:?}"
    );
}

#[test]
fn read_bounded_len_propagates_truncated_buffer() {
    // Continuation bit set, no following byte.
    let buf = &[0x80u8];
    let err = read_bounded_len(buf, 0, 4096).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

#[cfg(not(target_arch = "wasm32"))]
proptest::proptest! {
    #[test]
    fn varint_roundtrips_for_any_u64(value in proptest::prelude::any::<u64>()) {
        let mut buf = Vec::new();
        write_varint(value, &mut buf);
        let (decoded, _) = read_varint(&buf, 0).unwrap();
        proptest::prelude::prop_assert_eq!(value, decoded);
    }
}
