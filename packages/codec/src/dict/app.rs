// Dead-code lint suppressed: pub(crate) statics consumed by encode/decode in Phase 2B;
// #[expect] incompatible with inline-test target (lint never fires on test binary → unfulfilled_lint_expectations).
#![allow(dead_code)]

use phf::phf_map;

/// Application-level text dictionary — pre-Brotli substitution for common patterns.
///
/// Maps string pattern → 1-byte control code (0x02–0x1F range).
/// Entries are in length-descending order (longest match first) to avoid partial replacements.
/// This map is append-only forever (Constitution IV).
pub(crate) static APP_DICT: phf::Map<&'static str, u8> = phf_map! {
    "@outlook.com" => 0x02u8,
    "@hotmail.com" => 0x0cu8,
    "development"  => 0x0du8,
    "consulting"   => 0x0eu8,
    "@gmail.com"   => 0x03u8,
    "@yahoo.com"   => 0x04u8,
    "https://"     => 0x05u8,
    "Invoice"      => 0x06u8,
    "Payment"      => 0x07u8,
    ".com"         => 0x09u8,
    "INV-"         => 0x0fu8,
};
