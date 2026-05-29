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
    // T2-2: trailing bytes inside TLV value — full consumption required.
    if zeros_offset + 1 != bytes.len() {
        return Err(CodecError::InvalidData(format!(
            "trailing bytes in amount TLV value: expected {} bytes, got {}",
            zeros_offset + 1,
            bytes.len()
        )));
    }
    // T2-1: mantissa scale-aliasing reject — canonical encoder always strips
    // trailing zeros into the zeros byte. mantissa%10==0 with mantissa!=0
    // means a trailing zero is in the mantissa instead of zeros.
    // mantissa==0 must have zeros==0 (canonical zero is [0x00, 0x00]).
    let mantissa_is_zero = mantissa_bytes.iter().all(|&b| b == 0);
    if mantissa_is_zero && zeros != 0 {
        return Err(CodecError::InvalidData(
            "non-canonical zero amount: mantissa=0 must have zeros=0".to_string(),
        ));
    }
    if !mantissa_is_zero {
        // Check if the last byte of the big-endian mantissa has trailing decimal
        // zeros. Since mantissa_bytes is big-endian, we need to check the numeric
        // value mod 10 — but we can do a fast check: last byte (LE digit) mod 10.
        // For a big-endian number, divisibility by 10 means divisible by 2 and 5.
        // We delegate to the U256 check via a simple last-nibble approach:
        // any number whose decimal form ends in 0 has last byte even AND divisible
        // by 5 in decimal. Rather than recomputing, parse via U256:
        use ruint::aliases::U256;
        let mut be32 = [0u8; 32];
        let start = 32usize.saturating_sub(mantissa_bytes.len());
        be32[start..].copy_from_slice(&mantissa_bytes[mantissa_bytes.len().saturating_sub(32)..]);
        let m = U256::from_be_bytes(be32);
        if m % U256::from(10u64) == U256::ZERO {
            return Err(CodecError::InvalidData(
                "non-canonical mantissa: trailing decimal zero must be in zeros byte".to_string(),
            ));
        }
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
        // T2-1: scale-aliasing reject for item rate mantissa.
        let mantissa_is_zero = mantissa_be.iter().all(|&b| b == 0);
        if mantissa_is_zero && zeros != 0 {
            return Err(CodecError::InvalidData(format!(
                "non-canonical zero rate in item {i}: mantissa=0 must have zeros=0"
            )));
        }
        if !mantissa_is_zero {
            use ruint::aliases::U256;
            let mut be32 = [0u8; 32];
            let start = 32usize.saturating_sub(mantissa_be.len());
            be32[start..].copy_from_slice(&mantissa_be[mantissa_be.len().saturating_sub(32)..]);
            let m = U256::from_be_bytes(be32);
            if m % U256::from(10u64) == U256::ZERO {
                return Err(CodecError::InvalidData(format!(
                    "non-canonical mantissa in item {i} rate: trailing decimal zero must be in zeros byte"
                )));
            }
        }

        let rate = mantissa_to_decimal_string(&mantissa_be, zeros, &format!("item {i} rate"))?;

        items.push(InvoiceItem {
            description,
            quantity,
            rate,
        });
    }
    // T2-2: trailing bytes inside TLV value — full consumption required.
    if offset != data.len() {
        return Err(CodecError::InvalidData(format!(
            "trailing bytes in items TLV value: consumed {offset} of {} bytes",
            data.len()
        )));
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
