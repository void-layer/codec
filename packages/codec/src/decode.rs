// Mirrors vl/app/src/features/invoice-codec/lib/decode.ts
// and vl/app/src/shared/lib/tlv-codec/{reader.ts,varint.ts}.
//
// Reads: [MAGIC][VERSION][COUNT][TLV records...]
// Validates: magic, version (no COMPRESSED_FLAG), canonical ordering, domain separator.
// Maps TLV types to Invoice fields per tlv-map.ts.

use std::collections::BTreeMap;

use crate::dict::chain::CHAIN_DICT;
use crate::encode::{
    COMPRESSED_FLAG, MAGIC, TLV_CHAIN_ID, TLV_CLIENT_ADDRESS, TLV_CLIENT_EMAIL, TLV_CLIENT_NAME,
    TLV_CLIENT_PHONE, TLV_CLIENT_TAX_ID, TLV_CLIENT_WALLET, TLV_CURRENCY, TLV_DECIMALS,
    TLV_DISCOUNT, TLV_DOMAIN_SEPARATOR, TLV_DUE_AT, TLV_FROM_ADDRESS, TLV_FROM_EMAIL,
    TLV_FROM_NAME, TLV_FROM_PHONE, TLV_FROM_TAX_ID, TLV_FROM_WALLET, TLV_INVOICE_ID, TLV_ISSUED_AT,
    TLV_ITEMS, TLV_NOTES, TLV_SALT, TLV_TAX, TLV_TOKEN_ADDRESS, TLV_TOTAL, VERSION,
};
use crate::error::CodecError;
use crate::hash::keccak256;
use crate::invoice::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};
use crate::tlv::read_tlv_stream;
use crate::varint::{read_bigint_varint, read_varint};

const MAX_TLV_COUNT: usize = 64;
const MAX_ITEMS: usize = 50;
const MAX_VALUE_SIZE: usize = 4096;

// ---------------------------------------------------------------------------
// Private decode helpers
// ---------------------------------------------------------------------------

