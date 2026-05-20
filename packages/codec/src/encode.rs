// Mirrors vl/app/src/features/invoice-codec/lib/encode.ts
// and vl/app/src/shared/lib/tlv-codec/{writer.ts,varint.ts}.
//
// TLV type registry constants mirror tlv-map.ts TlvType enum.
// Encoding order: sort by TLV type ascending (BTreeMap), then append domain separator last.

use std::collections::BTreeMap;

use crate::dict::{app::APP_DICT, chain::CHAIN_DICT};
use crate::error::CodecError;
use crate::hash::keccak256;
use crate::tlv::write_tlv_stream;
use crate::varint::{write_bigint_varint, write_varint};

// ---------------------------------------------------------------------------
// TLV type numbers (mirrors tlv-map.ts TlvType)
// ---------------------------------------------------------------------------

// Optional (odd) types
pub(crate) const TLV_TOKEN_ADDRESS: u8 = 1;
pub(crate) const TLV_CLIENT_WALLET: u8 = 3;
pub(crate) const TLV_NOTES: u8 = 5;
pub(crate) const TLV_FROM_EMAIL: u8 = 7;
pub(crate) const TLV_FROM_PHONE: u8 = 9;
pub(crate) const TLV_FROM_ADDRESS: u8 = 11;
pub(crate) const TLV_CLIENT_EMAIL: u8 = 13;
pub(crate) const TLV_CLIENT_PHONE: u8 = 15;
pub(crate) const TLV_CLIENT_ADDRESS: u8 = 17;
pub(crate) const TLV_TAX: u8 = 19;
pub(crate) const TLV_DISCOUNT: u8 = 21;
pub(crate) const TLV_DOMAIN_SEPARATOR: u8 = 31;
pub(crate) const TLV_FROM_TAX_ID: u8 = 35;
pub(crate) const TLV_CLIENT_TAX_ID: u8 = 37;

// Required (even) types
pub(crate) const TLV_CHAIN_ID: u8 = 2;
pub(crate) const TLV_ISSUED_AT: u8 = 4;
pub(crate) const TLV_DUE_AT: u8 = 6;
pub(crate) const TLV_DECIMALS: u8 = 8;
pub(crate) const TLV_FROM_WALLET: u8 = 10;
pub(crate) const TLV_CURRENCY: u8 = 12;
pub(crate) const TLV_ITEMS: u8 = 14;
pub(crate) const TLV_FROM_NAME: u8 = 16;
pub(crate) const TLV_CLIENT_NAME: u8 = 18;
pub(crate) const TLV_SALT: u8 = 20;
pub(crate) const TLV_INVOICE_ID: u8 = 22;
pub(crate) const TLV_TOTAL: u8 = 24;

// Wire format constants
pub(crate) const MAGIC: u8 = 0x56; // 'V'
pub(crate) const VERSION: u8 = 0x01;
/// High bit of VERSION byte signals whole-payload Brotli compression (set by JS shim).
pub(crate) const COMPRESSED_FLAG: u8 = 0x80;

const MAX_TLV_COUNT: usize = 64;
const MAX_VALUE_SIZE: usize = 4096;
const MAX_PAYLOAD_SIZE: usize = 1481; // (2000 - 25 prefix) / 1.333 Base64url ratio

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Encode a UTF-8 string to bytes.
fn utf8_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Decode a 0x-prefixed hex address to 20 raw bytes.
fn address_to_bytes(address: &str) -> Result<[u8; 20], CodecError> {
    let hex = address.strip_prefix("0x").unwrap_or(address);
    if hex.len() != 40 {
        return Err(CodecError::BadMagic); // reuse: bad address treated as corrupt input
    }
    let mut out = [0u8; 20];
    for i in 0..20 {
        out[i] =
            u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|_| CodecError::BadMagic)?;
    }
    Ok(out)
}

