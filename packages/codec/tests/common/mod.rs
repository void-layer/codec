//! Shared test helpers for integration test files.

#![allow(dead_code)]

use void_layer_codec::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};

pub fn minimal_invoice() -> Invoice {
    Invoice {
        invoice_id: "INV-001".to_string(),
        issued_at: 1_700_000_000,
        due_at: 1_700_604_800,
        network_id: 1,
        currency: "USDC".to_string(),
        decimals: 6,
        from: InvoiceFrom {
            name: "Alice".to_string(),
            wallet_address: "0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string(),
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
            rate: "1000000".to_string(),
        }],
        token_address: None,
        notes: None,
        tax: None,
        discount: None,
        total: "1000000".to_string(),
        salt: "deadbeefdeadbeefdeadbeefdeadbeef".to_string(),
    }
}

pub fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

pub fn read_varint_from(buf: &[u8], offset: usize) -> (usize, usize) {
    let mut value: u64 = 0;
    let mut shift: u32 = 0;
    let mut n = 0usize;
    loop {
        let b = buf[offset + n];
        n += 1;
        value |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    (value as usize, n)
}
