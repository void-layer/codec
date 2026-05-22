//! Corpus-driven canonical roundtrip — Rust surface.
//!
//! Reads vectors/corpus.json and for every entry asserts:
//!   - encode_invoice_canonical(decoded) hex == canonical_hex
//!   - decode_invoice_canonical(from_hex(canonical_hex)) == decoded

#![cfg(not(target_arch = "wasm32"))]

use serde::Deserialize;
use void_layer_codec::{
    Invoice, InvoiceClient, InvoiceFrom, InvoiceItem, decode_invoice_canonical,
    encode_invoice_canonical,
};

// ---------------------------------------------------------------------------
// Corpus schema (mirrors corpus.json entries)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct CorpusFile {
    entries: Vec<CorpusEntry>,
}

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    name: String,
    canonical_hex: String,
    decoded: DecodedInvoice,
}

#[derive(Debug, Deserialize)]
struct DecodedInvoice {
    invoice_id: String,
    issued_at: u32,
    due_at: u32,
    network_id: u32,
    currency: String,
    decimals: u8,
    from: DecodedFrom,
    client: DecodedClient,
    items: Vec<DecodedItem>,
    #[serde(default)]
    token_address: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    tax: Option<String>,
    #[serde(default)]
    discount: Option<String>,
    total: String,
    salt: String,
}

#[derive(Debug, Deserialize)]
struct DecodedFrom {
    name: String,
    wallet_address: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    physical_address: Option<String>,
    #[serde(default)]
    tax_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DecodedClient {
    name: String,
    #[serde(default)]
    wallet_address: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    physical_address: Option<String>,
    #[serde(default)]
    tax_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DecodedItem {
    description: String,
    quantity: f64,
    rate: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_corpus() -> CorpusFile {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/vectors/corpus.json");
    let raw = std::fs::read_to_string(path).expect("vectors/corpus.json must exist");
    serde_json::from_str(&raw).expect("corpus.json must be valid JSON")
}

fn to_invoice(d: &DecodedInvoice) -> Invoice {
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

fn from_hex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex"))
        .collect()
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn corpus_canonical_encode_all() {
    let file = load_corpus();
    let mut failures: Vec<String> = Vec::new();

    for entry in &file.entries {
        let invoice = to_invoice(&entry.decoded);
        match encode_invoice_canonical(&invoice) {
            Ok(bytes) => {
                let actual = to_hex(&bytes);
                if actual != entry.canonical_hex {
                    failures.push(format!(
                        "ENCODE MISMATCH entry={}\n  expected: {}\n  actual:   {}",
                        entry.name, entry.canonical_hex, actual
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("ENCODE ERROR entry={}: {e:?}", entry.name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Corpus canonical encode failures:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn corpus_canonical_decode_all() {
    let file = load_corpus();
    let mut failures: Vec<String> = Vec::new();

    for entry in &file.entries {
        let bytes = from_hex(&entry.canonical_hex);
        match decode_invoice_canonical(&bytes) {
            Ok(actual) => {
                let expected = to_invoice(&entry.decoded);
                if actual != expected {
                    failures.push(format!(
                        "DECODE MISMATCH entry={}\n  expected: {expected:?}\n  actual:   {actual:?}",
                        entry.name
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("DECODE ERROR entry={}: {e:?}", entry.name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Corpus canonical decode failures:\n{}",
        failures.join("\n\n")
    );
}
