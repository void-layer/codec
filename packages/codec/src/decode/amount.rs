// Mantissa / quantity / U256 amount decoding and packed item unpacking.

use crate::error::CodecError;
use crate::invoice::InvoiceItem;
use crate::varint::{read_bigint_varint, read_bounded_len, read_varint};

use super::dict::reverse_dict;

const MAX_ITEMS: usize = 50;

/// Maximum trailing-zero count for a mantissa-encoded amount.
/// A valid U256 has at most 77 decimal digits, so a base-10 value can carry
/// up to 77 trailing zeros (e.g. 10^77 < 2^256). Decode must accept any count
/// a valid U256 can produce — capping lower would reject valid encodings.
const MAX_TRAILING_ZEROS: u32 = 77;

/// Maximum byte length of a single packed-item description value.
/// Bounds the per-item slice read against hostile varint lengths.
const MAX_DESC_LEN: usize = 4096;

/// Decode mantissa-encoded amount from bytes (mirrors readMantissa from varint.ts).
/// Returns amount as a decimal string (BigInt-safe).
pub(super) fn decode_mantissa(bytes: &[u8]) -> Result<String, CodecError> {
    if bytes.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    let (mantissa_bytes, m_consumed) = read_bigint_varint(bytes, 0)?;
    let zeros_offset = m_consumed;
    if zeros_offset >= bytes.len() {
        return Err(CodecError::Truncated {
            needed: zeros_offset + 1,
            had: bytes.len(),
        });
    }
    let zeros = bytes[zeros_offset] as u32;
    if zeros > MAX_TRAILING_ZEROS {
        return Err(CodecError::CompressionFailed(format!(
            "mantissa trailing zeros {zeros} exceeds maximum {MAX_TRAILING_ZEROS}"
        )));
    }

    // Reconstruct value: mantissa_bytes is big-endian → U256
    use ruint::aliases::U256;
    if mantissa_bytes.len() > 32 {
        return Err(CodecError::InvalidAmount(format!(
            "mantissa varint too large: {} bytes exceeds U256",
            mantissa_bytes.len()
        )));
    }
    let mut be32 = [0u8; 32];
    be32[32 - mantissa_bytes.len()..].copy_from_slice(&mantissa_bytes);
    let mantissa = U256::from_be_bytes(be32);
    let scale = U256::from(10u64).pow(U256::from(zeros));
    let value = mantissa
        .checked_mul(scale)
        .ok_or_else(|| CodecError::InvalidAmount("amount overflow U256".to_string()))?;
    Ok(value.to_string())
}

/// Decode packed items from Type 14 binary format (mirrors unpackItems from decode.ts).
pub(super) fn unpack_items(data: &[u8]) -> Result<Vec<InvoiceItem>, CodecError> {
    let mut offset = 0;
    // Bounded read: rejects a hostile count varint before any usize narrowing.
    let (count, n) = read_bounded_len(data, offset, MAX_ITEMS)?;
    offset += n;

    let mut items = Vec::with_capacity(count);
    for i in 0..count {
        // description length
        if offset >= data.len() {
            return Err(CodecError::Truncated {
                needed: offset + 1,
                had: data.len(),
            });
        }
        // Bounded read: rejects a hostile desc_len varint before usize narrowing.
        let (desc_len, n) = read_bounded_len(data, offset, MAX_DESC_LEN)?;
        offset += n;
        // checked_add guards against offset + desc_len overflowing usize.
        let desc_end = offset
            .checked_add(desc_len)
            .ok_or(CodecError::Truncated { needed: usize::MAX, had: data.len() })?;
        if desc_end > data.len() {
            return Err(CodecError::Truncated {
                needed: desc_end,
                had: data.len(),
            });
        }
        let desc_bytes = &data[offset..desc_end];
        let description = reverse_dict(desc_bytes)?;
        offset = desc_end;

        // quantity: [scale: u8][scaled_value: varint]
        if offset >= data.len() {
            return Err(CodecError::Truncated {
                needed: offset + 1,
                had: data.len(),
            });
        }
        let scale = data[offset] as u32;
        offset += 1;
        let (scaled_value, n) = read_varint(data, offset)?;
        offset += n;
        let quantity = scaled_value as f64 / 10f64.powi(scale as i32);

        // rate: mantissa + trailing zeros
        let (mantissa_be, m_n) = read_bigint_varint(data, offset)?;
        offset += m_n;
        if offset >= data.len() {
            return Err(CodecError::Truncated {
                needed: offset + 1,
                had: data.len(),
            });
        }
        let zeros = data[offset] as u32;
        offset += 1;
        if zeros > MAX_TRAILING_ZEROS {
            return Err(CodecError::CompressionFailed(format!(
                "item {i} rate zeros {zeros} exceeds max {MAX_TRAILING_ZEROS}"
            )));
        }

        use ruint::aliases::U256;
        if mantissa_be.len() > 32 {
            return Err(CodecError::InvalidAmount(format!(
                "item {i} rate mantissa varint too large: {} bytes exceeds U256",
                mantissa_be.len()
            )));
        }
        let mut be32 = [0u8; 32];
        be32[32 - mantissa_be.len()..].copy_from_slice(&mantissa_be);
        let mantissa = U256::from_be_bytes(be32);
        let scale = U256::from(10u64).pow(U256::from(zeros));
        let rate = mantissa
            .checked_mul(scale)
            .ok_or_else(|| CodecError::InvalidAmount(format!("item {i} rate overflow U256")))?
            .to_string();

        items.push(InvoiceItem {
            description,
            quantity,
            rate,
        });
    }
    Ok(items)
}

// ---------------------------------------------------------------------------
// Test helpers (pub only under #[cfg(test)])
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests_pub {
    use super::*;

    pub(crate) fn decode_mantissa_pub(bytes: &[u8]) -> Result<String, CodecError> {
        decode_mantissa(bytes)
    }
}
