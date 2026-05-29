// EVM address + salt hex encoding, token-address dict encoding.
// Mirrors spec §5.2 token-address scheme.

use crate::error::CodecError;

fn hex_nibble(byte: u8, label: &str) -> Result<u8, CodecError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(CodecError::InvalidAddress(format!("invalid {label} hex"))),
    }
}

fn hex_decode_fixed<const N: usize>(hex: &str, label: &str) -> Result<[u8; N], CodecError> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != N * 2 {
        return Err(CodecError::InvalidAddress(format!(
            "{label} must be {} hex chars ({N} bytes), got {}",
            N * 2,
            hex.len()
        )));
    }
    let mut out = [0u8; N];
    for (i, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        out[i] = (hex_nibble(pair[0], label)? << 4) | hex_nibble(pair[1], label)?;
    }
    Ok(out)
}

/// Decode a 0x-prefixed hex address to 20 raw bytes.
pub(super) fn address_to_bytes(address: &str) -> Result<[u8; 20], CodecError> {
    hex_decode_fixed::<20>(address, "address")
}

/// Encode a token address per spec §5.2:
///   0x00 <code>  — dict known token
///   0x01 <20 bytes> — raw address
pub(super) fn encode_token_address(address: &str, network_id: u32) -> Result<Vec<u8>, CodecError> {
    use crate::dict::token::{CHAIN_CODE_RANGES, TOKEN_DICT};
    use crate::dict::{DICT_FORM, RAW_FORM};

    let addr_lower = address.to_lowercase();

    if let Some(&(code, _)) = TOKEN_DICT.iter().find(|&&(_, k)| k == addr_lower.as_str()) {
        // Mirrors TS encodeTokenAddress: if CHAIN_CODE_RANGES has an entry for this
        // chain and the code is outside that range, encode as raw bytes.
        // Unknown chain → no range constraint → dict-encode (mirrors TS reference).
        let in_range = CHAIN_CODE_RANGES
            .iter()
            .find(|&&(chain_id, _, _)| chain_id == network_id)
            .is_none_or(|&(_, min, max)| (min..=max).contains(&code));

        if in_range {
            return Ok(vec![DICT_FORM, code]);
        }
    }

    let raw = address_to_bytes(address)?;
    let mut val = vec![RAW_FORM];
    val.extend_from_slice(&raw);
    Ok(val)
}

/// Decode a 32-char hex string (16 bytes) into raw bytes for salt.
pub(super) fn hex_decode_salt(hex: &str) -> Result<Vec<u8>, CodecError> {
    hex_decode_fixed::<16>(hex, "salt").map(|a| a.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_to_bytes_valid() {
        let b = address_to_bytes("0xaabbccddee0011223344556677889900aabbccdd").unwrap();
        assert_eq!(b[0], 0xaa);
        assert_eq!(b[1], 0xbb);
        assert_eq!(b[19], 0xdd);
    }
}
