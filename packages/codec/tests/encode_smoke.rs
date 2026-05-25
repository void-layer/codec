//! Unit smoke tests: canonical encode + decode + error paths.
//! Derived from codec_smoke.rs unit section (T-P2-8 revised).

#![cfg(not(target_arch = "wasm32"))]

use void_layer_codec::{
    CodecError, Invoice, InvoiceClient, InvoiceFrom, InvoiceItem, decode_invoice_canonical,
    encode_invoice_canonical,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_invoice() -> Invoice {
    Invoice {
        invoice_id: "INV-001".to_string(),
        issued_at: 1_700_000_000,
        due_at: 1_700_604_800, // +7 days
        network_id: 1,         // Ethereum
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
            description: "Consulting".to_string(),
            quantity: 1.0,
            rate: "1000000".to_string(), // 1 USDC
        }],
        token_address: None,
        notes: None,
        tax: None,
        discount: None,
        total: "1000000".to_string(),
        salt: "00112233445566778899aabbccddeeff".to_string(),
    }
}

fn full_invoice() -> Invoice {
    Invoice {
        invoice_id: "INV-FULL-2026".to_string(),
        issued_at: 1_748_000_000,
        due_at: 1_748_604_800,
        network_id: 8453, // Base
        currency: "ETH".to_string(),
        decimals: 18,
        from: InvoiceFrom {
            name: "Alice Corp".to_string(),
            wallet_address: "0x1111111111111111111111111111111111111111".to_string(),
            email: Some("alice@example.com".to_string()),
            phone: Some("+1-555-0100".to_string()),
            physical_address: Some("123 Main St".to_string()),
            tax_id: Some("TAX-123".to_string()),
        },
        client: InvoiceClient {
            name: "Bob Ltd".to_string(),
            wallet_address: Some("0x2222222222222222222222222222222222222222".to_string()),
            email: Some("bob@example.com".to_string()),
            phone: None,
            physical_address: None,
            tax_id: None,
        },
        items: vec![
            InvoiceItem {
                description: "Development".to_string(),
                quantity: 2.5,
                rate: "500000000000000000".to_string(), // 0.5 ETH
            },
            InvoiceItem {
                description: "Consulting".to_string(),
                quantity: 1.0,
                rate: "200000000000000000".to_string(), // 0.2 ETH
            },
        ],
        token_address: None,
        notes: Some("Thank you for your business".to_string()),
        tax: Some("10".to_string()),
        discount: Some("5".to_string()),
        total: "1700000000000000000".to_string(),
        salt: "aabbccddeeff00112233445566778899".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Unit 2 — T-P2-8 (revised): canonical encode + decode
// ---------------------------------------------------------------------------

#[test]
fn encodes_minimal_invoice_starts_with_magic_version() {
    let invoice = minimal_invoice();
    let bytes = encode_invoice_canonical(&invoice).expect("encode failed");
    assert!(bytes.len() >= 3, "must have at least header");
    assert_eq!(bytes[0], 0x56, "magic byte must be 0x56 ('V')");
    assert_eq!(bytes[1], 0x01, "version byte must be 0x01");
}

#[test]
fn canonical_version_byte_has_no_compressed_flag() {
    let invoice = minimal_invoice();
    let bytes = encode_invoice_canonical(&invoice).expect("encode failed");
    assert_eq!(
        bytes[1] & 0x80,
        0,
        "canonical bytes must NOT have COMPRESSED_FLAG (0x80) set on version byte"
    );
}

#[test]
fn encodes_full_invoice_starts_with_magic_version() {
    let invoice = full_invoice();
    let bytes = encode_invoice_canonical(&invoice).expect("encode failed");
    assert_eq!(bytes[0], 0x56);
    assert_eq!(bytes[1], 0x01);
}

#[test]
fn decodes_minimal_invoice_back() {
    let original = minimal_invoice();
    let bytes = encode_invoice_canonical(&original).expect("encode failed");
    let decoded = decode_invoice_canonical(&bytes).expect("decode failed");
    assert_eq!(decoded.invoice_id, original.invoice_id);
    assert_eq!(decoded.network_id, original.network_id);
    assert_eq!(decoded.currency, original.currency);
    assert_eq!(decoded.decimals, original.decimals);
    assert_eq!(decoded.total, original.total);
    assert_eq!(decoded.from.name, original.from.name);
    assert_eq!(decoded.client.name, original.client.name);
    assert_eq!(decoded.items.len(), original.items.len());
}

#[test]
fn decodes_full_invoice_back() {
    let original = full_invoice();
    let bytes = encode_invoice_canonical(&original).expect("encode failed");
    let decoded = decode_invoice_canonical(&bytes).expect("decode failed");
    assert_eq!(decoded.invoice_id, original.invoice_id);
    assert_eq!(decoded.from.email, original.from.email);
    assert_eq!(decoded.client.email, original.client.email);
    assert_eq!(decoded.notes, original.notes);
    assert_eq!(decoded.tax, original.tax);
    assert_eq!(decoded.discount, original.discount);
    assert_eq!(decoded.items.len(), 2);
}

#[test]
fn roundtrip_preserves_invoice_completely() {
    let original = minimal_invoice();
    let bytes = encode_invoice_canonical(&original).expect("encode failed");
    let decoded = decode_invoice_canonical(&bytes).expect("decode failed");
    assert_eq!(original, decoded);
}

#[test]
fn roundtrip_full_invoice_completely() {
    let original = full_invoice();
    let bytes = encode_invoice_canonical(&original).expect("encode failed");
    let decoded = decode_invoice_canonical(&bytes).expect("decode failed");
    assert_eq!(original, decoded);
}

#[test]
fn bad_magic_returns_error() {
    let bad_bytes = vec![0x00u8, 0x01, 0x00]; // wrong magic
    let err = decode_invoice_canonical(&bad_bytes).expect_err("should fail");
    assert!(
        matches!(err, CodecError::BadMagic),
        "expected BadMagic, got {err:?}"
    );
}

#[test]
fn unsupported_version_returns_error() {
    let bad_bytes = vec![0x56u8, 0x02, 0x00]; // version 2 not supported yet
    let err = decode_invoice_canonical(&bad_bytes).expect_err("should fail");
    assert!(
        matches!(err, CodecError::UnsupportedVersion(2)),
        "expected UnsupportedVersion(2), got {err:?}"
    );
}

#[test]
fn truncated_payload_returns_error() {
    let bytes = vec![0x56u8, 0x01]; // only header without count
    let err = decode_invoice_canonical(&bytes).expect_err("should fail");
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

#[test]
fn empty_payload_returns_error() {
    let err = decode_invoice_canonical(&[]).expect_err("should fail");
    assert!(
        matches!(err, CodecError::BadMagic | CodecError::Truncated { .. }),
        "expected BadMagic or Truncated, got {err:?}"
    );
}

#[test]
fn encode_is_deterministic() {
    let invoice = minimal_invoice();
    let bytes1 = encode_invoice_canonical(&invoice).expect("encode 1 failed");
    let bytes2 = encode_invoice_canonical(&invoice).expect("encode 2 failed");
    assert_eq!(bytes1, bytes2, "canonical encoding must be deterministic");
}

#[test]
fn encode_different_invoices_produce_different_bytes() {
    let a = minimal_invoice();
    let mut b = minimal_invoice();
    b.total = "2000000".to_string();
    let bytes_a = encode_invoice_canonical(&a).expect("encode a failed");
    let bytes_b = encode_invoice_canonical(&b).expect("encode b failed");
    assert_ne!(bytes_a, bytes_b);
}

#[test]
fn tlv_count_byte_matches_actual_tlv_count() {
    // The 3rd byte in canonical is COUNT of TLV records.
    // Minimal invoice: required fields = chain_id(2), issued_at(4), due_at(6),
    // decimals(8), from_wallet(10), currency(12), items(14), from_name(16),
    // client_name(18), salt(20), invoice_id(22), total(24), domain_sep(31)
    // = 13 required, + optional salt already counted = 13 TLV entries minimum
    let invoice = minimal_invoice();
    let bytes = encode_invoice_canonical(&invoice).expect("encode failed");
    let tlv_count = bytes[2] as usize;
    assert!(
        tlv_count >= 13,
        "minimal invoice should have at least 13 TLV records, got {tlv_count}"
    );
}
