// Dead-code lint suppressed: pub(crate) statics consumed by encode/decode in Phase 2B;
// #[expect] incompatible with inline-test target (lint never fires on test binary → unfulfilled_lint_expectations).
#![allow(dead_code)]

use phf::phf_map;

/// Chain ID dictionary — maps known EVM chain IDs to 1-byte dict codes.
///
/// Encoding scheme (mirror of TS chain-dict.ts):
///   0x00 <code>   — known chain (dict lookup, 2 bytes total)
///   0x01 <varint> — unknown chain (raw varint, 2+ bytes total)
///
/// This map is append-only forever (Constitution IV).
pub(crate) static CHAIN_DICT: phf::Map<u32, u8> = phf_map! {
    1u32     => 0x01u8, // Ethereum
    42161u32 => 0x02u8, // Arbitrum
    10u32    => 0x03u8, // Optimism
    137u32   => 0x04u8, // Polygon
    8453u32  => 0x05u8, // Base
};
