//! Domain-separator computation — payment-identity contract.
//! Used by encode (compute) and decode (verify). Single source of truth.
//! See spec §security / computeDomainSeparator in security.ts.
//!
//! If the two implementations ever drift, every payload silently fails
//! ChecksumMismatch. Co-locating them here makes that impossible.

use std::collections::BTreeMap;

use crate::encode::TLV_DOMAIN_SEPARATOR;
use crate::hash::keccak256;
use crate::varint::write_varint;

/// Domain separator prefix — wire constant, must not change after v1.0.
pub(crate) const DOMAIN_SEPARATOR_PREFIX: &[u8; 18] = b"VOIDPAY_INVOICE_V1";

/// Compute domain separator: keccak256(PREFIX || serialized TLV records except type 31).
/// Mirrors computeDomainSeparator from security.ts.
pub(crate) fn compute_domain_separator(records: &BTreeMap<u8, Vec<u8>>) -> [u8; 32] {
    let mut body: Vec<u8> = DOMAIN_SEPARATOR_PREFIX.to_vec();

    // Serialize each record except domain separator (type 31) in key-ascending order.
    // type(1) + length(varint) + value — mirrors TLV wire format.
    for (&tlv_type, value) in records {
        if tlv_type == TLV_DOMAIN_SEPARATOR {
            continue;
        }
        body.push(tlv_type);
        write_varint(value.len() as u64, &mut body);
        body.extend_from_slice(value);
    }

    keccak256(&body)
}
