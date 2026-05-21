// Per-field TLV writers: string fields, packed items, domain separator.

use std::collections::BTreeMap;

use crate::error::CodecError;
use crate::hash::keccak256;
use crate::varint::write_varint;

use super::amount::{mantissa_bytes, write_quantity};
use super::dict::apply_dict;
use super::tags::{MAX_ITEMS, TLV_DOMAIN_SEPARATOR};

/// Encode a UTF-8 string to bytes.
pub(super) fn utf8_bytes(s: &str) -> Vec<u8> {
    s.as_bytes().to_vec()
}

/// Encode items array into packed binary (Type 14, mirrors packItems from encode.ts).
/// Format: [count: varint] per item: [desc_len: varint][desc_bytes][qty: scale+varint][rate: mantissa]
pub(super) fn pack_items(items: &[crate::invoice::InvoiceItem]) -> Result<Vec<u8>, CodecError> {
    if items.len() > MAX_ITEMS {
        return Err(CodecError::CompressionFailed(format!(
            "item count {} exceeds max {MAX_ITEMS}",
            items.len()
        )));
    }
    let mut buf = Vec::new();
    write_varint(items.len() as u64, &mut buf);

    for item in items {
        // description: apply dict, then length-prefix with varint
        let desc_bytes = apply_dict(&item.description)?;
        write_varint(desc_bytes.len() as u64, &mut buf);
        buf.extend_from_slice(&desc_bytes);

        // quantity: [scale: u8][scaled_value: varint] — mirrors writeQuantity
        write_quantity(&mut buf, item.quantity)?;

        // rate: mantissa + trailing zeros — mirrors writeMantissa
        let rate_bytes = mantissa_bytes(&item.rate)?;
        buf.extend_from_slice(&rate_bytes);
    }
    Ok(buf)
}

/// Compute domain separator: keccak256("VOIDPAY_INVOICE_V1" || serialized TLV records except type 31).
/// Mirrors computeDomainSeparator from security.ts.
pub(super) fn compute_domain_separator(records: &BTreeMap<u8, Vec<u8>>) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice::InvoiceItem;

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

    // --- R4: MAX_ITEMS encode cap ---

    /// pack_items must reject item counts above MAX_ITEMS (50) with an error,
    /// not produce a blob that decode_invoice_canonical would reject later.
    #[test]
    fn r4_pack_items_above_max_items_errors() {
        let item = crate::invoice::InvoiceItem {
            description: "Work".to_string(),
            quantity: 1.0,
            rate: "1000000".to_string(),
        };
        // MAX_ITEMS = 50; create 51 items.
        let items: Vec<_> = (0..51).map(|_| item.clone()).collect();
        let err = pack_items(&items).unwrap_err();
        assert!(
            matches!(err, crate::error::CodecError::CompressionFailed(_)),
            "expected CompressionFailed for 51 items > MAX_ITEMS, got {err:?}"
        );
    }

    /// Exactly MAX_ITEMS (50) items must still encode without error.
    #[test]
    fn r4_pack_items_at_max_items_ok() {
        let item = crate::invoice::InvoiceItem {
            description: "Work".to_string(),
            quantity: 1.0,
            rate: "1000000".to_string(),
        };
        let items: Vec<_> = (0..50).map(|_| item.clone()).collect();
        assert!(
            pack_items(&items).is_ok(),
            "exactly MAX_ITEMS items must encode successfully"
        );
    }
}
