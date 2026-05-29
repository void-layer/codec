// T2 — WASM panic guard: non-ASCII bytes in hex strings must return Err, never panic.
// error.rs contract: "never panic on user input".
// &str slicing at non-char-boundary panics in Rust; WASM = unrecoverable abort.

use void_layer_codec::CodecError;

// We call internal functions via a minimal invoice encode path.
// The easiest public surface is encode_invoice_canonical with a crafted token_address / salt.

use void_layer_codec::{
    Invoice, InvoiceClient, InvoiceFrom, InvoiceItem, encode_invoice_canonical,
};

fn base_invoice() -> Invoice {
    Invoice {
        invoice_id: "INV-001".to_string(),
        issued_at: 1_700_000_000,
        due_at: 1_700_086_400,
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
            description: "Work".to_string(),
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

/// address_to_bytes must return Err (not panic) when the hex string contains
/// a non-ASCII multi-byte char before the end of a 40-char prefix.
#[test]
fn address_to_bytes_rejects_non_ascii_mid() {
    // Craft a token_address where position 2..4 contains a 2-byte UTF-8 char (é = 0xC3 0xA9).
    // Total byte length is > 40 but char length may be 40 — slicing &str[2..4] would
    // land mid-char and panic without the ASCII guard.
    let bad_addr = format!("0xab\u{00E9}{}", "0".repeat(36)); // é at chars 4-5
    let mut inv = base_invoice();
    // Use network_id=999 (unknown) so the address won't match any dict entry and
    // will go through address_to_bytes.
    inv.network_id = 999;
    inv.token_address = Some(bad_addr);
    // Must return an error, never panic.
    let result = encode_invoice_canonical(&inv);
    assert!(
        result.is_err(),
        "expected Err for non-ASCII address, got Ok"
    );
    assert!(
        matches!(result.unwrap_err(), CodecError::InvalidAddress(_)),
        "expected InvalidAddress"
    );
}

/// Variant: non-ASCII char after some valid hex prefix.
#[test]
fn address_to_bytes_rejects_non_ascii_late() {
    // 38 valid hex chars + é (2-byte char) — still 40 chars but slicing byte 38..40
    // would land on the first byte of é, panicking without the guard.
    let bad_addr = format!("0x{}\u{00E9}", "a".repeat(38));
    let mut inv = base_invoice();
    inv.network_id = 999;
    inv.token_address = Some(bad_addr);
    let result = encode_invoice_canonical(&inv);
    assert!(result.is_err(), "expected Err for non-ASCII address");
    assert!(matches!(result.unwrap_err(), CodecError::InvalidAddress(_)));
}

/// hex_decode_salt must return Err (not panic) when the salt hex string contains
/// a non-ASCII multi-byte char at an early position.
#[test]
fn hex_decode_salt_rejects_non_ascii_early() {
    // Salt: "ab" + é + 28 valid hex chars — total 32 chars but slicing byte 2..4
    // would land mid-char without the ASCII guard.
    let bad_salt = format!("ab\u{00E9}{}", "0".repeat(28));
    let mut inv = base_invoice();
    inv.salt = bad_salt;
    let result = encode_invoice_canonical(&inv);
    assert!(result.is_err(), "expected Err for non-ASCII salt");
    assert!(matches!(result.unwrap_err(), CodecError::InvalidAddress(_)));
}

/// hex_decode_salt must return Err for non-ASCII at the end (last byte pair position).
#[test]
fn hex_decode_salt_rejects_non_ascii_late() {
    // 30 valid hex chars + é — total 32 chars but slicing [30..32] byte-range
    // hits the first byte of é without the ASCII guard.
    let bad_salt = format!("{}\u{00E9}", "a".repeat(30));
    let mut inv = base_invoice();
    inv.salt = bad_salt;
    let result = encode_invoice_canonical(&inv);
    assert!(result.is_err(), "expected Err for non-ASCII salt (late)");
    assert!(matches!(result.unwrap_err(), CodecError::InvalidAddress(_)));
}
