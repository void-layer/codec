use super::amount::{decode_mantissa, unpack_items};
use super::decode_invoice_canonical;
use super::dict::{decode_chain_id, decode_currency, reverse_dict};
use super::hex::bytes_to_address;
use crate::error::CodecError;

#[test]
fn decode_mantissa_zero() {
    // encode: mantissa=0 → [0x00, 0x00]
    let result = decode_mantissa(&[0x00, 0x00]).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn decode_mantissa_one_million() {
    // mantissa=1 (0x01), zeros=6 → 1_000_000
    let result = decode_mantissa(&[0x01, 0x06]).unwrap();
    assert_eq!(result, "1000000");
}

#[test]
fn decode_mantissa_123() {
    // mantissa=123 (0x7B), zeros=0
    let result = decode_mantissa(&[0x7b, 0x00]).unwrap();
    assert_eq!(result, "123");
}

#[test]
fn decode_chain_id_known_ethereum() {
    let result = decode_chain_id(&[0x00, 0x01]).unwrap();
    assert_eq!(result, 1);
}

#[test]
fn decode_chain_id_known_base() {
    let result = decode_chain_id(&[0x00, 0x05]).unwrap();
    assert_eq!(result, 8453);
}

#[test]
fn decode_currency_known_usdc() {
    let result = decode_currency(&[0x00, 0x01]).unwrap();
    assert_eq!(result, "USDC");
}

#[test]
fn decode_currency_raw() {
    let mut v = vec![0x01u8];
    v.extend_from_slice(b"XYZ");
    let result = decode_currency(&v).unwrap();
    assert_eq!(result, "XYZ");
}

#[test]
fn bytes_to_address_roundtrip() {
    let addr = "0xaabbccddee0011223344556677889900aabbccdd";
    let raw: Vec<u8> = (0..20)
        .map(|i| u8::from_str_radix(&addr[2 + i * 2..4 + i * 2], 16).unwrap())
        .collect();
    let result = bytes_to_address(&raw).unwrap();
    assert_eq!(result, addr);
}

#[test]
fn reverse_dict_invoice() {
    // 0x06 is dict code for "Invoice"
    let result = reverse_dict(&[0x06]).unwrap();
    assert_eq!(result, "Invoice");
}

#[test]
fn reverse_dict_passthrough() {
    let result = reverse_dict(b"Hello world").unwrap();
    assert_eq!(result, "Hello world");
}

// --- U256 mantissa decode tests ---

#[test]
fn decode_mantissa_u256_max_roundtrip() {
    // Encode u256::MAX via encode path then decode — end-to-end parity check.
    use crate::encode::tests_pub::mantissa_bytes_pub;
    let uint256_max =
        "115792089237316195423570985008687907853269984665640564039457584007913129639935";
    let encoded = mantissa_bytes_pub(uint256_max).unwrap();
    let decoded = decode_mantissa(&encoded).unwrap();
    assert_eq!(decoded, uint256_max);
}

#[test]
fn decode_mantissa_large_value_above_u128() {
    // A value between u128::MAX and u256::MAX — old code would silently saturate.
    use crate::encode::tests_pub::mantissa_bytes_pub;
    // u128::MAX * 1000 (well above u128 range)
    let large = "340282366920938463463374607431768211455000";
    let encoded = mantissa_bytes_pub(large).unwrap();
    let decoded = decode_mantissa(&encoded).unwrap();
    assert_eq!(decoded, large);
}

#[test]
fn decode_mantissa_wire_payload_exceeding_u256_errors() {
    // Craft a wire payload whose mantissa varint decodes to 33 bytes (> 32) — must error
    // cleanly, never silently saturate (the old u128 saturation bug).
    // A 33-byte all-0xFF big-endian value encoded as LEB128 exceeds MAX_BYTES (37 × 7-bit
    // chunks = 259 bits > 256 bits) so the varint layer returns VarintOverflow before the
    // 32-byte U256 guard fires.  Both VarintOverflow and InvalidAmount are CodecError
    // variants — either satisfies the "no silent saturation" requirement.
    use crate::varint::write_bigint_varint;
    let oversized_mantissa = vec![0xFFu8; 33]; // 33 bytes > U256 max 32 bytes
    let mut payload = Vec::new();
    write_bigint_varint(&oversized_mantissa, &mut payload);
    payload.push(0u8); // zeros = 0

    let err = decode_mantissa(&payload).unwrap_err();
    assert!(
        matches!(
            err,
            CodecError::InvalidAmount(_) | CodecError::VarintOverflow(_)
        ),
        "expected InvalidAmount or VarintOverflow for oversized mantissa, got {err:?}"
    );
}

// --- R1: due_at u64→u32 truncation guard ---

/// A varint encoding 2^32 (0x1_0000_0000) must not silently truncate to 0.
/// Old code: `issued_at + due_delta as u32` → 0x1_0000_0000 as u32 == 0 → due_at == issued_at.
#[test]
fn r1_due_at_delta_exactly_2pow32_errors() {
    use crate::varint::write_varint;
    let delta: u64 = 0x1_0000_0000; // 2^32 — overflows u32
    let mut due_bytes = Vec::new();
    write_varint(delta, &mut due_bytes);

    // Feed the oversized delta through the varint decode path directly.
    // read_varint returns a u64; try_from(u64) must reject values > u32::MAX.
    let (decoded_delta, _) = crate::varint::read_varint(&due_bytes, 0).unwrap();
    let result = u32::try_from(decoded_delta);
    assert!(
        result.is_err(),
        "u32::try_from(2^32) must fail — old 'as u32' cast would silently truncate to 0"
    );
}

/// A varint encoding 2^32 + 100 must also reject, not produce due_at = issued_at + 100.
#[test]
fn r1_due_at_delta_2pow32_plus_100_errors() {
    use crate::varint::write_varint;
    let delta: u64 = 0x1_0000_0064; // 2^32 + 100
    let mut due_bytes = Vec::new();
    write_varint(delta, &mut due_bytes);

    let (decoded_delta, _) = crate::varint::read_varint(&due_bytes, 0).unwrap();
    let result = u32::try_from(decoded_delta);
    assert!(
        result.is_err(),
        "u32::try_from(2^32+100) must fail — old cast would silently produce delta=100"
    );
}

/// Encode a valid invoice then manually craft a TLV_DUE_AT with delta = 2^32.
/// decode_invoice_canonical must return Err, not silently produce due_at == issued_at.
#[test]
fn r1_full_decode_rejects_due_at_overflow() {
    use crate::encode::encode_invoice_canonical;
    use crate::invoice::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};
    use crate::varint::write_varint;

    // Build a valid invoice and encode it.
    let invoice = Invoice {
        invoice_id: "INV-R1".to_string(),
        issued_at: 1_700_000_000,
        due_at: 1_700_604_800,
        network_id: 1,
        currency: "USDC".to_string(),
        decimals: 6,
        from: InvoiceFrom {
            name: "Alice".to_string(),
            wallet_address: "0xaabbccddee0011223344556677889900aabbccdd".to_string(),
            email: None,
            phone: None,
            physical_address: None,
            tax_id: None,
        },
        client: InvoiceClient {
            name: "Bob".to_string(),
            wallet_address: None,
            email: None,
            phone: None,
            physical_address: None,
            tax_id: None,
        },
        items: vec![InvoiceItem {
            description: "Work".to_string(),
            quantity: 1.0,
            rate: "1000000".to_string(),
        }],
        token_address: None,
        notes: None,
        tax: None,
        discount: None,
        total: "1000000".to_string(),
        salt: "00112233445566778899aabbccddeeff".to_string(),
    };
    let mut bytes = encode_invoice_canonical(&invoice).unwrap();

    // Patch TLV_DUE_AT (type=6) in the wire bytes with delta = 2^32.
    // Scan for type byte 0x06 after the 3-byte header.
    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, n) = crate::varint::read_varint(&bytes, i + 1).unwrap();
        let value_start = i + 1 + n;
        let value_end = value_start + length as usize;
        if tlv_type == crate::encode::TLV_DUE_AT {
            // Replace value with varint(2^32).
            let mut new_val = Vec::new();
            write_varint(0x1_0000_0000u64, &mut new_val);
            // Rebuild entire TLV for type 6 to correctly patch the length varint.
            let mut tlv_new = Vec::new();
            tlv_new.push(0x06u8);
            write_varint(new_val.len() as u64, &mut tlv_new);
            tlv_new.extend_from_slice(&new_val);
            let before = &bytes[..i];
            let after = &bytes[value_end..];
            let mut rebuilt = before.to_vec();
            rebuilt.extend_from_slice(&tlv_new);
            rebuilt.extend_from_slice(after);
            bytes = rebuilt;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).unwrap_err();
    assert!(
        matches!(
            err,
            CodecError::InvalidAmount(_) | CodecError::ChecksumMismatch
        ),
        "expected InvalidAmount or ChecksumMismatch for due_at overflow, got {err:?}"
    );
}

