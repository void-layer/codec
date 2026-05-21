// Domain-separator verification over the canonical TLV record map.

use std::collections::BTreeMap;

use crate::encode::TLV_DOMAIN_SEPARATOR;
use crate::error::CodecError;
use crate::hash::keccak256;

/// Verify domain separator (mirrors validateSecurity from security.ts).
pub(super) fn verify_domain_separator(
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
