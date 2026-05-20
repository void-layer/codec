//! Golden-vector parity test — Rust surface (T-P2-13).
//!
//! Canonical only: Rust has no wire encoder (Brotli lives in the JS shim per B-v C3).
//! Wire parity is covered by tests/parity.test.ts on the TS surface.
//!
//! Reads vectors/v4-codec.json and asserts:
//!   - Non-malformed: encode → canonical_hex matches; decode canonical_hex → decoded payload matches.
//!   - Malformed decode-input (canonical_hex): decode → expected CodecError variant.
//!   - Malformed encode-input (over-u256): encode → CodecError::InvalidAmount.

#![cfg(not(target_arch = "wasm32"))]

use serde::Deserialize;
use void_layer_codec::{
    CodecError, Invoice, InvoiceClient, InvoiceFrom, InvoiceItem, decode_invoice_canonical,
    encode_invoice_canonical,
};

// ---------------------------------------------------------------------------
// Vector schema (mirrors v4-codec.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct VectorFile {
    vectors: Vec<Vector>,
}

/// A single test vector. Fields are optional because malformed vectors only
/// have a subset of them.
#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    /// Present on non-malformed vectors and canonical-malformed vectors.
    canonical_hex: Option<String>,
    /// Present on non-malformed and encode-input malformed vectors.
    decoded: Option<DecodedInvoice>,
    /// True for non-malformed roundtrip vectors.
    roundtrip: Option<bool>,
    /// Classification string.
    #[allow(dead_code)]
    diagnostic: String,
    /// Expected error variant name (present on malformed vectors).
    #[allow(dead_code)]
    expected_error: Option<String>,
}

/// JSON representation of the Invoice structure as stored in the vector file.
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

fn load_vectors() -> VectorFile {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/vectors/v4-codec.json");
    let raw = std::fs::read_to_string(path).expect("vectors/v4-codec.json must exist");
    serde_json::from_str(&raw).expect("v4-codec.json must be valid JSON")
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
// Non-malformed vectors — canonical encode + decode (both directions)
// ---------------------------------------------------------------------------

#[test]
fn parity_canonical_encode_all_non_malformed() {
    let file = load_vectors();
    let mut failures: Vec<String> = Vec::new();

    for v in &file.vectors {
        if v.roundtrip != Some(true) {
            continue;
        }
        let decoded = v
            .decoded
            .as_ref()
            .expect("non-malformed vector has decoded");
        let canonical_hex = v
            .canonical_hex
            .as_deref()
            .expect("non-malformed vector has canonical_hex");

        let invoice = to_invoice(decoded);
        match encode_invoice_canonical(&invoice) {
            Ok(bytes) => {
                let actual = to_hex(&bytes);
                if actual != canonical_hex {
                    failures.push(format!(
                        "ENCODE MISMATCH vector={}\n  expected: {}\n  actual:   {}",
                        v.name, canonical_hex, actual
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("ENCODE ERROR vector={}: {e:?}", v.name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Canonical encode parity failures:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn parity_canonical_decode_all_non_malformed() {
    let file = load_vectors();
    let mut failures: Vec<String> = Vec::new();

    for v in &file.vectors {
        if v.roundtrip != Some(true) {
            continue;
        }
        let expected_decoded = v
            .decoded
            .as_ref()
            .expect("non-malformed vector has decoded");
        let canonical_hex = v
            .canonical_hex
            .as_deref()
            .expect("non-malformed vector has canonical_hex");

        let bytes = from_hex(canonical_hex);
        match decode_invoice_canonical(&bytes) {
            Ok(actual) => {
                let expected = to_invoice(expected_decoded);
                if actual != expected {
                    failures.push(format!(
                        "DECODE MISMATCH vector={}\n  expected: {expected:?}\n  actual:   {actual:?}",
                        v.name
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("DECODE ERROR vector={}: {e:?}", v.name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Canonical decode parity failures:\n{}",
        failures.join("\n\n")
    );
}

// ---------------------------------------------------------------------------
// Malformed decode-input vectors — canonical_hex → expected CodecError variant
// ---------------------------------------------------------------------------

#[test]
fn parity_malformed_varint_overflow() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-varint-overflow")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::VarintOverflow(_)),
        "expected VarintOverflow, got {err:?}"
    );
}

#[test]
fn parity_malformed_checksum_mismatch() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-checksum-mismatch")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::ChecksumMismatch),
        "expected ChecksumMismatch, got {err:?}"
    );
}

#[test]
fn parity_malformed_oversize() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-oversize")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

#[test]
fn parity_malformed_bad_magic() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "malformed-bad-magic")
        .expect("vector must exist");

    let bytes = from_hex(v.canonical_hex.as_deref().expect("has canonical_hex"));
    let err = decode_invoice_canonical(&bytes).expect_err("must fail");
    assert!(
        matches!(err, CodecError::BadMagic),
        "expected BadMagic, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Malformed encode-input vector — bigint-amount-over-u256 → InvalidAmount
// ---------------------------------------------------------------------------

#[test]
fn parity_malformed_encode_input_over_u256() {
    let file = load_vectors();
    let v = file
        .vectors
        .iter()
        .find(|v| v.name == "bigint-amount-over-u256")
        .expect("vector must exist");

    let decoded = v.decoded.as_ref().expect("encode-input vector has decoded");
    let invoice = to_invoice(decoded);
    let err = encode_invoice_canonical(&invoice).expect_err("must fail");
    assert!(
        matches!(err, CodecError::InvalidAmount(_)),
        "expected InvalidAmount, got {err:?}"
    );
}