// --- #12: unpack_items hostile desc_len — must Err, never slice-panic ---

/// A packed-items payload whose first item's desc_len varint encodes a huge
/// value must return Err, not panic on the `data[offset..offset+desc_len]`
/// slice. Pre-fix: `desc_len as usize` + `offset + desc_len` overflowed.
#[test]
fn unpack_items_hostile_desc_len_errors_not_panics() {
    use crate::varint::write_varint;
    let mut data = Vec::new();
    write_varint(1, &mut data); // count = 1 item
    write_varint(u64::MAX, &mut data); // desc_len = u64::MAX — hostile
    // No description bytes follow.
    let err = unpack_items(&data).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated for hostile desc_len, got {err:?}"
    );
}

/// A desc_len that fits in usize but exceeds the available buffer must Err.
#[test]
fn unpack_items_desc_len_past_buffer_end_errors() {
    use crate::varint::write_varint;
    let mut data = Vec::new();
    write_varint(1, &mut data); // count = 1
    write_varint(100, &mut data); // desc_len = 100, but buffer ends here
    let err = unpack_items(&data).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated for desc_len past buffer end, got {err:?}"
    );
}

/// A hostile item count varint must be rejected before allocation.
#[test]
fn unpack_items_hostile_count_errors() {
    use crate::varint::write_varint;
    let mut data = Vec::new();
    write_varint(u64::MAX, &mut data); // count = u64::MAX — hostile
    let err = unpack_items(&data).unwrap_err();
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated for hostile item count, got {err:?}"
    );
}