/// Encode a u32 as 4-byte big-endian.
fn uint32_be(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Encode a u64 as LEB128 varint bytes.
fn varint_bytes(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    write_varint(value, &mut buf);
    buf
}

/// Encode a decimal integer string (BigInt) as mantissa + trailing-zeros.
/// Mirrors `writeMantissa` from varint.ts.
/// Amount domain is U256 — matches the on-chain uint256 domain and the TS BigInt reference.
fn mantissa_bytes(value_str: &str) -> Result<Vec<u8>, CodecError> {
    use ruint::aliases::U256;

    let value: U256 = U256::from_str_radix(value_str, 10)
        .map_err(|_| CodecError::InvalidAmount(value_str.to_string()))?;

    let mut buf = Vec::new();
    if value == U256::ZERO {
        // mantissa = 0 (single 0x00 byte), zeros = 0
        write_bigint_varint(&[0], &mut buf);
        buf.push(0);
        return Ok(buf);
    }

    let ten = U256::from(10u64);
    let mut mantissa = value;
    let mut zeros: u8 = 0;
    while mantissa % ten == U256::ZERO {
        mantissa /= ten;
        zeros += 1;
    }
    // Write mantissa as big-endian bytes via bigint_varint
    let mantissa_be: [u8; 32] = mantissa.to_be_bytes();
    write_bigint_varint(&mantissa_be, &mut buf);
    buf.push(zeros);
    Ok(buf)
}

/// Apply app-level dictionary substitution (mirrors applyDict from app-dict.ts).
/// Replaces known string patterns with 1-byte control codes.
/// Longest match first — iterate entries in length-descending order.
fn apply_dict(input: &str) -> Vec<u8> {
    // Sorted entries by key length descending (mirrors DICT_ENTRIES order in TS)
    // APP_DICT is a phf map; we must apply longest-match-first manually.
    let mut entries: Vec<(&str, u8)> = APP_DICT.entries().map(|(&k, &v)| (k, v)).collect();
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut text = input.to_string();
    for (pattern, code) in &entries {
        text = text.replace(pattern, &(String::from(char::from(*code))));
    }
    text.into_bytes()
}

/// Encode chain ID per chain-dict encoding scheme:
///   0x00 <code>   — known chain (dict lookup, 2 bytes)
///   0x01 <varint> — unknown chain (raw varint, 2+ bytes)
fn encode_chain_id(network_id: u32) -> Vec<u8> {
    if let Some(&code) = CHAIN_DICT.get(&network_id) {
        vec![0x00, code]
    } else {
        let mut buf = vec![0x01];
        write_varint(network_id as u64, &mut buf);
        buf
    }
}

/// Currency symbol → dict code (mirrors CURRENCY_DICT in tlv-map.ts). Static: zero per-call alloc.
static CURRENCY_SYMBOL_TO_CODE: &[(&str, u8)] = &[
    ("USDC", 1),
    ("USDT", 2),
    ("DAI", 3),
    ("ETH", 4),
    ("WETH", 5),
    ("MATIC", 6),
    ("POL", 7),
    ("WBTC", 8),
    ("USDC.E", 9),
    ("EURC", 10),
    ("USDT0", 11),
];

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

/// Encode currency per spec §5.1:
///   0x00 <code>  — dict known currency
///   0x01 <utf8>  — raw UTF-8
fn encode_currency(currency: &str) -> Vec<u8> {
    let upper = currency.to_uppercase();
    if let Some(&(_, code)) = CURRENCY_SYMBOL_TO_CODE
        .iter()
        .find(|&&(k, _)| k == upper.as_str())
    {
        vec![0x00, code]
    } else {
        let mut val = vec![0x01];
        val.extend_from_slice(currency.as_bytes());
        val
    }
}

/// Encode a token address per spec §5.2:
///   0x00 <code>  — dict known token
///   0x01 <20 bytes> — raw address
fn encode_token_address(address: &str, network_id: u32) -> Result<Vec<u8>, CodecError> {
    let addr_lower = address.to_lowercase();

    if let Some(&(_, code)) = TOKEN_ADDRESS_TO_CODE
        .iter()
        .find(|&&(k, _)| k == addr_lower.as_str())
    {
        // WETH at 0x4200…0006 is shared by Optimism (code 24) and Base (code 43).
        // On Base, override to 43 so the decoder resolves the correct chain context.
        let effective_code =
            if addr_lower == "0x4200000000000000000000000000000000000006" && network_id == 8453 {
                43u8
            } else {
                code
            };

        let in_range = CHAIN_CODE_RANGES
            .iter()
            .find(|&&(chain_id, _, _)| chain_id == network_id)
            .map(|&(_, min, max)| effective_code >= min && effective_code <= max)
            .unwrap_or(true); // unknown chain → no range restriction

        if in_range {
            return Ok(vec![0x00, effective_code]);
        }
    }

    let raw = address_to_bytes(address)?;
    let mut val = vec![0x01];
    val.extend_from_slice(&raw);
    Ok(val)
}

/// Encode items array into packed binary (Type 14, mirrors packItems from encode.ts).
/// Format: [count: varint] per item: [desc_len: varint][desc_bytes][qty: scale+varint][rate: mantissa]
fn pack_items(items: &[crate::invoice::InvoiceItem]) -> Result<Vec<u8>, CodecError> {
    let mut buf = Vec::new();
    write_varint(items.len() as u64, &mut buf);

    for item in items {
        // description: apply dict, then length-prefix with varint
        let desc_bytes = apply_dict(&item.description);
        write_varint(desc_bytes.len() as u64, &mut buf);
        buf.extend_from_slice(&desc_bytes);

        // quantity: [scale: u8][scaled_value: varint] — mirrors writeQuantity
        write_quantity(&mut buf, item.quantity);

        // rate: mantissa + trailing zeros — mirrors writeMantissa
        let rate_bytes = mantissa_bytes(&item.rate)?;
        buf.extend_from_slice(&rate_bytes);
    }
    Ok(buf)
}

/// Encode a fractional quantity as [scale: u8][scaled_value: varint].
/// Mirrors writeQuantity from varint.ts.
fn write_quantity(buf: &mut Vec<u8>, qty: f64) {
    let mut scale = 0u8;
    let mut scaled = qty;
    while scale < 9 && (scaled.round() - scaled).abs() > 1e-9 {
        scale += 1;
        scaled = qty * 10f64.powi(scale as i32);
    }
    let scaled_int = scaled.round() as u64;
    buf.push(scale);
    write_varint(scaled_int, buf);
}

/// Compute domain separator: keccak256("VOIDPAY_INVOICE_V1" || serialized TLV records except type 31).
/// Mirrors computeDomainSeparator from security.ts.
fn compute_domain_separator(records: &BTreeMap<u8, Vec<u8>>) -> Vec<u8> {
    let prefix = b"VOIDPAY_INVOICE_V1";
    let mut body: Vec<u8> = prefix.to_vec();

    // Serialize each record except domain separator (type 31) in key-ascending order
    for (&tlv_type, value) in records {
        if tlv_type == TLV_DOMAIN_SEPARATOR {
            continue;
        }
        // type(1) + length(varint) + value — mirrors TLV wire format
        body.push(tlv_type);
        write_varint(value.len() as u64, &mut body);
        body.extend_from_slice(value);
    }

    keccak256(&body).to_vec()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Encode an [`Invoice`] to canonical pre-compression bytes (payment identity).
///
/// The output is the raw TLV binary: `[MAGIC][VERSION][COUNT][TLV records...]`.
/// Feed the output to `compute_content_hash()` for ERC-3009 nonce binding.
/// The COMPRESSED_FLAG (0x80) is never set — compression lives in the JS shim layer.
///
/// # Errors
/// Returns [`CodecError`] if any field is malformed (bad address hex, invalid amount, etc.).
///
/// # Example
/// ```
/// use void_layer_codec::{encode_invoice_canonical, Invoice, InvoiceFrom, InvoiceClient, InvoiceItem};
/// let invoice = Invoice {
///     invoice_id: "INV-001".to_string(),
///     issued_at: 1_700_000_000,
///     due_at: 1_700_604_800,
///     network_id: 1,
///     currency: "USDC".to_string(),
///     decimals: 6,
///     from: InvoiceFrom {
///         name: "Alice".to_string(),
///         wallet_address: "0xaabbccddee0011223344556677889900aabbccdd".to_string(),
///         email: None, phone: None, physical_address: None, tax_id: None,
///     },
///     client: InvoiceClient {
///         name: "Bob".to_string(),
///         wallet_address: None, email: None, phone: None,
///         physical_address: None, tax_id: None,
///     },
///     items: vec![InvoiceItem {
///         description: "Work".to_string(), quantity: 1.0, rate: "1000000".to_string(),
///     }],
///     token_address: None, notes: None, tax: None, discount: None,
///     total: "1000000".to_string(),
///     salt: "00112233445566778899aabbccddeeff".to_string(),
/// };
/// let bytes = encode_invoice_canonical(&invoice).unwrap();
/// assert_eq!(bytes[0], 0x56); // magic
/// assert_eq!(bytes[1], 0x01); // version (no COMPRESSED_FLAG)
/// ```
pub fn encode_invoice_canonical(invoice: &crate::invoice::Invoice) -> Result<Vec<u8>, CodecError> {
    let mut map: BTreeMap<u8, Vec<u8>> = BTreeMap::new();

    // --- Required fields (even TLV types) ---

    // Chain ID (type 2)
    map.insert(TLV_CHAIN_ID, encode_chain_id(invoice.network_id));

    // Issued at (type 4): uint32 BE
    map.insert(TLV_ISSUED_AT, uint32_be(invoice.issued_at));

    // Due at (type 6): delta from issuedAt as varint
    let due_delta = invoice.due_at.saturating_sub(invoice.issued_at);
    map.insert(TLV_DUE_AT, varint_bytes(due_delta as u64));

    // Decimals (type 8): single byte
    map.insert(TLV_DECIMALS, vec![invoice.decimals]);

    // From wallet (type 10): 20 raw bytes
    let from_wallet = address_to_bytes(&invoice.from.wallet_address)?;
    map.insert(TLV_FROM_WALLET, from_wallet.to_vec());

    // Currency (type 12)
    map.insert(TLV_CURRENCY, encode_currency(&invoice.currency));

    // Items (type 14): packed binary
    map.insert(TLV_ITEMS, pack_items(&invoice.items)?);

    // From name (type 16): dict-applied UTF-8
    map.insert(TLV_FROM_NAME, apply_dict(&invoice.from.name));

    // Client name (type 18): dict-applied UTF-8
    map.insert(TLV_CLIENT_NAME, apply_dict(&invoice.client.name));

    // Salt (type 20): decode hex string → raw bytes
    let salt_bytes = hex_decode_salt(&invoice.salt)?;
    map.insert(TLV_SALT, salt_bytes);

    // Invoice ID (type 22): raw UTF-8 (NOT dict-applied per encode.ts comment)
    map.insert(TLV_INVOICE_ID, utf8_bytes(&invoice.invoice_id));

    // Total (type 24): mantissa-encoded
    map.insert(TLV_TOTAL, mantissa_bytes(&invoice.total)?);

    // --- Optional fields (odd TLV types) ---

    if let Some(ref addr) = invoice.token_address {
        map.insert(
            TLV_TOKEN_ADDRESS,
            encode_token_address(addr, invoice.network_id)?,
        );
    }

    if let Some(ref wallet) = invoice.client.wallet_address {
        let raw = address_to_bytes(wallet)?;
        map.insert(TLV_CLIENT_WALLET, raw.to_vec());
    }

    if let Some(ref notes) = invoice.notes {
        map.insert(TLV_NOTES, apply_dict(notes));
    }

    if let Some(ref email) = invoice.from.email {
        map.insert(TLV_FROM_EMAIL, apply_dict(email));
    }

    if let Some(ref phone) = invoice.from.phone {
        map.insert(TLV_FROM_PHONE, apply_dict(phone));
    }

    if let Some(ref addr) = invoice.from.physical_address {
        map.insert(TLV_FROM_ADDRESS, apply_dict(addr));
    }

    if let Some(ref tax_id) = invoice.from.tax_id {
        map.insert(TLV_FROM_TAX_ID, apply_dict(tax_id));
    }

    if let Some(ref email) = invoice.client.email {
        map.insert(TLV_CLIENT_EMAIL, apply_dict(email));
    }

    if let Some(ref phone) = invoice.client.phone {
        map.insert(TLV_CLIENT_PHONE, apply_dict(phone));
    }

    if let Some(ref addr) = invoice.client.physical_address {
        map.insert(TLV_CLIENT_ADDRESS, apply_dict(addr));
    }

    if let Some(ref tax_id) = invoice.client.tax_id {
        map.insert(TLV_CLIENT_TAX_ID, apply_dict(tax_id));
    }

    if let Some(ref tax) = invoice.tax {
        map.insert(TLV_TAX, utf8_bytes(tax));
    }

    if let Some(ref discount) = invoice.discount {
        map.insert(TLV_DISCOUNT, utf8_bytes(discount));
    }

    // Domain separator (type 31): computed over all other records
    let domain_sep = compute_domain_separator(&map);
    map.insert(TLV_DOMAIN_SEPARATOR, domain_sep);

    // Validate counts and sizes
    if map.len() > MAX_TLV_COUNT {
        return Err(CodecError::CompressionFailed(format!(
            "TLV count {} exceeds max {}",
            map.len(),
            MAX_TLV_COUNT
        )));
    }
    for value in map.values() {
        if value.len() > MAX_VALUE_SIZE {
            return Err(CodecError::CompressionFailed(format!(
                "TLV value size {} exceeds max {}",
                value.len(),
                MAX_VALUE_SIZE
            )));
        }
    }

    // Serialize: [MAGIC][VERSION][COUNT][TLV records in type-ascending order]
    let mut out = Vec::new();
    out.push(MAGIC);
    out.push(VERSION);
    out.push(map.len() as u8);
    write_tlv_stream(&map, &mut out);

    if out.len() > MAX_PAYLOAD_SIZE {
        return Err(CodecError::CompressionFailed(format!(
            "payload size {} exceeds max {}",
            out.len(),
            MAX_PAYLOAD_SIZE
        )));
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Private helpers (continued)
// ---------------------------------------------------------------------------

/// Decode a 32-char hex string (16 bytes) into raw bytes for salt.
fn hex_decode_salt(hex: &str) -> Result<Vec<u8>, CodecError> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 32 {
        return Err(CodecError::CompressionFailed(format!(
            "salt must be 32 hex chars (16 bytes), got {} chars",
            hex.len()
        )));
    }
    let mut bytes = Vec::with_capacity(16);
    for i in 0..16 {
        bytes.push(
            u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| CodecError::CompressionFailed("invalid salt hex".to_string()))?,
        );
    }
    Ok(bytes)
}

// ---------------------------------------------------------------------------
// Test helpers (pub only under #[cfg(test)])
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests_pub {
    use super::*;

    pub(crate) fn mantissa_bytes_pub(s: &str) -> Result<Vec<u8>, crate::error::CodecError> {
        mantissa_bytes(s)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice::InvoiceItem;

    #[test]
    fn mantissa_bytes_zero() {
        let b = mantissa_bytes("0").unwrap();
        // mantissa=0 → write_bigint_varint([0]) = [0x00], zeros=0
        assert_eq!(b, vec![0x00, 0x00]);
    }

    #[test]
    fn mantissa_bytes_one_million() {
        // 1_000_000 = 1 * 10^6 → mantissa=1 (0x01), zeros=6
        let b = mantissa_bytes("1000000").unwrap();
        assert_eq!(b, vec![0x01, 0x06]);
    }

    #[test]
    fn mantissa_bytes_123() {
        // 123 — no trailing zeros → mantissa=123, zeros=0
        // 123 = 0x7B → LEB128 single byte
        let b = mantissa_bytes("123").unwrap();
        assert_eq!(b, vec![0x7b, 0x00]);
    }

    #[test]
    fn write_quantity_integer_one() {
        let mut buf = Vec::new();
        write_quantity(&mut buf, 1.0);
        // scale=0, value=1 → [0x00, 0x01]
        assert_eq!(buf, vec![0x00, 0x01]);
    }

    #[test]
    fn write_quantity_1_5() {
        let mut buf = Vec::new();
        write_quantity(&mut buf, 1.5);
        // scale=1, value=15 → [0x01, 0x0F]
        assert_eq!(buf, vec![0x01, 0x0F]);
    }

    #[test]
    fn encode_chain_id_known_ethereum() {
        let b = encode_chain_id(1);
        assert_eq!(b, vec![0x00, 0x01]);
    }

    #[test]
    fn encode_chain_id_unknown() {
        let b = encode_chain_id(999999);
        assert_eq!(b[0], 0x01, "unknown chain prefix must be 0x01");
        assert!(b.len() > 1, "must include varint after prefix");
    }

    #[test]
    fn encode_currency_known_usdc() {
        let b = encode_currency("USDC");
        assert_eq!(b, vec![0x00, 0x01]);
    }

    #[test]
    fn encode_currency_unknown() {
        let b = encode_currency("XYZ");
        assert_eq!(b[0], 0x01);
        assert_eq!(&b[1..], b"XYZ");
    }

    #[test]
    fn address_to_bytes_valid() {
        let b = address_to_bytes("0xaabbccddee0011223344556677889900aabbccdd").unwrap();
        assert_eq!(b[0], 0xaa);
        assert_eq!(b[1], 0xbb);
        assert_eq!(b[19], 0xdd);
    }

    #[test]
    fn pack_items_single_item() {
        let items = vec![InvoiceItem {
            description: "Work".to_string(),
            quantity: 1.0,
            rate: "1000000".to_string(),
        }];
        let b = pack_items(&items).unwrap();
        // count = 1 (varint 0x01)
        assert_eq!(b[0], 0x01);
    }

    #[test]
    fn apply_dict_substitutes_pattern() {
        let result = apply_dict("Invoice total");
        // "Invoice" → 0x06
        assert_eq!(result[0], 0x06);
    }

    #[test]
    fn apply_dict_no_match_passthrough() {
        let result = apply_dict("Hello world");
        assert_eq!(result, b"Hello world");
    }

    // --- U256 amount domain tests ---

    #[test]
    fn mantissa_bytes_u128_max() {
        // u128::MAX = 340282366920938463463374607431768211455
        // Must produce byte-identical output to the old u128 path.
        let s = u128::MAX.to_string();
        let b = mantissa_bytes(&s).unwrap();
        // Verify encode→decode roundtrip produces the same string.
        // Spot-check: no trailing zeros, so zeros byte = 0.
        assert_eq!(*b.last().unwrap(), 0u8, "u128::MAX has no trailing zeros");
    }

    #[test]
    fn mantissa_bytes_u256_max_roundtrips() {
        // 2^256 - 1 as decimal
        let uint256_max =
            "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        let b = mantissa_bytes(uint256_max).unwrap();
        // Last byte = trailing zeros count (should be 0 — u256::MAX is odd)
        assert_eq!(*b.last().unwrap(), 0u8);
        // Verify the encoded bytes decode back (via decode_mantissa)
        let decoded = crate::decode::tests_pub::decode_mantissa_pub(&b).unwrap();
        assert_eq!(decoded, uint256_max);
    }

    #[test]
    fn mantissa_bytes_large_round_value() {
        // 10^30 — large round value well above u128::MAX range in theory but fits U256
        let s = "1".to_string() + &"0".repeat(30);
        let b = mantissa_bytes(&s).unwrap();
        // mantissa = 1, zeros = 30
        assert_eq!(*b.last().unwrap(), 30u8);
    }

    #[test]
    fn mantissa_bytes_above_u256_errors() {
        // 2^256 — one above U256::MAX
        let over = "115792089237316195423570985008687907853269984665640564039457584007913129639936";
        let err = mantissa_bytes(over).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount, got {err:?}"
        );
    }

    #[test]
    fn mantissa_bytes_non_numeric_errors() {
        let err = mantissa_bytes("not_a_number").unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::InvalidAmount(_)),
            "expected InvalidAmount, got {err:?}"
        );
    }
}
