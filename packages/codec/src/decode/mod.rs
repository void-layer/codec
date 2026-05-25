// Mirrors vl/app/src/features/invoice-codec/lib/decode.ts
// and vl/app/src/shared/lib/tlv-codec/{reader.ts,varint.ts}.
//
// Reads: [MAGIC][VERSION][COUNT][TLV records...]
// Validates: magic, version (no COMPRESSED_FLAG), canonical ordering, domain separator.
// Maps TLV types to Invoice fields per tlv-map.ts.

mod amount;
mod canonical;
mod dict;
mod hex;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::encode::{
    COMPRESSED_FLAG, KNOWN_TAGS, MAGIC, TLV_CHAIN_ID, TLV_CLIENT_ADDRESS, TLV_CLIENT_EMAIL,
    TLV_CLIENT_NAME, TLV_CLIENT_PHONE, TLV_CLIENT_TAX_ID, TLV_CLIENT_WALLET, TLV_CURRENCY,
    TLV_DECIMALS, TLV_DISCOUNT, TLV_DOMAIN_SEPARATOR, TLV_DUE_AT, TLV_FROM_ADDRESS, TLV_FROM_EMAIL,
    TLV_FROM_NAME, TLV_FROM_PHONE, TLV_FROM_TAX_ID, TLV_FROM_WALLET, TLV_INVOICE_ID, TLV_ISSUED_AT,
    TLV_ITEMS, TLV_NOTES, TLV_SALT, TLV_TAX, TLV_TOKEN_ADDRESS, TLV_TOTAL, VERSION,
};
use crate::error::CodecError;
use crate::invoice::{Invoice, InvoiceClient, InvoiceFrom};
use crate::limits::{MAX_TLV_COUNT, MAX_VALUE_SIZE};
use crate::tlv::read_tlv_stream;
use crate::varint::read_varint;

use amount::{decode_mantissa, unpack_items};
use canonical::verify_domain_separator;
use dict::{decode_chain_id, decode_currency, decode_token_address, reverse_dict};
use hex::{bytes_to_address, bytes_to_hex};

// ---------------------------------------------------------------------------
// Test helpers (pub only under #[cfg(test)])
// ---------------------------------------------------------------------------

// Re-exported so `crate::decode::tests_pub::decode_mantissa_pub` keeps resolving
// for `encode.rs` after the decode/ submodule split.
#[cfg(test)]
pub(crate) use amount::tests_pub;

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
        return Err(CodecError::InvalidData(
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
        return Err(CodecError::Overflow(format!(
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
            return Err(CodecError::Overflow(format!(
                "TLV type {tlv_type} value size {} exceeds max {MAX_VALUE_SIZE}",
                value.len()
            )));
        }
    }

    // C-2: reject any tag outside the known v1 set before checksum validation.
    // An unknown tag means unread bytes are part of the accepted payload, which
    // creates semantic divergence between readers — different content_hash values.
    for &tag in records.keys() {
        if !KNOWN_TAGS.contains(&tag) {
            return Err(CodecError::UnknownExtension(tag));
        }
    }

    let salt_bytes = records.get(&TLV_SALT).ok_or(CodecError::ChecksumMismatch)?;
    if salt_bytes.len() != 16 {
        return Err(CodecError::ChecksumMismatch);
    }

    let stored_sep = records
        .get(&TLV_DOMAIN_SEPARATOR)
        .ok_or(CodecError::ChecksumMismatch)?;
    verify_domain_separator(&records, stored_sep)?;

    let chain_id_bytes = records
        .get(&TLV_CHAIN_ID)
        .ok_or(CodecError::MissingField(TLV_CHAIN_ID))?;
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
    let due_delta_u32 = u32::try_from(due_delta).map_err(|_| {
        CodecError::InvalidAmount(format!("due_at delta {due_delta} overflows u32"))
    })?;
    let due_at = issued_at.checked_add(due_delta_u32).ok_or_else(|| {
        CodecError::InvalidAmount(format!(
            "due_at overflow: issued_at {issued_at} + delta {due_delta_u32}"
        ))
    })?;

    let decimals_bytes = records
        .get(&TLV_DECIMALS)
        .ok_or(CodecError::Truncated { needed: 1, had: 0 })?;
    // Canonical encoder always emits exactly 1 byte for TLV_DECIMALS.
    // len > 1 silently truncated via .first() before this fix — reject instead.
    if decimals_bytes.len() != 1 {
        return Err(CodecError::InvalidData(format!(
            "non-canonical TLV_DECIMALS length: expected 1, got {}",
            decimals_bytes.len()
        )));
    }
    let decimals = decimals_bytes[0];

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
        .map_err(|_| CodecError::InvalidData("invalid UTF-8 in invoice_id".to_string()))?;

    let total_bytes = records
        .get(&TLV_TOTAL)
        .ok_or(CodecError::Truncated { needed: 2, had: 0 })?;
    let total = decode_mantissa(total_bytes)?;

    let salt_hex = bytes_to_hex(salt_bytes);

    let token_address = records
        .get(&TLV_TOKEN_ADDRESS)
        .map(|v| decode_token_address(v))
        .transpose()?;

    let client_wallet_address = records
        .get(&TLV_CLIENT_WALLET)
        .map(|v| bytes_to_address(v))
        .transpose()?;

    let notes = records
        .get(&TLV_NOTES)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let from_email = records
        .get(&TLV_FROM_EMAIL)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let from_phone = records
        .get(&TLV_FROM_PHONE)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let from_physical_address = records
        .get(&TLV_FROM_ADDRESS)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let from_tax_id = records
        .get(&TLV_FROM_TAX_ID)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let client_email = records
        .get(&TLV_CLIENT_EMAIL)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let client_phone = records
        .get(&TLV_CLIENT_PHONE)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let client_physical_address = records
        .get(&TLV_CLIENT_ADDRESS)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let client_tax_id = records
        .get(&TLV_CLIENT_TAX_ID)
        .map(|v| reverse_dict(v))
        .transpose()?;

    let decode_utf8 = |v: &Vec<u8>, field: &'static str| -> Result<String, CodecError> {
        String::from_utf8(v.clone())
            .map_err(|_| CodecError::InvalidData(format!("invalid UTF-8 in {field}")))
    };

    let tax = records
        .get(&TLV_TAX)
        .map(|v| decode_utf8(v, "tax"))
        .transpose()?;

    let discount = records
        .get(&TLV_DISCOUNT)
        .map(|v| decode_utf8(v, "discount"))
        .transpose()?;

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
