//! Tests for tlv.
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
