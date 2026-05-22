//! Codec error type.

use thiserror::Error;

/// Errors produced by the codec. Never panics on user input.
///
/// The `#[error("...")]` display strings are a semver-locked public contract:
/// the TS parity test (`tests/parity.test.ts`) matches error substrings as a
/// stable surface. See `REGISTRY.md` § Breaking-change policy.
#[derive(Debug, Error)]
pub enum CodecError {
    /// A LEB128 varint exceeded the maximum byte budget at the given offset.
    #[error("varint overflow at offset {0}")]
    VarintOverflow(usize),
    /// The payload ended before a required number of bytes could be read.
    #[error("truncated payload: needed {needed} bytes, had {had}")]
    Truncated {
        /// Number of bytes the reader required.
        needed: usize,
        /// Number of bytes actually available.
        had: usize,
    },
    /// An unknown extension TLV type was encountered.
    #[error("unknown extension TLV type {0}")]
    UnknownExtension(u8),
    /// A dictionary code did not match the expected value.
    #[error("dictionary mismatch: expected {expected}, actual {actual}")]
    DictionaryMismatch {
        /// The dictionary code the decoder expected.
        expected: u8,
        /// The dictionary code actually found.
        actual: u8,
    },
    /// A signature failed validation.
    #[error("signature invalid")]
    SignatureInvalid,
    /// The version byte is not a supported codec version.
    #[error("unsupported version {0}")]
    UnsupportedVersion(u8),
    /// The leading magic byte did not match the codec magic.
    #[error("bad magic bytes")]
    BadMagic,
    /// The domain-separator / checksum TLV did not match the computed value.
    #[error("checksum mismatch")]
    ChecksumMismatch,
    /// Brotli compression or decompression failed.
    #[error("compression failed: {0}")]
    CompressionFailed(String),
    /// A monetary amount was malformed or out of the U256 domain.
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    /// An EVM address string was malformed (bad length or non-hex bytes).
    #[error("invalid address: {0}")]
    InvalidAddress(String),
    /// A required TLV field was absent from the canonical payload.
    #[error("missing required TLV field {0}")]
    MissingField(u8),
    /// A structural size or count limit was exceeded.
    #[error("payload overflow: {0}")]
    Overflow(String),
}
