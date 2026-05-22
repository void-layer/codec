use tiny_keccak::{Hasher, Keccak};

/// Keccak-256 over `bytes`. Returns the 32-byte digest.
pub(crate) fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut k = Keccak::v256();
    k.update(bytes);
    let mut out = [0u8; 32];
    k.finalize(&mut out);
    out
}

/// Compute the content hash for ERC-3009 nonce binding (spec §0.2).
///
/// Input MUST be the canonical pre-compression binary (the TLV form), NOT wire bytes.
///
/// # Example
/// ```
/// use void_layer_codec::compute_content_hash;
/// let hash = compute_content_hash(b"hello");
/// assert_eq!(hash.len(), 32);
/// ```
pub fn compute_content_hash(canonical_binary: &[u8]) -> [u8; 32] {
    keccak256(canonical_binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_to_bytes(hex: &str) -> [u8; 32] {
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    #[test]
    fn keccak256_empty() {
        // keccak256("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
        let expected =
            hex_to_bytes("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
        assert_eq!(keccak256(b""), expected);
    }

    #[test]
    fn keccak256_abc() {
        // keccak256("abc") = 4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45
        let expected =
            hex_to_bytes("4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45");
        assert_eq!(keccak256(b"abc"), expected);
    }

    #[test]
    fn compute_content_hash_stable() {
        // Hand-crafted canonical TLV sample: tag=0x01, length=0x03, value=[0xAA, 0xBB, 0xCC]
        let canonical_binary: &[u8] = &[0x01, 0x03, 0xAA, 0xBB, 0xCC];
        let hash1 = compute_content_hash(canonical_binary);
        let hash2 = compute_content_hash(canonical_binary);
        // Deterministic: same input always yields same 32-byte digest
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
        // Different input yields different digest
        let other = compute_content_hash(&[0x01, 0x03, 0xAA, 0xBB, 0xCD]);
        assert_ne!(hash1, other);
    }
}