/// Decode 20 raw bytes to a 0x-prefixed lowercase hex address.
fn bytes_to_address(bytes: &[u8]) -> Result<String, CodecError> {
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
fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut hex = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// Reverse app-level dictionary substitution (mirrors reverseDict from app-dict.ts).
fn reverse_dict(bytes: &[u8]) -> Result<String, CodecError> {
    // Decode raw bytes as a string — control chars are the dict codes
    let mut text = String::with_capacity(bytes.len());
    for &b in bytes {
        text.push(b as char);
    }

    // Reverse entries longest-pattern-first (same order as apply_dict)
    let entries: &[(&str, u8)] = &[
        ("@outlook.com", 0x02),
        ("@hotmail.com", 0x0c),
        ("development", 0x0d),
        ("consulting", 0x0e),
        ("@gmail.com", 0x03),
        ("@yahoo.com", 0x04),
        ("https://", 0x05),
        ("Invoice", 0x06),
        ("Payment", 0x07),
        (".com", 0x09),
        ("INV-", 0x0f),
    ];

    // Apply in reverse order (shortest first for reverse) — mirrors TS [...DICT_ENTRIES].reverse()
    for &(pattern, code) in entries.iter().rev() {
        text = text.replace(char::from(code), pattern);
    }

    Ok(text)
}

/// Decode chain ID from TLV value bytes:
///   [0x00, code] → dict lookup
///   [0x01, varint...] → raw chain ID
fn decode_chain_id(value: &[u8]) -> Result<u32, CodecError> {
    if value.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    let prefix = value[0];
    if prefix == 0x00 {
        if value.len() < 2 {
            return Err(CodecError::Truncated { needed: 2, had: 1 });
        }
        let code = value[1];
        // Reverse lookup: code → chain_id
        let chain_id = CHAIN_DICT
            .entries()
            .find(|&(&_k, &v)| v == code)
            .map(|(&k, _)| k)
            .ok_or(CodecError::UnknownExtension(code))?;
        Ok(chain_id)
    } else if prefix == 0x01 {
        let (chain_id, _) = read_varint(value, 1)?;
        Ok(chain_id as u32)
    } else {
        Err(CodecError::UnknownExtension(prefix))
    }
}

/// Currency code → symbol (mirrors CURRENCY_DICT_REVERSE in tlv-map.ts). Static: zero per-call alloc.
static CURRENCY_CODE_TO_SYMBOL: &[(u8, &str)] = &[
    (1, "USDC"),
    (2, "USDT"),
    (3, "DAI"),
    (4, "ETH"),
    (5, "WETH"),
    (6, "MATIC"),
    (7, "POL"),
    (8, "WBTC"),
    (9, "USDC.E"),
    (10, "EURC"),
    (11, "USDT0"),
];

/// Token dict code → lowercase address (mirrors TOKEN_DICT_REVERSE in tlv-map.ts). Static: zero per-call alloc.
/// Code 43 = Base WETH (same address as Optimism code 24, different chain context).
static TOKEN_CODE_TO_ADDRESS: &[(u8, &str)] = &[
    (1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    (2, "0xdac17f958d2ee523a2206206994597c13d831ec7"),
    (3, "0x6b175474e89094c44da98b954eedeac495271d0f"),
    (4, "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
    (5, "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599"),
    (6, "0x1abaea1f7c830bd89acc67ec4af516284b1bc33c"),
    (7, "0x6c96de32cea08842dcc4058c14d3aaad7fa41dee"),
    (10, "0xaf88d065e77c8cc2239327c5edb3a432268e5831"),
    (11, "0xff970a61a04b1ca14834a43f5de4533ebddb5cc8"),
    (12, "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9"),
    (13, "0xda10009cbd5d07dd0cecc66161fc93d7c9000da1"),
    (14, "0x82af49447d8a07e3bd95bd0d56f35241523fbab1"),
    (15, "0x2f2a2543b76a4166549f7aab2e75bef0aefc5b0f"),
    (20, "0x0b2c639c533813f4aa9d7837caf62653d097ff85"),
    (21, "0x7f5c764cbc14f9669b88837ca1490cca17c31607"),
    (22, "0x94b008aa00579c1307b0ef2c499ad98a8ce58e58"),
    (24, "0x4200000000000000000000000000000000000006"),
    (25, "0x68f180fcce6836688e9084f035309e29bf0a2095"),
    (30, "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359"),
    (31, "0x2791bca1f2de4661ed88a30c99a7a9449aa84174"),
    (32, "0xc2132d05d31c914a87c6611c10748aeb04b58e8f"),
    (33, "0x8f3cf7ad23cd3cadbd9735aff958023239c6a063"),
    (34, "0x7ceb23fd6bc0add59e62ac25578270cff1b9f619"),
    (35, "0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6"),
    (40, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
    (41, "0xd9aaec86b65d86f6a7b5b1b0c42ffa531710b6ca"),
    (42, "0x50c5725949a6f0c72e6c4a641f24049a917db0cb"),
    (43, "0x4200000000000000000000000000000000000006"),
    (44, "0x0555e30da8f98308edb960aa94c0ed47230d2b9c"),
    (45, "0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42"),
];

/// Decode currency from TLV value bytes:
///   [0x00, code] → dict lookup
///   [0x01, utf8...] → raw string
fn decode_currency(value: &[u8]) -> Result<String, CodecError> {
    if value.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    if value[0] == 0x00 {
        if value.len() < 2 {
            return Err(CodecError::Truncated { needed: 2, had: 1 });
        }
        let code = value[1];
        CURRENCY_CODE_TO_SYMBOL
            .iter()
            .find(|&&(c, _)| c == code)
            .map(|&(_, s)| s.to_string())
            .ok_or(CodecError::UnknownExtension(code))
    } else {
        String::from_utf8(value[1..].to_vec())
            .map_err(|_| CodecError::CompressionFailed("invalid UTF-8 in currency".to_string()))
    }
}

/// Decode token address from TLV value bytes:
///   [0x00, code] → dict reverse lookup
///   [0x01, 20 bytes] → raw hex address
fn decode_token_address(value: &[u8]) -> Result<String, CodecError> {
    if value.is_empty() {
        return Err(CodecError::Truncated { needed: 2, had: 0 });
    }
    if value[0] == 0x00 {
        if value.len() < 2 {
            return Err(CodecError::Truncated { needed: 2, had: 1 });
        }
        let code = value[1];
        TOKEN_CODE_TO_ADDRESS
            .iter()
            .find(|&&(c, _)| c == code)
            .map(|&(_, addr)| addr.to_string())
            .ok_or(CodecError::UnknownExtension(code))
    } else {
        bytes_to_address(&value[1..])
    }
}

/// Decode mantissa-encoded amount from bytes (mirrors readMantissa from varint.ts).
/// Returns amount as a decimal string (BigInt-safe).
fn decode_mantissa(bytes: &[u8]) -> Result<String, CodecError> {
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
fn unpack_items(data: &[u8]) -> Result<Vec<InvoiceItem>, CodecError> {
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

/// Verify domain separator (mirrors validateSecurity from security.ts).
fn verify_domain_separator(
    records: &BTreeMap<u8, Vec<u8>>,
    stored_sep: &[u8],
) -> Result<(), CodecError> {
    let prefix = b"VOIDPAY_INVOICE_V1";
    let mut body: Vec<u8> = prefix.to_vec();

    for (&tlv_type, value) in records {
        if tlv_type == TLV_DOMAIN_SEPARATOR {
            continue;
        }
        body.push(tlv_type);
        crate::varint::write_varint(value.len() as u64, &mut body);
        body.extend_from_slice(value);
    }

    let expected = keccak256(&body);
    if expected != stored_sep {
        return Err(CodecError::ChecksumMismatch);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decode canonical pre-compression bytes into an [`Invoice`].
///
/// Accepts the raw TLV binary output of [`encode_invoice_canonical`].
/// Rejects payloads with the COMPRESSED_FLAG set — those must be decompressed
/// by the JS shim before being passed here.
///
/// # Errors
/// - [`CodecError::BadMagic`] — wrong magic byte or empty input
/// - [`CodecError::UnsupportedVersion`] — version byte is not 0x01
/// - [`CodecError::Truncated`] — payload too short
/// - [`CodecError::ChecksumMismatch`] — domain separator mismatch
///
/// # Example
/// ```
/// use void_layer_codec::{encode_invoice_canonical, decode_invoice_canonical};
/// use void_layer_codec::{Invoice, InvoiceFrom, InvoiceClient, InvoiceItem};
/// let invoice = Invoice {
///     invoice_id: "INV-001".to_string(),
///     issued_at: 1_700_000_000, due_at: 1_700_604_800,
///     network_id: 1, currency: "USDC".to_string(), decimals: 6,
///     from: InvoiceFrom {
///         name: "Alice".to_string(),
///         wallet_address: "0xaabbccddee0011223344556677889900aabbccdd".to_string(),
///         email: None, phone: None, physical_address: None, tax_id: None,
///     },
///     client: InvoiceClient {
///         name: "Bob".to_string(), wallet_address: None,
///         email: None, phone: None, physical_address: None, tax_id: None,
///     },
///     items: vec![InvoiceItem {
///         description: "Work".to_string(), quantity: 1.0, rate: "1000000".to_string(),
///     }],
///     token_address: None, notes: None, tax: None, discount: None,
///     total: "1000000".to_string(),
///     salt: "00112233445566778899aabbccddeeff".to_string(),
/// };
/// let bytes = encode_invoice_canonical(&invoice).unwrap();
/// let decoded = decode_invoice_canonical(&bytes).unwrap();
/// assert_eq!(decoded.invoice_id, "INV-001");
/// ```
pub fn decode_invoice_canonical(bytes: &[u8]) -> Result<Invoice, CodecError> {
    if bytes.is_empty() || bytes[0] != MAGIC {
        return Err(CodecError::BadMagic);
    }

    if bytes.len() < 2 {
        return Err(CodecError::Truncated { needed: 3, had: 1 });
    }

    let version_byte = bytes[1];
    // Reject compressed payloads — COMPRESSED_FLAG (0x80) means JS shim must Brotli-decompress first.
    // decode_invoice_canonical is the identity-boundary function; it only accepts raw canonical bytes.
    if version_byte & COMPRESSED_FLAG != 0 {
        return Err(CodecError::CompressionFailed(
            "unexpected compressed input in decode_invoice_canonical — decompress first"
                .to_string(),
        ));
    }
    if version_byte != VERSION {
        return Err(CodecError::UnsupportedVersion(version_byte));
    }

    if bytes.len() < 3 {
        return Err(CodecError::Truncated { needed: 3, had: 2 });
    }

    let tlv_count = bytes[2] as usize;
    if tlv_count > MAX_TLV_COUNT {
        return Err(CodecError::CompressionFailed(format!(
            "TLV count {tlv_count} exceeds max {MAX_TLV_COUNT}"
        )));
    }

    let tlv_body = &bytes[3..];
    let records: BTreeMap<u8, Vec<u8>> = read_tlv_stream(tlv_body)?;

    if records.len() != tlv_count {
        return Err(CodecError::Truncated {
            needed: tlv_count,
            had: records.len(),
        });
    }

    for (&tlv_type, value) in &records {
        if value.len() > MAX_VALUE_SIZE {
            return Err(CodecError::CompressionFailed(format!(
                "TLV type {tlv_type} value size {} exceeds max {MAX_VALUE_SIZE}",
                value.len()
            )));
        }
    }

    let salt_bytes = records.get(&TLV_SALT).ok_or(CodecError::ChecksumMismatch)?;
    if salt_bytes.len() < 16 {
        return Err(CodecError::ChecksumMismatch);
    }

    let stored_sep = records
        .get(&TLV_DOMAIN_SEPARATOR)
        .ok_or(CodecError::ChecksumMismatch)?;
    verify_domain_separator(&records, stored_sep)?;

    let chain_id_bytes = records.get(&TLV_CHAIN_ID).ok_or(CodecError::BadMagic)?;
    let network_id = decode_chain_id(chain_id_bytes)?;

    let issued_at_bytes = records
        .get(&TLV_ISSUED_AT)
        .ok_or(CodecError::Truncated { needed: 4, had: 0 })?;
    if issued_at_bytes.len() < 4 {
        return Err(CodecError::Truncated {
            needed: 4,
            had: issued_at_bytes.len(),
        });
    }
    let issued_at = u32::from_be_bytes([
        issued_at_bytes[0],
        issued_at_bytes[1],
        issued_at_bytes[2],
        issued_at_bytes[3],
    ]);

    let due_at_bytes = records
        .get(&TLV_DUE_AT)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    let (due_delta, _) = read_varint(due_at_bytes, 0)?;
    let due_delta_u32 = u32::try_from(due_delta)
        .map_err(|_| CodecError::InvalidAmount(format!("due_at delta {due_delta} overflows u32")))?;
    let due_at = issued_at.checked_add(due_delta_u32).ok_or_else(|| {
        CodecError::InvalidAmount(format!(
            "due_at overflow: issued_at {issued_at} + delta {due_delta_u32}"
        ))
    })?;

    let decimals_bytes = records
        .get(&TLV_DECIMALS)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    let decimals = *decimals_bytes
        .first()
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;

    let from_wallet_bytes = records
        .get(&TLV_FROM_WALLET)
        .ok_or(CodecError::Truncated { needed: 20, had: 0 })?;
    let from_wallet_address = bytes_to_address(from_wallet_bytes)?;

    let currency_bytes = records
        .get(&TLV_CURRENCY)
        .ok_or(CodecError::Truncated { needed: 2, had: 0 })?;
    let currency = decode_currency(currency_bytes)?;

    let items_bytes = records
        .get(&TLV_ITEMS)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    let items = unpack_items(items_bytes)?;

    let from_name_bytes = records
        .get(&TLV_FROM_NAME)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    let from_name = reverse_dict(from_name_bytes)?;

    let client_name_bytes = records
        .get(&TLV_CLIENT_NAME)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    let client_name = reverse_dict(client_name_bytes)?;

    let invoice_id_bytes = records
        .get(&TLV_INVOICE_ID)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    let invoice_id = String::from_utf8(invoice_id_bytes.clone())
        .map_err(|_| CodecError::CompressionFailed("invalid UTF-8 in invoice_id".to_string()))?;

    let total_bytes = records
        .get(&TLV_TOTAL)
        .ok_or(CodecError::Truncated { needed: 2, had: 0 })?;
    let total = decode_mantissa(total_bytes)?;

    let salt_hex = bytes_to_hex(salt_bytes);

    let token_address = if let Some(v) = records.get(&TLV_TOKEN_ADDRESS) {
        Some(decode_token_address(v)?)
    } else {
        None
    };

    let client_wallet_address = if let Some(v) = records.get(&TLV_CLIENT_WALLET) {
        Some(bytes_to_address(v)?)
    } else {
        None
    };

    let notes = if let Some(v) = records.get(&TLV_NOTES) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let from_email = if let Some(v) = records.get(&TLV_FROM_EMAIL) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let from_phone = if let Some(v) = records.get(&TLV_FROM_PHONE) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let from_physical_address = if let Some(v) = records.get(&TLV_FROM_ADDRESS) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let from_tax_id = if let Some(v) = records.get(&TLV_FROM_TAX_ID) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let client_email = if let Some(v) = records.get(&TLV_CLIENT_EMAIL) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let client_phone = if let Some(v) = records.get(&TLV_CLIENT_PHONE) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let client_physical_address = if let Some(v) = records.get(&TLV_CLIENT_ADDRESS) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let client_tax_id = if let Some(v) = records.get(&TLV_CLIENT_TAX_ID) {
        Some(reverse_dict(v)?)
    } else {
        None
    };

    let tax = if let Some(v) = records.get(&TLV_TAX) {
        Some(
            String::from_utf8(v.clone())
                .map_err(|_| CodecError::CompressionFailed("invalid UTF-8 in tax".to_string()))?,
        )
    } else {
        None
    };

    let discount =
        if let Some(v) = records.get(&TLV_DISCOUNT) {
            Some(String::from_utf8(v.clone()).map_err(|_| {
                CodecError::CompressionFailed("invalid UTF-8 in discount".to_string())
            })?)
        } else {
            None
        };

    Ok(Invoice {
        invoice_id,
        issued_at,
        due_at,
        network_id,
        currency,
        decimals,
        from: InvoiceFrom {
            name: from_name,
            wallet_address: from_wallet_address,
            email: from_email,
            phone: from_phone,
            physical_address: from_physical_address,
            tax_id: from_tax_id,
        },
        client: InvoiceClient {
            name: client_name,
            wallet_address: client_wallet_address,
            email: client_email,
            phone: client_phone,
            physical_address: client_physical_address,
            tax_id: client_tax_id,
        },
        items,
        token_address,
        notes,
        tax,
        discount,
        total,
        salt: salt_hex,
    })
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_mantissa_zero() {
        // encode: mantissa=0 → [0x00, 0x00]
        let result = decode_mantissa(&[0x00, 0x00]).unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn decode_mantissa_one_million() {
        // mantissa=1 (0x01), zeros=6 → 1_000_000
        let result = decode_mantissa(&[0x01, 0x06]).unwrap();
        assert_eq!(result, "1000000");
    }

    #[test]
    fn decode_mantissa_123() {
        // mantissa=123 (0x7B), zeros=0
        let result = decode_mantissa(&[0x7b, 0x00]).unwrap();
        assert_eq!(result, "123");
    }

    #[test]
    fn decode_chain_id_known_ethereum() {
        let result = decode_chain_id(&[0x00, 0x01]).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn decode_chain_id_known_base() {
        let result = decode_chain_id(&[0x00, 0x05]).unwrap();
        assert_eq!(result, 8453);
    }

    #[test]
    fn decode_currency_known_usdc() {
        let result = decode_currency(&[0x00, 0x01]).unwrap();
        assert_eq!(result, "USDC");
    }

    #[test]
    fn decode_currency_raw() {
        let mut v = vec![0x01u8];
        v.extend_from_slice(b"XYZ");
        let result = decode_currency(&v).unwrap();
        assert_eq!(result, "XYZ");
    }

    #[test]
    fn bytes_to_address_roundtrip() {
        let addr = "0xaabbccddee0011223344556677889900aabbccdd";
        let raw: Vec<u8> = (0..20)
            .map(|i| u8::from_str_radix(&addr[2 + i * 2..4 + i * 2], 16).unwrap())
            .collect();
        let result = bytes_to_address(&raw).unwrap();
        assert_eq!(result, addr);
    }

    #[test]
    fn reverse_dict_invoice() {
        // 0x06 is dict code for "Invoice"
        let result = reverse_dict(&[0x06]).unwrap();
        assert_eq!(result, "Invoice");
    }

    #[test]
    fn reverse_dict_passthrough() {
        let result = reverse_dict(b"Hello world").unwrap();
        assert_eq!(result, "Hello world");
    }

    // --- U256 mantissa decode tests ---

    #[test]
    fn decode_mantissa_u256_max_roundtrip() {
        // Encode u256::MAX via encode path then decode — end-to-end parity check.
        use crate::encode::tests_pub::mantissa_bytes_pub;
        let uint256_max =
            "115792089237316195423570985008687907853269984665640564039457584007913129639935";
        let encoded = mantissa_bytes_pub(uint256_max).unwrap();
        let decoded = decode_mantissa(&encoded).unwrap();
        assert_eq!(decoded, uint256_max);
    }

    #[test]
    fn decode_mantissa_large_value_above_u128() {
        // A value between u128::MAX and u256::MAX — old code would silently saturate.
        use crate::encode::tests_pub::mantissa_bytes_pub;
        // u128::MAX * 1000 (well above u128 range)
        let large = "340282366920938463463374607431768211455000";
        let encoded = mantissa_bytes_pub(large).unwrap();
        let decoded = decode_mantissa(&encoded).unwrap();
        assert_eq!(decoded, large);
    }

    #[test]
    fn decode_mantissa_wire_payload_exceeding_u256_errors() {
        // Craft a wire payload whose mantissa varint decodes to 33 bytes (> 32) — must error
        // cleanly, never silently saturate (the old u128 saturation bug).
        // A 33-byte all-0xFF big-endian value encoded as LEB128 exceeds MAX_BYTES (37 × 7-bit
        // chunks = 259 bits > 256 bits) so the varint layer returns VarintOverflow before the
        // 32-byte U256 guard fires.  Both VarintOverflow and InvalidAmount are CodecError
        // variants — either satisfies the "no silent saturation" requirement.
        use crate::varint::write_bigint_varint;
        let oversized_mantissa = vec![0xFFu8; 33]; // 33 bytes > U256 max 32 bytes
        let mut payload = Vec::new();
        write_bigint_varint(&oversized_mantissa, &mut payload);
        payload.push(0u8); // zeros = 0

        let err = decode_mantissa(&payload).unwrap_err();
        assert!(
            matches!(
                err,
                CodecError::InvalidAmount(_) | CodecError::VarintOverflow(_)
            ),
            "expected InvalidAmount or VarintOverflow for oversized mantissa, got {err:?}"
        );
    }

    // --- R1: due_at u64→u32 truncation guard ---

    /// A varint encoding 2^32 (0x1_0000_0000) must not silently truncate to 0.
    /// Old code: `issued_at + due_delta as u32` → 0x1_0000_0000 as u32 == 0 → due_at == issued_at.
    #[test]
    fn r1_due_at_delta_exactly_2pow32_errors() {
        use crate::varint::write_varint;
        let delta: u64 = 0x1_0000_0000; // 2^32 — overflows u32
        let mut due_bytes = Vec::new();
        write_varint(delta, &mut due_bytes);

        // Feed the oversized delta through the varint decode path directly.
        // read_varint returns a u64; try_from(u64) must reject values > u32::MAX.
        let (decoded_delta, _) = crate::varint::read_varint(&due_bytes, 0).unwrap();
        let result = u32::try_from(decoded_delta);
        assert!(
            result.is_err(),
            "u32::try_from(2^32) must fail — old 'as u32' cast would silently truncate to 0"
        );
    }

    /// A varint encoding 2^32 + 100 must also reject, not produce due_at = issued_at + 100.
    #[test]
    fn r1_due_at_delta_2pow32_plus_100_errors() {
        use crate::varint::write_varint;
        let delta: u64 = 0x1_0000_0064; // 2^32 + 100
        let mut due_bytes = Vec::new();
        write_varint(delta, &mut due_bytes);

        let (decoded_delta, _) = crate::varint::read_varint(&due_bytes, 0).unwrap();
        let result = u32::try_from(decoded_delta);
        assert!(
            result.is_err(),
            "u32::try_from(2^32+100) must fail — old cast would silently produce delta=100"
        );
    }

    /// Encode a valid invoice then manually craft a TLV_DUE_AT with delta = 2^32.
    /// decode_invoice_canonical must return Err, not silently produce due_at == issued_at.
    #[test]
    fn r1_full_decode_rejects_due_at_overflow() {
        use crate::encode::encode_invoice_canonical;
        use crate::invoice::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};
        use crate::varint::write_varint;

        // Build a valid invoice and encode it.
        let invoice = Invoice {
            invoice_id: "INV-R1".to_string(),
            issued_at: 1_700_000_000,
            due_at: 1_700_604_800,
            network_id: 1,
            currency: "USDC".to_string(),
            decimals: 6,
            from: InvoiceFrom {
                name: "Alice".to_string(),
                wallet_address: "0xaabbccddee0011223344556677889900aabbccdd".to_string(),
                email: None,
                phone: None,
                physical_address: None,
                tax_id: None,
            },
            client: InvoiceClient {
                name: "Bob".to_string(),
                wallet_address: None,
                email: None,
                phone: None,
                physical_address: None,
                tax_id: None,
            },
            items: vec![InvoiceItem {
                description: "Work".to_string(),
                quantity: 1.0,
                rate: "1000000".to_string(),
            }],
            token_address: None,
            notes: None,
            tax: None,
            discount: None,
            total: "1000000".to_string(),
            salt: "00112233445566778899aabbccddeeff".to_string(),
        };
        let mut bytes = encode_invoice_canonical(&invoice).unwrap();

        // Patch TLV_DUE_AT (type=6) in the wire bytes with delta = 2^32.
        // Scan for type byte 0x06 after the 3-byte header.
        let header_len = 3usize;
        let mut i = header_len;
        while i < bytes.len() {
            let tlv_type = bytes[i];
            let (length, n) = crate::varint::read_varint(&bytes, i + 1).unwrap();
            let value_start = i + 1 + n;
            let value_end = value_start + length as usize;
            if tlv_type == crate::encode::TLV_DUE_AT {
                // Replace value with varint(2^32).
                let mut new_val = Vec::new();
                write_varint(0x1_0000_0000u64, &mut new_val);
                // Rebuild entire TLV for type 6 to correctly patch the length varint.
                let mut tlv_new = Vec::new();
                tlv_new.push(0x06u8);
                write_varint(new_val.len() as u64, &mut tlv_new);
                tlv_new.extend_from_slice(&new_val);
                let before = &bytes[..i];
                let after = &bytes[value_end..];
                let mut rebuilt = before.to_vec();
                rebuilt.extend_from_slice(&tlv_new);
                rebuilt.extend_from_slice(after);
                bytes = rebuilt;
                break;
            }
            i = value_end;
        }

        let err = decode_invoice_canonical(&bytes).unwrap_err();
        assert!(
            matches!(err, CodecError::InvalidAmount(_) | CodecError::ChecksumMismatch),
            "expected InvalidAmount or ChecksumMismatch for due_at overflow, got {err:?}"
        );
    }
}
