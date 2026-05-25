//! G-08, G-09, G-10, G-11: items count/description/quantity edges

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::*;

use void_layer_codec::{
    CodecError, InvoiceItem, decode_invoice_canonical, encode_invoice_canonical,
};

// ---------------------------------------------------------------------------
// G-08: unpack_items with count=0 → Ok(empty vec)
// ---------------------------------------------------------------------------

#[test]
fn g08_unpack_items_count_zero_returns_empty_vec() {
    let mut invoice = minimal_invoice();
    invoice.items = vec![];
    let result = encode_invoice_canonical(&invoice);
    match result {
        Ok(bytes) => {
            let decoded = decode_invoice_canonical(&bytes).expect("decode with 0 items");
            assert!(
                decoded.items.is_empty(),
                "0 items must roundtrip as empty vec"
            );
        }
        Err(e) => {
            assert!(
                matches!(e, CodecError::Overflow(_) | CodecError::InvalidAmount(_)),
                "0 items encode error must be Overflow or InvalidAmount, got {e:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// G-09: unpack_items with item having empty description string
// ---------------------------------------------------------------------------

#[test]
fn g09_item_with_empty_description_roundtrips() {
    let mut invoice = minimal_invoice();
    invoice.items = vec![InvoiceItem {
        description: String::new(),
        quantity: 1.0,
        rate: "1000000".to_string(),
    }];
    let bytes = encode_invoice_canonical(&invoice).expect("encode with empty description");
    let decoded = decode_invoice_canonical(&bytes).expect("decode with empty description");
    assert_eq!(
        decoded.items[0].description, "",
        "empty description must roundtrip"
    );
}

// ---------------------------------------------------------------------------
// G-10: write_quantity(0.0) → [scale=0x00, value=0x00]
// ---------------------------------------------------------------------------

#[test]
fn g10_write_quantity_zero_encodes_as_two_zeros() {
    let mut invoice = minimal_invoice();
    invoice.items = vec![InvoiceItem {
        description: "Zero qty item".to_string(),
        quantity: 0.0,
        rate: "1000000".to_string(),
    }];
    let bytes = encode_invoice_canonical(&invoice).expect("encode qty=0.0");
    let decoded = decode_invoice_canonical(&bytes).expect("decode qty=0.0");
    assert_eq!(
        decoded.items[0].quantity, 0.0,
        "quantity=0.0 must roundtrip"
    );
}

// ---------------------------------------------------------------------------
// G-11: write_quantity(0.1234567891) — >9 decimals rejected (T5 fix)
// ---------------------------------------------------------------------------

#[test]
fn g11_write_quantity_clamps_scale_at_9_silently() {
    let mut invoice = minimal_invoice();
    invoice.items = vec![InvoiceItem {
        description: "Fractional qty".to_string(),
        quantity: 0.1234567891,
        rate: "1000000".to_string(),
    }];
    let result = encode_invoice_canonical(&invoice);
    assert!(
        result.is_err(),
        "write_quantity(0.1234567891) must fail with >9 decimals (T5 precision guard)"
    );
    assert!(
        matches!(result.unwrap_err(), CodecError::InvalidAmount(_)),
        "expected InvalidAmount for >9 significant decimals"
    );
}
