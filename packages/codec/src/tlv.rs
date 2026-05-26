use std::collections::BTreeMap;

use crate::error::CodecError;
use crate::varint::{read_varint, write_varint};

/// A single TLV (Type-Length-Value) record.
#[derive(Debug)]
pub(crate) struct TlvRecord {
    pub tlv_type: u8,
    pub value: Vec<u8>,
}

/// Reads one TLV record from `buf` starting at `offset`.
///
/// Returns `(record, bytes_consumed)`.
///
/// Wire format: `TYPE(1) | LENGTH(LEB128) | VALUE(length bytes)`.
///
/// Errors:
/// - `CodecError::Truncated` if the buffer ends before the type byte or mid-value.
pub(crate) fn read_tlv(buf: &[u8], offset: usize) -> Result<(TlvRecord, usize), CodecError> {
    if offset >= buf.len() {
        return Err(CodecError::Truncated {
            needed: offset + 1,
            had: buf.len(),
        });
    }
    let tlv_type = buf[offset];
    let mut consumed = 1usize;

    let (length, varint_bytes) = read_varint(buf, offset + consumed)?;
    consumed += varint_bytes;

    // Guard before cast: a length > MAX_VALUE_SIZE is invalid regardless of
    // target pointer width (prevents silent u64→usize truncation on wasm32).
    if length > crate::limits::MAX_VALUE_SIZE as u64 {
        return Err(CodecError::Truncated {
            needed: length as usize,
            had: buf.len(),
        });
    }
    let length = length as usize;
    let value_end = offset + consumed + length;
    if value_end > buf.len() {
        return Err(CodecError::Truncated {
            needed: value_end,
            had: buf.len(),
        });
    }
    let value = buf[offset + consumed..value_end].to_vec();
    consumed += length;

    Ok((TlvRecord { tlv_type, value }, consumed))
}

/// Serializes one TLV record into `out`.
///
/// Wire format: `TYPE(1) | LENGTH(LEB128) | VALUE`.
pub(crate) fn write_tlv(record: &TlvRecord, out: &mut Vec<u8>) {
    out.push(record.tlv_type);
    write_varint(record.value.len() as u64, out);
    out.extend_from_slice(&record.value);
}

/// Reads a flat sequence of TLV records from `buf` (the entire slice).
///
/// Returns a `BTreeMap<type, value>`. The wire stream MUST be strictly-monotone
/// (each tag strictly greater than the previous) per BOLT-01 and the void-layer
/// canonical TLV contract (decision: codec-bolt12-strict-monotone-decode).
/// Non-monotone or duplicate tags are rejected with
/// `CodecError::InvalidData("non-monotone TLV stream")` — two wire representations
/// of the same logical invoice must never be accepted.
///
/// Errors: propagated from `read_tlv`, or `InvalidData` on non-monotone / duplicate.
pub(crate) fn read_tlv_stream(buf: &[u8]) -> Result<BTreeMap<u8, Vec<u8>>, CodecError> {
    let mut map = BTreeMap::new();
    let mut offset = 0;
    let mut prev_type: Option<u8> = None;
    while offset < buf.len() {
        let (record, consumed) = read_tlv(buf, offset)?;
        if let Some(prev) = prev_type {
            if record.tlv_type <= prev {
                return Err(CodecError::InvalidData(
                    "non-monotone TLV stream".to_string(),
                ));
            }
        }
        prev_type = Some(record.tlv_type);
        // Duplicate check is now structurally unreachable under strict-monotone
        // (duplicate implies tlv_type == prev, caught above), but kept defensively.
        if map.contains_key(&record.tlv_type) {
            return Err(CodecError::InvalidData("duplicate TLV tag".to_string()));
        }
        map.insert(record.tlv_type, record.value);
        offset += consumed;
    }
    Ok(map)
}

/// Serializes a `BTreeMap` of TLV entries into `out` in key order.
///
/// `BTreeMap` guarantees ascending key iteration, so output is deterministic
/// (D-B4: byte-stable encoding requires deterministic field ordering).
pub(crate) fn write_tlv_stream(stream: &BTreeMap<u8, Vec<u8>>, out: &mut Vec<u8>) {
    for (&tlv_type, value) in stream {
        let record = TlvRecord {
            tlv_type,
            value: value.clone(),
        };
        write_tlv(&record, out);
    }
}

#[cfg(test)]
mod tests;
