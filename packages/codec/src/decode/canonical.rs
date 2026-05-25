// Domain-separator verification over the canonical TLV record map.

use std::collections::BTreeMap;

use crate::error::CodecError;

/// Verify domain separator (mirrors validateSecurity from security.ts).
/// Delegates to crate::canonical::compute_domain_separator — single source of truth.
pub(super) fn verify_domain_separator(
    records: &BTreeMap<u8, Vec<u8>>,
    stored_sep: &[u8],
) -> Result<(), CodecError> {
    let computed = crate::canonical::compute_domain_separator(records);
    if computed.as_slice() != stored_sep {
        return Err(CodecError::ChecksumMismatch);
    }
    Ok(())
}
