use thiserror::Error;

/// Errors produced by the codec. Never panics on user input.
#[derive(Debug, Error)]
pub enum CodecError {
    #[error("varint overflow at offset {0}")]
    VarintOverflow(usize),
    #[error("truncated payload: needed {needed} bytes, had {had}")]
    Truncated { needed: usize, had: usize },
    #[error("unknown extension TLV type {0}")]
    UnknownExtension(u8),
    #[error("dictionary mismatch: expected {expected}, actual {actual}")]
    DictionaryMismatch { expected: u8, actual: u8 },
    #[error("signature invalid")]
    SignatureInvalid,
    #[error("unsupported version {0}")]
    UnsupportedVersion(u8),
    #[error("bad magic bytes")]
    BadMagic,
    #[error("checksum mismatch")]
    ChecksumMismatch,
    #[error("compression failed: {0}")]
    CompressionFailed(String),
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
}
