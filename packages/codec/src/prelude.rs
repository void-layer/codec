//! Convenience re-exports of the canonical public API.
//!
//! `use void_layer_codec::prelude::*;` brings the codec entry points and
//! types into scope.

pub use crate::decode::decode_invoice_canonical;
pub use crate::encode::encode_invoice_canonical;
pub use crate::error::CodecError;
pub use crate::hash::compute_content_hash;
pub use crate::invoice::{Invoice, InvoiceClient, InvoiceFrom, InvoiceItem};
