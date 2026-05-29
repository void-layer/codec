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
            // C-3: reject non-canonical encoding — a terminal byte of 0x00 with
            // preceding bytes means the value fits in fewer bytes (e.g. 0x80 0x00).
            if bytes_read > 1 && (byte & 0x7F) == 0 {
                return Err(CodecError::InvalidData("non-canonical varint".to_string()));
            }
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
            // C-3: reject non-canonical encoding — terminal 0x00 with preceding bytes.
            if bytes_read > 1 && (byte & 0x7F) == 0 {
                return Err(CodecError::InvalidData("non-canonical varint".to_string()));
            }
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

#[cfg(test)]
mod tests;
