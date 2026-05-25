//! Golden-vector roundtrip parity — Rust surface (T-P2-13).
//!
//! Canonical only: Rust has no wire encoder (Brotli lives in the JS shim per B-v C3).
//! Wire parity is covered by tests/parity.test.ts on the TS surface.
//!
//! Asserts non-malformed vectors: encode → canonical_hex matches; decode canonical_hex → decoded payload matches.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::{from_hex, load_vectors, to_hex, to_invoice};
use void_layer_codec::{decode_invoice_canonical, encode_invoice_canonical};

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
