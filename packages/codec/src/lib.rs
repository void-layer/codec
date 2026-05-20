//! @void-layer/codec — Phase 1 scaffolding. Real impl lands Phase 2.
//! See spec 056 in voidpay-ai for full design.

pub mod error;
pub use error::CodecError;

pub(crate) mod varint;

pub fn hello() -> &'static str {
    "void-layer-codec phase 1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_works() {
        assert_eq!(hello(), "void-layer-codec phase 1");
    }
}
