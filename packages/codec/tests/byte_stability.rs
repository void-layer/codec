//! G-01: encode(decode(encode(inv))) == encode(inv) byte-stable

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{
    Invoice, InvoiceClient, InvoiceFrom, InvoiceItem, decode_invoice_canonical,
    encode_invoice_canonical,
};

#[test]
fn g01_encode_decode_encode_is_byte_stable() {
    let invoice = minimal_invoice();
    let bytes1 = encode_invoice_canonical(&invoice).expect("first encode");
    let decoded = decode_invoice_canonical(&bytes1).expect("decode");
    let bytes2 = encode_invoice_canonical(&decoded).expect("second encode");
    assert_eq!(
        to_hex(&bytes1),
        to_hex(&bytes2),
        "encode(decode(encode(inv))) must equal encode(inv)"
    );
}

#[test]
fn g01_encode_decode_encode_byte_stable_with_all_optional_fields() {
    let invoice = Invoice {
        invoice_id: "INV-FULL".to_string(),
        issued_at: 1_748_000_000,
        due_at: 1_748_604_800,
        network_id: 8453,
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
        items: vec![InvoiceItem {
            description: "Development".to_string(),
            quantity: 2.5,
            rate: "500000000000000000".to_string(),
        }],
        token_address: None,
        notes: Some("Thank you".to_string()),
        tax: Some("10".to_string()),
        discount: Some("5".to_string()),
        total: "1250000000000000000".to_string(),
        salt: "aabbccddeeff00112233445566778899".to_string(),
    };
    let bytes1 = encode_invoice_canonical(&invoice).expect("first encode");
    let decoded = decode_invoice_canonical(&bytes1).expect("decode");
    let bytes2 = encode_invoice_canonical(&decoded).expect("second encode");
    assert_eq!(
        to_hex(&bytes1),
        to_hex(&bytes2),
        "full invoice: encode(decode(encode(inv))) must equal encode(inv)"
    );
}
