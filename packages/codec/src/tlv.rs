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
/// Returns a `BTreeMap<type, value>`. Duplicate types are last-write-wins
/// (matches TS reader behaviour — the stream is trusted to be canonical).
///
/// Errors: propagated from `read_tlv`.
pub(crate) fn read_tlv_stream(buf: &[u8]) -> Result<BTreeMap<u8, Vec<u8>>, CodecError> {
    let mut map = BTreeMap::new();
    let mut offset = 0;
    while offset < buf.len() {
        let (record, consumed) = read_tlv(buf, offset)?;
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

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- read_tlv / write_tlv single-record roundtrip ----------------------

    #[test]
    fn single_record_roundtrip() {
        let record = TlvRecord {
            tlv_type: 0x01,
            value: vec![0xAA, 0xBB, 0xCC],
        };
        let mut buf = Vec::new();
        write_tlv(&record, &mut buf);

        // Wire: [0x01, 0x03, 0xAA, 0xBB, 0xCC]
        assert_eq!(buf, vec![0x01, 0x03, 0xAA, 0xBB, 0xCC]);

        let (decoded, consumed) = read_tlv(&buf, 0).unwrap();
        assert_eq!(decoded.tlv_type, 0x01);
        assert_eq!(decoded.value, vec![0xAA, 0xBB, 0xCC]);
        assert_eq!(consumed, 5);
    }

    #[test]
    fn empty_value_record_roundtrip() {
        let record = TlvRecord {
            tlv_type: 0xFF,
            value: vec![],
        };
        let mut buf = Vec::new();
        write_tlv(&record, &mut buf);

        // Wire: [0xFF, 0x00]
        assert_eq!(buf, vec![0xFF, 0x00]);

        let (decoded, consumed) = read_tlv(&buf, 0).unwrap();
        assert_eq!(decoded.tlv_type, 0xFF);
        assert_eq!(decoded.value, vec![]);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn large_value_uses_multi_byte_varint_length() {
        // 128-byte value → length encoded as two LEB128 bytes [0x80, 0x01]
        let value = vec![0u8; 128];
        let record = TlvRecord {
            tlv_type: 0x02,
            value: value.clone(),
        };
        let mut buf = Vec::new();
        write_tlv(&record, &mut buf);

        // TYPE(1) + LENGTH(2) + VALUE(128) = 131 bytes
        assert_eq!(buf.len(), 131);
        assert_eq!(buf[0], 0x02);
        assert_eq!(&buf[1..3], &[0x80, 0x01]); // LEB128(128) = [0x80, 0x01]

        let (decoded, consumed) = read_tlv(&buf, 0).unwrap();
        assert_eq!(decoded.tlv_type, 0x02);
        assert_eq!(decoded.value, value);
        assert_eq!(consumed, 131);
    }

    // --- read_tlv_stream / write_tlv_stream multi-record roundtrip ----------

    #[test]
    fn stream_roundtrip_multi_record() {
        let mut stream = BTreeMap::new();
        stream.insert(0x01u8, vec![0x11u8, 0x22]);
        stream.insert(0x02u8, vec![0x33u8]);
        stream.insert(0x05u8, vec![0xAAu8, 0xBBu8, 0xCC]);

        let mut buf = Vec::new();
        write_tlv_stream(&stream, &mut buf);

        let decoded = read_tlv_stream(&buf).unwrap();
        assert_eq!(decoded, stream);
    }

    #[test]
    fn stream_empty_map_produces_empty_bytes() {
        let stream: BTreeMap<u8, Vec<u8>> = BTreeMap::new();
        let mut buf = Vec::new();
        write_tlv_stream(&stream, &mut buf);
        assert!(buf.is_empty());

        let decoded = read_tlv_stream(&buf).unwrap();
        assert!(decoded.is_empty());
    }

    // --- byte-stability invariant ------------------------------------------

    #[test]
    fn write_tlv_stream_is_byte_stable_across_two_runs() {
        let mut stream = BTreeMap::new();
        stream.insert(0x03u8, vec![0x01u8, 0x02, 0x03]);
        stream.insert(0x01u8, vec![0xFFu8]);
        stream.insert(0x02u8, vec![0x00u8, 0x00]);

        let mut buf1 = Vec::new();
        write_tlv_stream(&stream, &mut buf1);

        let mut buf2 = Vec::new();
        write_tlv_stream(&stream, &mut buf2);

        assert_eq!(buf1, buf2, "write_tlv_stream must be byte-stable");
    }

    #[test]
    fn write_tlv_stream_key_order_is_ascending() {
        // Insert in reverse order; BTreeMap must emit in key-ascending order.
        let mut stream = BTreeMap::new();
        stream.insert(0x05u8, vec![0x55u8]);
        stream.insert(0x01u8, vec![0x11u8]);
        stream.insert(0x03u8, vec![0x33u8]);

        let mut buf = Vec::new();
        write_tlv_stream(&stream, &mut buf);

        // First type byte in wire output must be 0x01 (lowest key).
        assert_eq!(buf[0], 0x01, "first emitted type should be the lowest key");
    }

    // --- Truncated errors ---------------------------------------------------

    #[test]
    fn truncated_on_empty_buffer() {
        let err = read_tlv(&[], 0).unwrap_err();
        assert!(
            matches!(err, CodecError::Truncated { .. }),
            "expected Truncated, got {err:?}"
        );
    }

    #[test]
    fn truncated_when_value_bytes_missing() {
        // TYPE=0x01, LENGTH=0x03 (3 bytes), but only 1 value byte present.
        let buf = &[0x01u8, 0x03, 0xAA];
        let err = read_tlv(buf, 0).unwrap_err();
        assert!(
            matches!(err, CodecError::Truncated { needed: 5, had: 3 }),
            "expected Truncated{{needed:5, had:3}}, got {err:?}"
        );
    }

    #[test]
    fn truncated_when_type_byte_at_offset_beyond_buf() {
        let buf = &[0x01u8, 0x01, 0xAAu8]; // valid single record, 3 bytes
        let err = read_tlv(buf, 3).unwrap_err(); // offset == buf.len()
        assert!(
            matches!(err, CodecError::Truncated { .. }),
            "expected Truncated, got {err:?}"
        );
    }

    #[test]
    fn truncated_mid_stream_surfaces_error() {
        // Write a valid two-record stream, then truncate the second record's value.
        let mut good_buf = Vec::new();
        write_tlv(
            &TlvRecord {
                tlv_type: 0x01,
                value: vec![0x01],
            },
            &mut good_buf,
        );
        write_tlv(
            &TlvRecord {
                tlv_type: 0x02,
                value: vec![0xAA, 0xBB, 0xCC],
            },
            &mut good_buf,
        );

        // Truncate: drop the last byte of the second record's value.
        let truncated = &good_buf[..good_buf.len() - 1];
        let err = read_tlv_stream(truncated).unwrap_err();
        assert!(
            matches!(err, CodecError::Truncated { .. }),
            "expected Truncated from stream, got {err:?}"
        );
    }

    // --- R2: u64→usize TLV length truncation guard ---

    /// A TLV length prefix of 0x1_0000_0064 (> 4096 MAX_VALUE_SIZE) must be
    /// rejected before the u64→usize cast. On wasm32, the cast would truncate
    /// 0x1_0000_0064 → 100, then read 100 bytes of garbage — silent misalignment.
    #[test]
    fn r2_oversized_tlv_length_prefix_errors() {
        use crate::varint::write_varint;

        // Craft a TLV record: type=0x01, length=0x1_0000_0064 (4GiB+100 — way above MAX_VALUE_SIZE)
        let mut buf = Vec::new();
        buf.push(0x01u8); // type
        write_varint(0x1_0000_0064u64, &mut buf); // length varint > u32::MAX, > MAX_VALUE_SIZE

        // No value bytes follow — the guard must fire before attempting to read them.
        let err = read_tlv(&buf, 0).unwrap_err();
        assert!(
            matches!(err, CodecError::Truncated { .. }),
            "expected Truncated for oversized length prefix, got {err:?}"
        );
    }

    /// A TLV length just above MAX_VALUE_SIZE (4097) must also be rejected.
    #[test]
    fn r2_tlv_length_just_above_max_value_size_errors() {
        use crate::varint::write_varint;

        let mut buf = Vec::new();
        buf.push(0x02u8); // type
        write_varint(4097u64, &mut buf); // MAX_VALUE_SIZE=4096, so 4097 must error

        let err = read_tlv(&buf, 0).unwrap_err();
        assert!(
            matches!(err, CodecError::Truncated { .. }),
            "expected Truncated for length 4097 > MAX_VALUE_SIZE, got {err:?}"
        );
    }
}
