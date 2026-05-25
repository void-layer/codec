// Mantissa / quantity / U256 amount decoding and packed item unpacking.

use crate::error::CodecError;
use crate::invoice::InvoiceItem;
use crate::limits::{
    MAX_CANONICAL_QUANTITY_SCALE, MAX_ITEMS, MAX_SAFE_F64_INT, MAX_TRAILING_ZEROS, MAX_VALUE_SIZE,
};
use crate::varint::{read_bigint_varint, read_bounded_len, read_varint};

use super::dict::reverse_dict;

/// Maximum byte length of a single packed-item description value.
/// Bounds the per-item slice read against hostile varint lengths.
const MAX_DESC_LEN: usize = MAX_VALUE_SIZE;

/// Convert a big-endian mantissa byte slice + trailing-zero count to a decimal string.
/// `overflow_ctx` is used verbatim in error messages to identify the call site.
fn mantissa_to_decimal_string(
    mantissa_be: &[u8],
    zeros: u32,
    overflow_ctx: &str,
) -> Result<String, CodecError> {
    use ruint::aliases::U256;
    if mantissa_be.len() > 32 {
        return Err(CodecError::InvalidAmount(format!(
            "{overflow_ctx} mantissa varint too large: {} bytes exceeds U256",
            mantissa_be.len()
        )));
    }
    let mut be32 = [0u8; 32];
    be32[32 - mantissa_be.len()..].copy_from_slice(mantissa_be);
    let mantissa = U256::from_be_bytes(be32);
    let scale = U256::from(10u64).pow(U256::from(zeros));
    mantissa
        .checked_mul(scale)
        .map(|v| v.to_string())
        .ok_or_else(|| CodecError::InvalidAmount(format!("{overflow_ctx} overflow U256")))
}

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
        return Err(CodecError::Overflow(format!(
            "mantissa trailing zeros {zeros} exceeds maximum {MAX_TRAILING_ZEROS}"
        )));
    }
    mantissa_to_decimal_string(&mantissa_bytes, zeros, "amount")
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
        let desc_end = offset.checked_add(desc_len).ok_or(CodecError::Truncated {
            needed: usize::MAX,
            had: data.len(),
        })?;
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
        // Encoder caps at MAX_CANONICAL_QUANTITY_SCALE (limits.rs); reject anything above
        // as non-canonical — the decoder must not accept what the encoder cannot produce.
        if scale > MAX_CANONICAL_QUANTITY_SCALE as u32 {
            return Err(CodecError::InvalidData(format!(
                "non-canonical quantity scale {scale}: encoder cap is {MAX_CANONICAL_QUANTITY_SCALE}"
            )));
        }
        let (scaled_value, n) = read_varint(data, offset)?;
        offset += n;
        if scaled_value > MAX_SAFE_F64_INT {
            return Err(CodecError::InvalidAmount(format!(
                "scaled_value {scaled_value} exceeds f64 mantissa precision (2^53)"
            )));
        }
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
            return Err(CodecError::Overflow(format!(
                "item {i} rate zeros {zeros} exceeds max {MAX_TRAILING_ZEROS}"
            )));
        }

        let rate = mantissa_to_decimal_string(&mantissa_be, zeros, &format!("item {i} rate"))?;

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