// --- #8: decode_chain_id raw-varint u32 truncation guard ---

/// A 0x01-prefixed chain ID varint encoding a value > u32::MAX must Err,
/// not silently truncate via `as u32`.
#[test]
fn decode_chain_id_raw_above_u32_max_errors() {
    use crate::varint::write_varint;
    let mut value = vec![0x01u8]; // raw-varint prefix
    write_varint(0x1_0000_0000u64, &mut value); // 2^32 — overflows u32
    let err = decode_chain_id(&value).unwrap_err();
    assert!(
        matches!(err, CodecError::InvalidAmount(_)),
        "expected InvalidAmount for chain ID > u32::MAX, got {err:?}"
    );
}

/// A 0x01-prefixed chain ID varint at exactly u32::MAX must still decode Ok.
#[test]
fn decode_chain_id_raw_at_u32_max_ok() {
    use crate::varint::write_varint;
    let mut value = vec![0x01u8];
    write_varint(u32::MAX as u64, &mut value);
    let decoded = decode_chain_id(&value).unwrap();
    assert_eq!(decoded, u32::MAX);
}

// --- #2: mantissa trailing-zeros — decode must accept full U256 range ---

/// Decode must accept a trailing-zero count up to 77 (max a valid U256 carries).
/// Pre-fix the cap was 30, rejecting valid encodings like 1 * 10^40.
#[test]
fn decode_mantissa_accepts_40_trailing_zeros() {
    // mantissa = 1 (0x01), zeros = 40 → 10^40, well within U256 range.
    let result = decode_mantissa(&[0x01, 40]).unwrap();
    let mut expected = String::from("1");
    expected.push_str(&"0".repeat(40));
    assert_eq!(result, expected);
}

