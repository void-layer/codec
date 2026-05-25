// EVM address + salt hex encoding, token-address dict encoding.
// Mirrors spec §5.2 token-address scheme.

use crate::error::CodecError;

/// Decode a 0x-prefixed hex address to 20 raw bytes.
pub(super) fn address_to_bytes(address: &str) -> Result<[u8; 20], CodecError> {
    let hex = address.strip_prefix("0x").unwrap_or(address);
    if hex.len() != 40 {
        return Err(CodecError::InvalidAddress(format!(
            "address must be 40 hex chars (20 bytes), got {}",
            hex.len()
        )));
    }
    let hex_bytes = hex.as_bytes();
    if !hex_bytes.iter().all(|b| b.is_ascii()) {
        return Err(CodecError::InvalidAddress(
            "invalid address hex".to_string(),
        ));
    }
    let mut out = [0u8; 20];
    for i in 0..20 {
        let slice = std::str::from_utf8(&hex_bytes[i * 2..i * 2 + 2])
            .map_err(|_| CodecError::InvalidAddress("invalid address hex".to_string()))?;
        out[i] = u8::from_str_radix(slice, 16)
            .map_err(|_| CodecError::InvalidAddress("invalid address hex".to_string()))?;
    }
    Ok(out)
}

/// Token address → dict code (mirrors TOKEN_DICT in tlv-map.ts). Static: zero per-call alloc.
/// WETH on Optimism and Base share address 0x4200…0006; Base gets code 43 via chain range check.
static TOKEN_ADDRESS_TO_CODE: &[(&str, u8)] = &[
    ("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", 1),
    ("0xdac17f958d2ee523a2206206994597c13d831ec7", 2),
    ("0x6b175474e89094c44da98b954eedeac495271d0f", 3),
    ("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", 4),
    ("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599", 5),
    ("0x1abaea1f7c830bd89acc67ec4af516284b1bc33c", 6),
    ("0x6c96de32cea08842dcc4058c14d3aaad7fa41dee", 7),
    ("0xaf88d065e77c8cc2239327c5edb3a432268e5831", 10),
    ("0xff970a61a04b1ca14834a43f5de4533ebddb5cc8", 11),
    ("0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9", 12),
    ("0xda10009cbd5d07dd0cecc66161fc93d7c9000da1", 13),
    ("0x82af49447d8a07e3bd95bd0d56f35241523fbab1", 14),
    ("0x2f2a2543b76a4166549f7aab2e75bef0aefc5b0f", 15),
    ("0x0b2c639c533813f4aa9d7837caf62653d097ff85", 20),
    ("0x7f5c764cbc14f9669b88837ca1490cca17c31607", 21),
    ("0x94b008aa00579c1307b0ef2c499ad98a8ce58e58", 22),
    ("0x4200000000000000000000000000000000000006", 24), // op=24 by default; base=43 via chain check
    ("0x68f180fcce6836688e9084f035309e29bf0a2095", 25),
    ("0x3c499c542cef5e3811e1192ce70d8cc03d5c3359", 30),
    ("0x2791bca1f2de4661ed88a30c99a7a9449aa84174", 31),
    ("0xc2132d05d31c914a87c6611c10748aeb04b58e8f", 32),
    ("0x8f3cf7ad23cd3cadbd9735aff958023239c6a063", 33),
    ("0x7ceb23fd6bc0add59e62ac25578270cff1b9f619", 34),
    ("0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6", 35),
    ("0x833589fcd6edb6e08f4c7c32d4f71b54bda02913", 40),
    ("0xd9aaec86b65d86f6a7b5b1b0c42ffa531710b6ca", 41),
    ("0x50c5725949a6f0c72e6c4a641f24049a917db0cb", 42),
    ("0x0555e30da8f98308edb960aa94c0ed47230d2b9c", 44),
    ("0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42", 45),
];

/// Chain ID → (code_min, code_max) range for token dict validation.
static CHAIN_CODE_RANGES: &[(u32, u8, u8)] = &[
    (1, 1, 9),
    (42161, 10, 19),
    (10, 20, 29),
    (137, 30, 39),
    (8453, 40, 49),
];

/// Encode a token address per spec §5.2:
///   0x00 <code>  — dict known token
///   0x01 <20 bytes> — raw address
pub(super) fn encode_token_address(address: &str, network_id: u32) -> Result<Vec<u8>, CodecError> {
    let addr_lower = address.to_lowercase();

    if let Some(&(_, code)) = TOKEN_ADDRESS_TO_CODE
        .iter()
        .find(|&&(k, _)| k == addr_lower.as_str())
    {
        // Mirrors TS encodeTokenAddress: if CHAIN_CODE_RANGES has an entry for this
        // chain and the code is outside that range, encode as raw bytes.
        // If the chain is unknown (not in CHAIN_CODE_RANGES), encode as dict — mirrors
        // TS: `if (range && ...)` is falsy for undefined → returns entry → dict encode.
        let maybe_range = CHAIN_CODE_RANGES
            .iter()
            .find(|&&(chain_id, _, _)| chain_id == network_id)
            .map(|&(_, min, max)| (min, max));

        let in_range = match maybe_range {
            Some((min, max)) => code >= min && code <= max,
            None => true, // unknown chain: no range constraint → dict-encode
        };

        if in_range {
            return Ok(vec![0x00, code]);
        }
    }

    let raw = address_to_bytes(address)?;
    let mut val = vec![0x01];
    val.extend_from_slice(&raw);
    Ok(val)
}

/// Decode a 32-char hex string (16 bytes) into raw bytes for salt.
pub(super) fn hex_decode_salt(hex: &str) -> Result<Vec<u8>, CodecError> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 32 {
        return Err(CodecError::InvalidAddress(format!(
            "salt must be 32 hex chars (16 bytes), got {} chars",
            hex.len()
        )));
    }
    let hex_bytes = hex.as_bytes();
    if !hex_bytes.iter().all(|b| b.is_ascii()) {
        return Err(CodecError::InvalidAddress("invalid salt hex".to_string()));
    }
    let mut bytes = Vec::with_capacity(16);
    for i in 0..16 {
        let slice = std::str::from_utf8(&hex_bytes[i * 2..i * 2 + 2])
            .map_err(|_| CodecError::InvalidAddress("invalid salt hex".to_string()))?;
        bytes.push(
            u8::from_str_radix(slice, 16)
                .map_err(|_| CodecError::InvalidAddress("invalid salt hex".to_string()))?,
        );
    }
    Ok(bytes)
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
