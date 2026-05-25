//! Shared test helpers for integration test files.

#![allow(dead_code)]

use serde::Deserialize;
use void_layer_codec::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};

// ---------------------------------------------------------------------------
// Vector schema (mirrors vectors/v4-codec.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VectorFile {
    pub vectors: Vec<Vector>,
}

/// A single test vector. Fields are optional because malformed vectors only
/// have a subset of them.
#[derive(Debug, Deserialize)]
pub struct Vector {
    pub name: String,
    /// Present on non-malformed vectors and canonical-malformed vectors.
    pub canonical_hex: Option<String>,
    /// Present on non-malformed and encode-input malformed vectors.
    pub decoded: Option<DecodedInvoice>,
    /// True for non-malformed roundtrip vectors.
    pub roundtrip: Option<bool>,
    /// Classification string.
    #[allow(dead_code)]
    pub diagnostic: String,
    /// Expected error variant name (present on malformed vectors).
    #[allow(dead_code)]
    pub expected_error: Option<String>,
}

/// JSON representation of the Invoice structure as stored in the vector file.
#[derive(Debug, Deserialize)]
pub struct DecodedInvoice {
    pub invoice_id: String,
    pub issued_at: u32,
    pub due_at: u32,
    pub network_id: u32,
    pub currency: String,
    pub decimals: u8,
    pub from: DecodedFrom,
    pub client: DecodedClient,
    pub items: Vec<DecodedItem>,
    #[serde(default)]
    pub token_address: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub tax: Option<String>,
    #[serde(default)]
    pub discount: Option<String>,
    pub total: String,
    pub salt: String,
}

#[derive(Debug, Deserialize)]
pub struct DecodedFrom {
    pub name: String,
    pub wallet_address: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub physical_address: Option<String>,
    #[serde(default)]
    pub tax_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecodedClient {
    pub name: String,
    #[serde(default)]
    pub wallet_address: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub physical_address: Option<String>,
    #[serde(default)]
    pub tax_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DecodedItem {
    pub description: String,
    pub quantity: f64,
    pub rate: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn load_vectors() -> VectorFile {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/vectors/v4-codec.json");
    let raw = std::fs::read_to_string(path).expect("vectors/v4-codec.json must exist");
    serde_json::from_str(&raw).expect("v4-codec.json must be valid JSON")
}

pub fn to_invoice(d: &DecodedInvoice) -> Invoice {
    Invoice {
        invoice_id: d.invoice_id.clone(),
        issued_at: d.issued_at,
        due_at: d.due_at,
        network_id: d.network_id,
        currency: d.currency.clone(),
        decimals: d.decimals,
        from: InvoiceFrom {
            name: d.from.name.clone(),
            wallet_address: d.from.wallet_address.clone(),
            email: d.from.email.clone(),
            phone: d.from.phone.clone(),
            physical_address: d.from.physical_address.clone(),
            tax_id: d.from.tax_id.clone(),
        },
        client: InvoiceClient {
            name: d.client.name.clone(),
            wallet_address: d.client.wallet_address.clone(),
            email: d.client.email.clone(),
            phone: d.client.phone.clone(),
            physical_address: d.client.physical_address.clone(),
            tax_id: d.client.tax_id.clone(),
        },
        items: d
            .items
            .iter()
            .map(|i| InvoiceItem {
                description: i.description.clone(),
                quantity: i.quantity,
                rate: i.rate.clone(),
            })
            .collect(),
        token_address: d.token_address.clone(),
        notes: d.notes.clone(),
        tax: d.tax.clone(),
        discount: d.discount.clone(),
        total: d.total.clone(),
        salt: d.salt.clone(),
    }
}

pub fn from_hex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
        .collect()
}

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
