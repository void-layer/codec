use void_layer_codec::CodecError;

#[test]
fn varint_overflow_displays_with_offset() {
    let err = CodecError::VarintOverflow(42);
    assert_eq!(err.to_string(), "varint overflow at offset 42");
}

#[test]
fn truncated_displays_needed_and_had() {
    let err = CodecError::Truncated { needed: 10, had: 3 };
    assert_eq!(err.to_string(), "truncated payload: needed 10 bytes, had 3");
}

#[test]
fn unknown_extension_displays_type() {
    let err = CodecError::UnknownExtension(0xAB);
    assert_eq!(err.to_string(), "unknown extension TLV type 171");
}

#[test]
fn dictionary_mismatch_displays_expected_and_actual() {
    let err = CodecError::DictionaryMismatch {
        expected: 1,
        actual: 2,
    };
    assert_eq!(err.to_string(), "dictionary mismatch: expected 1, actual 2");
}

#[test]
fn signature_invalid_displays() {
    let err = CodecError::SignatureInvalid;
    assert_eq!(err.to_string(), "signature invalid");
}

#[test]
fn unsupported_version_displays() {
    let err = CodecError::UnsupportedVersion(7);
    assert_eq!(err.to_string(), "unsupported version 7");
}

#[test]
fn bad_magic_displays() {
    let err = CodecError::BadMagic;
    assert_eq!(err.to_string(), "bad magic bytes");
}

#[test]
fn checksum_mismatch_displays() {
    let err = CodecError::ChecksumMismatch;
    assert_eq!(err.to_string(), "checksum mismatch");
}

#[test]
fn compression_failed_displays_inner_message() {
    let err = CodecError::CompressionFailed("buffer full".to_string());
    assert_eq!(err.to_string(), "compression failed: buffer full");
}

#[test]
fn invalid_amount_displays_inner_message() {
    let err = CodecError::InvalidAmount("not_a_number".to_string());
    assert_eq!(err.to_string(), "invalid amount: not_a_number");
}

#[test]
fn invalid_address_displays_inner_message() {
    let err = CodecError::InvalidAddress("bad hex".to_string());
    assert_eq!(err.to_string(), "invalid address: bad hex");
}

#[test]
fn missing_field_displays_tlv_type() {
    let err = CodecError::MissingField(2);
    assert_eq!(err.to_string(), "missing required TLV field 2");
}

#[test]
fn overflow_displays_inner_message() {
    let err = CodecError::Overflow("TLV count 65 exceeds max 64".to_string());
    assert_eq!(
        err.to_string(),
        "payload overflow: TLV count 65 exceeds max 64"
    );
}
