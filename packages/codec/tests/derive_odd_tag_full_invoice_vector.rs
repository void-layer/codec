// Vector derivation complete — this file is a stub kept because the sandbox
// cannot delete files. The actual golden vector test lives in parity_malformed.rs
// (parity_y1_odd_tag_in_full_invoice_decodes_successfully).

#![cfg(not(target_arch = "wasm32"))]

mod common;
use common::from_hex;

#[test]
fn derive_odd_tag_full_invoice_vector() {
    use void_layer_codec::decode_invoice_canonical;
    // Verify the derived canonical_hex decodes successfully (smoke check).
    let canonical_hex = "56010e0202000104046553f100060380a3050801060a14d8da6bf26964af9d7eed9e03e53415d37aa960450c0200010e10010a436f6e73756c74696e67000101061005416c6963651203426f621410deadbeefdeadbeefdeadbeefdeadbeef1607494e562d303031180201061f2037e4c5c28ca49a1e1c851f159401a9a27812e1ecca7a63d4d4d0046dfe14b0652702dead";
    let bytes = from_hex(canonical_hex);
    let invoice = decode_invoice_canonical(&bytes)
        .expect("decode_unknown_odd_tag_in_full_invoice must decode under Y1");
    assert_eq!(invoice.invoice_id, "INV-001");
}