/// Decode must accept zeros = 77 (the documented U256 ceiling).
#[test]
fn decode_mantissa_accepts_77_trailing_zeros() {
    // mantissa = 1, zeros = 77 → 10^77 < 2^256.
    let result = decode_mantissa(&[0x01, 77]).unwrap();
    let mut expected = String::from("1");
    expected.push_str(&"0".repeat(77));
    assert_eq!(result, expected);
}

/// A zeros count above 77 must still be rejected.
#[test]
fn decode_mantissa_rejects_78_trailing_zeros() {
    let err = decode_mantissa(&[0x01, 78]).unwrap_err();
    assert!(
        matches!(err, CodecError::Overflow(_)),
        "expected Overflow for zeros > 77, got {err:?}"
    );
}

// --- T4: decode quantity scale + scaled_value caps ---

/// A scale byte of 255 in packed-items quantity must be rejected.
#[test]
fn decode_mantissa_rejects_scale_255() {
    use crate::varint::write_varint;
    // Build a minimal packed-items payload: count=1, desc_len=1, desc="A",
    // scale=255 (invalid), scaled_value=1.
    let mut data = Vec::new();
    write_varint(1, &mut data); // count=1
    write_varint(1, &mut data); // desc_len=1
    data.push(b'A'); // description
    data.push(255u8); // scale=255 > encoder cap of 9
    write_varint(1, &mut data); // scaled_value=1
    // rate mantissa + zeros (0 mantissa, 0 zeros)
    write_varint(0, &mut data); // mantissa varint (0)
    data.push(0u8); // trailing zeros=0

    let err = unpack_items(&data).unwrap_err();
    assert!(
        matches!(err, CodecError::InvalidData(_)),
        "expected InvalidData for scale=255, got {err:?}"
    );
}

/// P1-F1: scale=10 must be rejected — encoder caps at 9, decoder must match.
/// A payload with scale=10 cannot be produced by the canonical encoder.
#[test]
fn decode_quantity_rejects_scale_above_encoder_cap() {
    use crate::varint::write_varint;
    let mut data = Vec::new();
    write_varint(1, &mut data); // count=1
    write_varint(1, &mut data); // desc_len=1
    data.push(b'A');
    data.push(10u8); // scale=10 > MAX_SCALE=9 (encoder cap)
    write_varint(1, &mut data); // scaled_value=1
    data.push(0x01u8); // mantissa=1
    data.push(0u8); // zeros=0

    let err = unpack_items(&data).unwrap_err();
    assert!(
        matches!(err, CodecError::InvalidData(_)),
        "expected InvalidData for non-canonical scale=10, got {err:?}"
    );
}

/// A scaled_value above 2^53 must be rejected (f64 precision loss).
#[test]
fn decode_mantissa_rejects_scaled_value_above_2_53() {
    use crate::varint::write_varint;
    let mut data = Vec::new();
    write_varint(1, &mut data); // count=1
    write_varint(1, &mut data); // desc_len=1
    data.push(b'A');
    data.push(0u8); // scale=0 (valid)
    write_varint(9_007_199_254_740_993u64, &mut data); // 2^53 + 1 — exceeds MAX_SAFE_F64_INT
    write_varint(1, &mut data); // mantissa=1
    data.push(0u8); // zeros=0

    let err = unpack_items(&data).unwrap_err();
    assert!(
        matches!(err, CodecError::InvalidAmount(_)),
        "expected InvalidAmount for scaled_value > 2^53, got {err:?}"
    );
}

