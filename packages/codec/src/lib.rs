//! @void-layer/codec — canonical Invoice codec.
//!
//! TLV wire format + keccak256 content hash. Brotli compression lives
//! in the JS shim layer (`src/index.ts`) over the `brotli-wasm` peerDep
//! per B-v replan (2026-05-20).
//!
//! # Public API (B-v — canonical only in Rust)
//!
//! ```text
//! encode_invoice_canonical  → canonical TLV bytes (pre-compression, payment identity)
//! decode_invoice_canonical  → Invoice from canonical bytes
//! compute_content_hash      → keccak256 of canonical bytes (ERC-3009 nonce)
//! ```
//!
//! Wire encoding (Brotli + COMPRESSED_FLAG) is provided by the JS shim
//! (`encodeInvoiceWire` / `decodeInvoiceWire`) which wraps these fns and
//! calls `brotli-wasm` as a peerDep.
//!
//! See spec 056 in voidpay-ai for full design.

pub mod error;
pub mod invoice;

pub(crate) mod varint;
pub(crate) mod tlv;
pub(crate) mod dict;
pub(crate) mod hash;
pub(crate) mod encode;
pub(crate) mod decode;

#[cfg(target_arch = "wasm32")]
mod wasm;

// --- Canonical public surface ---
pub use error::CodecError;
pub use invoice::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};
pub use encode::encode_invoice_canonical;
pub use decode::decode_invoice_canonical;
pub use hash::compute_content_hash;
