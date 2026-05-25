//! Proptest-based canonical roundtrip and determinism checks.
//! Includes basic arb_invoice and G-33 extended arb_invoice_with_optionals.

#![cfg(not(target_arch = "wasm32"))]

use void_layer_codec::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};

use proptest::prelude::*;

prop_compose! {
    fn arb_wallet_address()(
        bytes in prop::array::uniform20(any::<u8>())
    ) -> String {
        use std::fmt::Write as _;
        let hex = bytes.iter().fold(String::with_capacity(40), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
        format!("0x{hex}")
    }
}

prop_compose! {
    fn arb_invoice_item()(
        desc in "[a-zA-Z ]{1,20}",
        qty_n in 1u32..100,
        qty_d in 1u32..10,
        rate in 1u64..1_000_000_000u64,
    ) -> InvoiceItem {
        let qty = qty_n as f64 / qty_d as f64;
        // Snap to 2-decimal precision to avoid float encoding edge cases
        let qty = (qty * 100.0).round() / 100.0;
        InvoiceItem {
            description: desc,
            quantity: qty,
            rate: rate.to_string(),
        }
    }
}

prop_compose! {
    fn arb_invoice()(
        wallet in arb_wallet_address(),
        issued_at in 1_600_000_000u32..1_800_000_000u32,
        due_delta in 86400u32..2_592_000u32,
        network_id in prop::sample::select(vec![1u32, 10, 137, 8453, 42161]),
        currency in prop::sample::select(vec!["USDC", "ETH", "USDT", "DAI"]),
        decimals in prop::sample::select(vec![6u8, 18]),
        from_name in "[a-zA-Z ]{1,15}",
        client_name in "[a-zA-Z ]{1,15}",
        item in arb_invoice_item(),
        total in 1u64..1_000_000_000u64,
        salt_bytes in prop::array::uniform16(any::<u8>()),
    ) -> Invoice {
        use std::fmt::Write as _;
        let salt = salt_bytes.iter().fold(String::with_capacity(32), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
        Invoice {
            invoice_id: "INV-001".to_string(),
            issued_at,
            due_at: issued_at + due_delta,
            network_id,
            currency: currency.to_string(),
            decimals,
            from: InvoiceFrom {
                name: from_name,
                wallet_address: wallet,
                email: None,
                phone: None,
                physical_address: None,
                tax_id: None,
            },
            client: InvoiceClient {
                name: client_name,
                wallet_address: None,
                email: None,
                phone: None,
                physical_address: None,
                tax_id: None,
            },
            items: vec![item],
            token_address: None,
            notes: None,
            tax: None,
            discount: None,
            total: total.to_string(),
            salt,
        }
    }
}

proptest! {
    #[test]
    fn canonical_roundtrip(inv in arb_invoice()) {
        use void_layer_codec::{decode_invoice_canonical, encode_invoice_canonical};
        let bytes = encode_invoice_canonical(&inv).unwrap();
        let decoded = decode_invoice_canonical(&bytes).unwrap();
        prop_assert_eq!(inv, decoded);
    }

    #[test]
    fn canonical_encoding_is_deterministic(inv in arb_invoice()) {
        use void_layer_codec::encode_invoice_canonical;
        let bytes1 = encode_invoice_canonical(&inv).unwrap();
        let bytes2 = encode_invoice_canonical(&inv).unwrap();
        prop_assert_eq!(bytes1, bytes2);
    }
}

// ---------------------------------------------------------------------------
// G-33: extended arb_invoice with optional fields at controlled probability.
// ---------------------------------------------------------------------------

prop_compose! {
    /// Optional ASCII string for email, phone, notes, tax, discount fields.
    /// Uses a simple charset that avoids dict reserved codes.
    fn arb_opt_ascii()(
        present in any::<bool>(),
        s in "[a-zA-Z0-9 @.+]{1,20}",
    ) -> Option<String> {
        if present { Some(s) } else { None }
    }
}

prop_compose! {
    fn arb_invoice_with_optionals()(
        wallet in arb_wallet_address(),
        client_wallet in prop::option::of(arb_wallet_address()),
        issued_at in 1_600_000_000u32..1_800_000_000u32,
        due_delta in 86400u32..2_592_000u32,
        network_id in prop::sample::select(vec![1u32, 10, 137, 8453, 42161]),
        currency in prop::sample::select(vec!["USDC", "ETH", "USDT", "DAI"]),
        decimals in prop::sample::select(vec![6u8, 18]),
        from_name in "[a-zA-Z ]{1,15}",
        client_name in "[a-zA-Z ]{1,15}",
        item in arb_invoice_item(),
        total in 1u64..1_000_000_000u64,
        salt_bytes in prop::array::uniform16(any::<u8>()),
        email in arb_opt_ascii(),
        notes in arb_opt_ascii(),
        tax in prop::option::of("[0-9]{1,3}"),
        discount in prop::option::of("[0-9]{1,3}"),
    ) -> Invoice {
        use std::fmt::Write as _;
        let salt = salt_bytes.iter().fold(String::with_capacity(32), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        });
        Invoice {
            invoice_id: "INV-G33".to_string(),
            issued_at,
            due_at: issued_at + due_delta,
            network_id,
            currency: currency.to_string(),
            decimals,
            from: InvoiceFrom {
                name: from_name,
                wallet_address: wallet,
                email: email.clone(),
                phone: None,
                physical_address: None,
                tax_id: None,
            },
            client: InvoiceClient {
                name: client_name,
                wallet_address: client_wallet,
                email: None,
                phone: None,
                physical_address: None,
                tax_id: None,
            },
            items: vec![item],
            token_address: None,
            notes,
            tax,
            discount,
            total: total.to_string(),
            salt,
        }
    }
}

proptest! {
    /// G-33: canonical roundtrip with optional fields at controlled probability.
    #[test]
    fn canonical_roundtrip_with_optionals(inv in arb_invoice_with_optionals()) {
        use void_layer_codec::{decode_invoice_canonical, encode_invoice_canonical};
        let bytes = encode_invoice_canonical(&inv).unwrap();
        let decoded = decode_invoice_canonical(&bytes).unwrap();
        prop_assert_eq!(inv, decoded);
    }
}
