// Mirrors vl/app/src/features/invoice-codec/lib/encode.ts
// and vl/app/src/shared/lib/tlv-codec/{writer.ts,varint.ts}.
//
// TLV type registry constants mirror tlv-map.ts TlvType enum.
// Encoding order: sort by TLV type ascending (BTreeMap), then append domain separator last.

use std::collections::BTreeMap;

use crate::error::CodecError;
use crate::tlv::write_tlv_stream;

mod address;
mod amount;
mod dict;
mod fields;
mod tags;

use address::{address_to_bytes, encode_token_address, hex_decode_salt};
use amount::{mantissa_bytes, uint32_be, varint_bytes};
use dict::{apply_dict, encode_chain_id, encode_currency};
use fields::{compute_domain_separator, pack_items, utf8_bytes};
// `MAX_*` limits stay module-internal (originally unmarked in encode.rs → `pub(super)`).
use tags::{MAX_PAYLOAD_SIZE, MAX_TLV_COUNT, MAX_VALUE_SIZE};

// Re-export the wire-format + TLV-tag constants at their real names so
// `crate::encode::TLV_DUE_AT`, `crate::encode::MAGIC`, etc. continue to resolve
// for `decode.rs`. These are `pub(crate)` in `tags` — visibility unchanged.
pub(crate) use tags::{
    COMPRESSED_FLAG, MAGIC, TLV_CHAIN_ID, TLV_CLIENT_ADDRESS, TLV_CLIENT_EMAIL, TLV_CLIENT_NAME,
    TLV_CLIENT_PHONE, TLV_CLIENT_TAX_ID, TLV_CLIENT_WALLET, TLV_CURRENCY, TLV_DECIMALS,
    TLV_DISCOUNT, TLV_DOMAIN_SEPARATOR, TLV_DUE_AT, TLV_FROM_ADDRESS, TLV_FROM_EMAIL,
    TLV_FROM_NAME, TLV_FROM_PHONE, TLV_FROM_TAX_ID, TLV_FROM_WALLET, TLV_INVOICE_ID, TLV_ISSUED_AT,
    TLV_ITEMS, TLV_NOTES, TLV_SALT, TLV_TAX, TLV_TOKEN_ADDRESS, TLV_TOTAL, VERSION,
};

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
    map.insert(TLV_FROM_NAME, apply_dict(&invoice.from.name)?);

    // Client name (type 18): dict-applied UTF-8
    map.insert(TLV_CLIENT_NAME, apply_dict(&invoice.client.name)?);

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
        map.insert(TLV_NOTES, apply_dict(notes)?);
    }

    if let Some(ref email) = invoice.from.email {
        map.insert(TLV_FROM_EMAIL, apply_dict(email)?);
    }

    if let Some(ref phone) = invoice.from.phone {
        map.insert(TLV_FROM_PHONE, apply_dict(phone)?);
    }

    if let Some(ref addr) = invoice.from.physical_address {
        map.insert(TLV_FROM_ADDRESS, apply_dict(addr)?);
    }

    if let Some(ref tax_id) = invoice.from.tax_id {
        map.insert(TLV_FROM_TAX_ID, apply_dict(tax_id)?);
    }

    if let Some(ref email) = invoice.client.email {
        map.insert(TLV_CLIENT_EMAIL, apply_dict(email)?);
    }

    if let Some(ref phone) = invoice.client.phone {
        map.insert(TLV_CLIENT_PHONE, apply_dict(phone)?);
    }

    if let Some(ref addr) = invoice.client.physical_address {
        map.insert(TLV_CLIENT_ADDRESS, apply_dict(addr)?);
    }

    if let Some(ref tax_id) = invoice.client.tax_id {
        map.insert(TLV_CLIENT_TAX_ID, apply_dict(tax_id)?);
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
// Test helpers (pub only under #[cfg(test)])
// ---------------------------------------------------------------------------

#[cfg(test)]
pub(crate) mod tests_pub {
    pub(crate) fn mantissa_bytes_pub(s: &str) -> Result<Vec<u8>, crate::error::CodecError> {
        super::mantissa_bytes(s)
    }
}
