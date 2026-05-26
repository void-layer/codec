# TLV Type Range Registry

> **Status**: LOCKED at npm 0.1.0 publish (~Jun 1 2026).
> **Decision**: [codec-bolt12-type-range-experimental](../../../../.ai/ops/decisions/2026-05-26-codec-bolt12-type-range-experimental.md)
> **Companion**: [codec-bolt12-odd-even-forward-compat](../../../../.ai/ops/decisions/2026-05-26-codec-bolt12-odd-even-forward-compat.md)

## Type Range Partition (codec v1, u8 namespace)

| Range | Reserved for | Allocator | Stability |
|-------|--------------|-----------|-----------|
| 0 | (forbidden — collides with magic / structural) | — | reserved |
| 1–127 | Spec-allocated fields | `@void-layer/codec` maintainers via void-layer/codec repo PR | LOCKED at allocation |
| 128–255 | Experimental / vendor / third-party extensions | adopter; no central registry | not stable |

## Odd/Even Parity Rule

Parity semantics apply across **both** ranges:

| Tag parity | Meaning | Decoder behavior |
|------------|---------|-----------------|
| **Odd** (bit 0 = 1) | Optional extension | MUST ignore if unknown |
| **Even** (bit 0 = 0) | Mandatory schema change | MUST reject if unknown (`UnknownExtension`) |

This is the BOLT-12 "It's OK to be odd" rule, adopted verbatim.
See [BOLT 01 §"It's OK to be odd"](https://github.com/lightning/bolts/blob/master/01-messaging.md).

## Parity + Range Interaction

| Range | Even tag | Odd tag |
|-------|----------|---------|
| 1–127 (spec) | Mandatory spec field — existing decoder MUST reject | Optional spec field — existing decoder MUST ignore |
| 128–255 (experimental) | Experimental mandatory — decoder MUST reject (third-party asking for hard-fail in unaware decoders) | Experimental optional — decoder MUST ignore |

## Allocation Process

**Spec range (1–127)**: Open a PR against `void-layer/codec` with:
- The new TLV constant added to `src/encode/tags.rs`
- An entry in `KNOWN_TAGS`
- A golden vector in `vectors/v4-codec.json`
- An update to `REGISTRY.md`

Odd-numbered tags for optional fields; even-numbered for fields that require all decoders to upgrade before the wire can be used.

**Experimental range (128–255)**: No PR required. Allocate freely within your adopter namespace. Collisions between independent adopters are possible — this is documented behavior, not a bug. If your extension needs cross-adopter interop, promote it to the spec range via PR.

## Currently Allocated Tags (spec range)

| Tag | Parity | Field | Required |
|-----|--------|-------|---------|
| 1 | odd | `token_address` | no |
| 2 | even | `chain_id` | yes |
| 3 | odd | `client_wallet` | no |
| 4 | even | `issued_at` | yes |
| 5 | odd | `notes` | no |
| 6 | even | `due_at` | yes |
| 7 | odd | `from_email` | no |
| 8 | even | `decimals` | yes |
| 9 | odd | `from_phone` | no |
| 10 | even | `from_wallet` | yes |
| 11 | odd | `from_address` | no |
| 12 | even | `currency` | yes |
| 13 | odd | `client_email` | no |
| 14 | even | `items` | yes |
| 15 | odd | `client_phone` | no |
| 16 | even | `from_name` | yes |
| 17 | odd | `client_address` | no |
| 18 | even | `client_name` | yes |
| 19 | odd | `tax` | no |
| 20 | even | `salt` | yes |
| 21 | odd | `discount` | no |
| 22 | even | `invoice_id` | yes |
| 24 | even | `total` | yes |
| 31 | odd | `domain_separator` | yes (special) |
| 35 | odd | `from_tax_id` | no |
| 37 | odd | `client_tax_id` | no |

Next spec-allocated tags: odd 39+ for optional fields, even 26+ for mandatory fields.

## Cross-references

- Spec 067 TLV registry public-governance (GH AI#117) — the 1–127 spec range is the registry surface
- Decision: [codec-bolt12-type-range-experimental](../../../../.ai/ops/decisions/2026-05-26-codec-bolt12-type-range-experimental.md)
- Decision: [codec-bolt12-odd-even-forward-compat](../../../../.ai/ops/decisions/2026-05-26-codec-bolt12-odd-even-forward-compat.md)
- Decision: [codec-bolt12-strict-monotone-decode](../../../../.ai/ops/decisions/2026-05-26-codec-bolt12-strict-monotone-decode.md)
- Allocation tracking: `REGISTRY.md` (per-tag changelog), `src/encode/tags.rs` (source of truth)