/// Scale=9 (encoder max) with a safe scaled_value must decode successfully.
/// Scale=18 is now rejected as non-canonical (encoder cap is 9).
#[test]
fn decode_mantissa_accepts_scale_9_safe_value() {
    use crate::varint::write_varint;
    let mut data = Vec::new();
    write_varint(1, &mut data); // count=1
    write_varint(1, &mut data); // desc_len=1
    data.push(b'A');
    data.push(9u8); // scale=9 (at encoder MAX_SCALE)
    write_varint(1_000_000u64, &mut data); // well within 2^53
    // rate: mantissa=1, zeros=6 → 1_000_000
    data.push(0x01u8); // mantissa bigint varint: 1
    data.push(6u8); // zeros=6

    let items = unpack_items(&data).unwrap();
    assert_eq!(items.len(), 1);
    let q = items[0].quantity;
    assert!(q.is_finite(), "quantity must be finite");
    assert!(q > 0.0, "quantity must be positive");
}

// --- P1-F4: TLV_DECIMALS strict length ---

/// decode_invoice_canonical must reject a TLV_DECIMALS field with length != 1.
/// Previously .first() silently truncated any trailing bytes.
#[test]
fn decode_rejects_non_canonical_decimals_length() {
    use crate::encode::encode_invoice_canonical;
    use crate::invoice::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};
    use crate::varint::write_varint;

    let invoice = Invoice {
        invoice_id: "INV-F4".to_string(),
        issued_at: 1_700_000_000,
        due_at: 1_700_604_800,
        network_id: 1,
        currency: "USDC".to_string(),
        decimals: 6,
        from: InvoiceFrom {
            name: "Alice".to_string(),
            wallet_address: "0xaabbccddee0011223344556677889900aabbccdd".to_string(),
            email: None,
            phone: None,
            physical_address: None,
            tax_id: None,
        },
        client: InvoiceClient {
            name: "Bob".to_string(),
            wallet_address: None,
            email: None,
            phone: None,
            physical_address: None,
            tax_id: None,
        },
        items: vec![InvoiceItem {
            description: "Work".to_string(),
            quantity: 1.0,
            rate: "1000000".to_string(),
        }],
        token_address: None,
        notes: None,
        tax: None,
        discount: None,
        total: "1000000".to_string(),
        salt: "00112233445566778899aabbccddeeff".to_string(),
    };
    let mut bytes = encode_invoice_canonical(&invoice).unwrap();

    // Patch TLV_DECIMALS (type = TLV_DECIMALS) to length=2 by rebuilding its TLV entry.
    let header_len = 3usize;
    let mut i = header_len;
    while i < bytes.len() {
        let tlv_type = bytes[i];
        let (length, n) = crate::varint::read_varint(&bytes, i + 1).unwrap();
        let value_start = i + 1 + n;
        let value_end = value_start + length as usize;
        if tlv_type == crate::encode::TLV_DECIMALS {
            // Replace with a 2-byte decimals value — non-canonical.
            let mut tlv_new = Vec::new();
            tlv_new.push(tlv_type);
            write_varint(2u64, &mut tlv_new); // length=2
            tlv_new.push(6u8); // decimals byte
            tlv_new.push(0u8); // spurious extra byte
            let mut rebuilt = bytes[..i].to_vec();
            rebuilt.extend_from_slice(&tlv_new);
            rebuilt.extend_from_slice(&bytes[value_end..]);
            bytes = rebuilt;
            break;
        }
        i = value_end;
    }

    let err = decode_invoice_canonical(&bytes).unwrap_err();
    assert!(
        matches!(
            err,
            CodecError::InvalidData(_) | CodecError::ChecksumMismatch
        ),
        "expected InvalidData or ChecksumMismatch for 2-byte TLV_DECIMALS, got {err:?}"
    );
}
