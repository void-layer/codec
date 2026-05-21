// Hex encoding of raw byte slices for address / salt fields.

use crate::error::CodecError;

/// Decode 20 raw bytes to a 0x-prefixed lowercase hex address.
pub(super) fn bytes_to_address(bytes: &[u8]) -> Result<String, CodecError> {
    if bytes.len() != 20 {
        return Err(CodecError::Truncated {
            needed: 20,
            had: bytes.len(),
        });
    }
    use std::fmt::Write as _;
    let mut hex = String::with_capacity(42);
    hex.push_str("0x");
    for b in bytes {
        let _ = write!(hex, "{b:02x}");
    }
    Ok(hex)
}

/// Decode raw bytes to a lowercase hex string (for salt, arbitrary length).
pub(super) fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hex = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(hex, "{b:02x}");
    }
    hex
}
