// Mantissa / quantity / U256 amount decoding and packed item unpacking.

use crate::error::CodecError;
use crate::invoice::InvoiceItem;
use crate::varint::{read_bigint_varint, read_varint};

use super::dict::reverse_dict;

const MAX_ITEMS: usize = 50;

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
    if zeros > 30 {
        return Err(CodecError::CompressionFailed(format!(
            "mantissa trailing zeros {zeros} exceeds maximum 30"
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
    let (count, n) = read_varint(data, offset)?;
    offset += n;
    let count = count as usize;
    if count > MAX_ITEMS {
        return Err(CodecError::CompressionFailed(format!(
            "item count {count} exceeds max {MAX_ITEMS}"
        )));
    }

    let mut items = Vec::with_capacity(count);
    for i in 0..count {
        // description length
        if offset >= data.len() {
            return Err(CodecError::Truncated {
                needed: offset + 1,
                had: data.len(),
            });
        }
        let (desc_len, n) = read_varint(data, offset)?;
        offset += n;
        let desc_len = desc_len as usize;
        if offset + desc_len > data.len() {
            return Err(CodecError::Truncated {
                needed: offset + desc_len,
                had: data.len(),
            });
        }
        let desc_bytes = &data[offset..offset + desc_len];
        let description = reverse_dict(desc_bytes)?;
        offset += desc_len;

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
        if zeros > 30 {
            return Err(CodecError::CompressionFailed(format!(
                "item {i} rate zeros {zeros} exceeds max 30"
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
