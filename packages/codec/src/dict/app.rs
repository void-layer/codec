/// Application-level text dictionary — pre-Brotli substitution for common patterns.
///
/// Maps string pattern → 1-byte control code (0x02–0x1F range).
/// This `phf_map!` iterates in hash-order; the runtime codec uses the
/// length-ordered `encode::dict::APP_DICT_ENTRIES` slice for longest-match.
/// This map is the canonical reference the dict-lock test validates against
/// (test-only — gated `#[cfg(test)]` since the codec path uses the slice).
/// The dictionary is append-only forever (Constitution IV).
#[cfg(test)]
use phf::phf_map;

#[cfg(test)]
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
