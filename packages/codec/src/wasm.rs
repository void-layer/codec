//! WASM bindings — compiled only for `target_arch = "wasm32"`.
//!
//! Exports exactly 2 functions to JS per B-v replan (2026-05-20):
//!   - `encodeInvoiceCanonical` — TLV canonical bytes (no Brotli)
//!   - `decodeInvoiceCanonical` — Invoice from canonical bytes
//!
//! Wire encoding (Brotli + COMPRESSED_FLAG) lives in the JS shim
//! (`src/index.ts`) which wraps these and calls `brotli-wasm` as peerDep.

#![cfg(target_arch = "wasm32")]

use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

use crate::{decode_invoice_canonical, encode_invoice_canonical, Invoice};

/// BigInt-safe serializer: amounts like `u64::MAX` come back as JS BigInt, not lossy f64.
/// Required per D-B11 (BigInt boundary discipline).
fn ts_serializer() -> Serializer {
    Serializer::new().serialize_large_number_types_as_bigints(true)
}

/// Encode an Invoice to canonical TLV bytes (pre-compression, payment identity).
///
/// The COMPRESSED_FLAG (0x80) is never set on the output — Brotli compression
/// is the caller's responsibility via the JS shim and `brotli-wasm` peerDep.
///
/// Feed the output to `compute_content_hash()` for ERC-3009 nonce binding.
#[wasm_bindgen(js_name = encodeInvoiceCanonical)]
pub fn encode_invoice_canonical_js(invoice: JsValue) -> Result<Vec<u8>, JsError> {
    let invoice: Invoice = serde_wasm_bindgen::from_value(invoice)
        .map_err(|e| JsError::new(&e.to_string()))?;
    encode_invoice_canonical(&invoice).map_err(|e| JsError::new(&e.to_string()))
}

/// Decode canonical TLV bytes into an Invoice object.
///
/// Input must NOT have the COMPRESSED_FLAG set — decompress first via the JS shim.
#[wasm_bindgen(js_name = decodeInvoiceCanonical)]
pub fn decode_invoice_canonical_js(bytes: &[u8]) -> Result<JsValue, JsError> {
    let invoice = decode_invoice_canonical(bytes).map_err(|e| JsError::new(&e.to_string()))?;
    invoice
        .serialize(&ts_serializer())
        .map_err(|e| JsError::new(&e.to_string()))
}

/// dlmalloc allocator — ~5 KB overhead, replaces the default (wee_alloc is forbidden per §3.8).
#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
