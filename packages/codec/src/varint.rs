use crate::error::CodecError;

/// Maximum LEB128 bytes allowed per value.
/// ceil(256 / 7) = 37 — covers uint256 with margin (spec §3.15).
pub(crate) const MAX_BYTES: usize = 37;

/// Encodes a `u64` as LEB128 into `out`.
pub(crate) fn write_varint(value: u64, out: &mut Vec<u8>) {
    let mut v = value;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(byte);
            break;
        } else {
            out.push(byte | 0x80);
        }
    }
}

/// Decodes a LEB128-encoded `u64` from `buf` starting at `offset`.
///
/// Returns `(value, bytes_consumed)`.
///
/// Errors:
/// - `CodecError::Truncated` if the buffer ends mid-varint.
/// - `CodecError::VarintOverflow` if continuation bytes exceed `MAX_BYTES`.
pub(crate) fn read_varint(buf: &[u8], offset: usize) -> Result<(u64, usize), CodecError> {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    let mut bytes_read: usize = 0;

    loop {
        if bytes_read >= MAX_BYTES {
            return Err(CodecError::VarintOverflow(offset));
        }
        let pos = offset + bytes_read;
        if pos >= buf.len() {
            return Err(CodecError::Truncated {
                needed: pos + 1,
                had: buf.len(),
            });
        }
        let byte = buf[pos];
        bytes_read += 1;

        // Guard: shift >= 64 means this value cannot fit in a u64.
        // Must precede the left-shift to prevent overflow.
        if shift >= 64 {
            return Err(CodecError::VarintOverflow(offset));
        }
        let data = (byte & 0x7F) as u64;
        value |= data << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    Ok((value, bytes_read))
}

/// Encodes an arbitrary-precision unsigned integer (big-endian byte slice) as LEB128 into `out`.
///
/// `value` is interpreted as a big-endian unsigned integer.
/// An empty slice or all-zero slice encodes as a single `0x00` byte.
pub(crate) fn write_bigint_varint(value: &[u8], out: &mut Vec<u8>) {
    // Strip leading zero bytes to find the canonical representation.
    let value = strip_leading_zeros(value);

    if value.is_empty() {
        out.push(0);
        return;
    }

    // Work on a mutable little-endian byte copy for bit-shifting.
    let mut le = to_le_bytes(value);

    loop {
        let low7 = le[0] & 0x7F;
        shr7_le(&mut le);
        if is_zero_le(&le) {
            out.push(low7);
            break;
        } else {
            out.push(low7 | 0x80);
        }
    }
}

/// Decodes a LEB128-encoded arbitrary-precision unsigned integer from `buf` at `offset`.
///
/// Returns `(big_endian_bytes, bytes_consumed)`.
///
/// Errors:
/// - `CodecError::Truncated` if buffer ends mid-varint.
/// - `CodecError::VarintOverflow` if continuation bytes exceed `MAX_BYTES`.
pub(crate) fn read_bigint_varint(
    buf: &[u8],
    offset: usize,
) -> Result<(Vec<u8>, usize), CodecError> {
    // Collect LEB128 bytes, then reconstruct the big integer.
    let mut le_chunks: Vec<u8> = Vec::new(); // 7-bit chunks, little-endian order
    let mut bytes_read: usize = 0;

    loop {
        if bytes_read >= MAX_BYTES {
            return Err(CodecError::VarintOverflow(offset));
        }
        let pos = offset + bytes_read;
        if pos >= buf.len() {
            return Err(CodecError::Truncated {
                needed: pos + 1,
                had: buf.len(),
            });
        }
        let byte = buf[pos];
        bytes_read += 1;
        le_chunks.push(byte & 0x7F);
        if byte & 0x80 == 0 {
            break;
        }
    }

    // Reconstruct the integer from 7-bit LE chunks into a LE byte array,
    // then convert to big-endian.
    let total_bits = le_chunks.len() * 7;
    let byte_count = (total_bits + 7) / 8;
    let mut result_le = vec![0u8; byte_count];

    let mut bit_pos: usize = 0;
    for chunk in &le_chunks {
        let bits = *chunk as u16;
        let byte_idx = bit_pos / 8;
        let bit_off = (bit_pos % 8) as u16;

        if byte_idx < result_le.len() {
            result_le[byte_idx] |= ((bits << bit_off) & 0xFF) as u8;
        }
        if bit_off > 1 && byte_idx + 1 < result_le.len() {
            result_le[byte_idx + 1] |= (bits >> (8 - bit_off)) as u8;
        }
        bit_pos += 7;
    }

    // Convert to big-endian and strip leading zeros.
    result_le.reverse();
    let result = strip_leading_zeros(&result_le).to_vec();

    // An empty result means zero — return a single zero byte.
    if result.is_empty() {
        return Ok((vec![0], bytes_read));
    }

    Ok((result, bytes_read))
}

/// Reads a LEB128 varint as a length-style value and rejects any value that
/// exceeds `max` **before** narrowing to `usize`.
///
/// This guards the wasm32 target where `usize` is 32-bit: a `u64` varint of
/// `2^33` would silently truncate under a bare `as usize` cast. By rejecting
/// against `max` (always `<= usize::MAX` on every supported target) before the
/// cast, the narrowing is provably lossless.
///
/// Returns `(len, bytes_consumed)`.
///
/// Errors:
/// - `CodecError::Truncated` if the decoded value exceeds `max`.
/// - any error propagated from [`read_varint`] (truncated / overflow).
pub(crate) fn read_bounded_len(
    data: &[u8],
    offset: usize,
    max: usize,
) -> Result<(usize, usize), CodecError> {
    let (raw, consumed) = read_varint(data, offset)?;
    // Reject before casting: max as u64 is lossless (max <= usize::MAX always).
    if raw > max as u64 {
        return Err(CodecError::Truncated {
            needed: max.saturating_add(1),
            had: max,
        });
    }
    // Provably lossless: raw <= max <= usize::MAX.
    Ok((raw as usize, consumed))
}

// --- Private helpers -------------------------------------------------------

fn strip_leading_zeros(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    &bytes[start..]
}

/// Convert big-endian byte slice to a little-endian Vec<u8>.
fn to_le_bytes(be: &[u8]) -> Vec<u8> {
    let mut le = be.to_vec();
    le.reverse();
    le
}

/// Right-shift a little-endian byte array by 7 bits in place.
fn shr7_le(le: &mut Vec<u8>) {
    let mut carry: u16 = 0;
    for b in le.iter_mut().rev() {
        let val = (*b as u16) | (carry << 8);
        *b = (val >> 7) as u8;
        carry = val & 0x7F;
    }
    // Trim trailing zero bytes (which are the most-significant in LE).
    while le.len() > 1 && le[le.len() - 1] == 0 {
        le.pop();
    }
}

fn is_zero_le(le: &[u8]) -> bool {
    le.iter().all(|&b| b == 0)
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
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
}
