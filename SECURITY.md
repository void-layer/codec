# Security Policy

## Supported Versions

Latest published version on npm only (Phase 3+).

| Version | Supported |
|---------|-----------|
| latest  | ✅        |
| < latest | ❌       |

## Reporting a Vulnerability

**Preferred**: open a private advisory at https://github.com/void-layer/codec/security/advisories/new

**Email fallback**: `ign.romanov@gmail.com` with subject prefix `[security][@void-layer/codec]`

**Response SLA**: 72 hours initial acknowledgment.

## Scope

### In scope

- Codec encoding/decoding correctness
- Schema v1 backward-compatibility violations
- BigInt boundary issues (precision loss, silent truncation)
- WASM initialization security (race conditions, init bypass)
- Wire format determinism (canonical hash drift)

### Out of scope

- VoidPay product application — see [voidpay/SECURITY.md](https://github.com/ignromanov/voidpay/blob/master/SECURITY.md)
- RPC provider issues — those are external infrastructure

## Integrity vs authenticity

The domain separator and content hash (`keccak256` over the canonical TLV bytes) are **integrity** mechanisms. They detect accidental corruption and enforce deterministic field ordering — nothing more.

They are **not** a signature. There is no secret key and no authentication. Any party can construct a fully valid, well-formed invoice URL with arbitrary values for `total`, `wallet_address`, or any other field. A structurally valid invoice is not a trusted or authenticated invoice.

Integrators MUST NOT treat a passing decode or a matching content hash as proof that the invoice was created by a specific party or that its contents are authoritative. In the voidpay.xyz reference implementation the payer reviews the rendered payment card and confirms the details before sending funds. Platforms building on `@void-layer/codec` must apply equivalent confirmation or authentication at their own layer.

## Decoder strictness invariants (v1)

The v1 decoder is **fail-loud**. A successful `Ok(Invoice)` means every byte was read and accounted for, with exactly one interpretation. The codec rejects three classes of input that would otherwise produce *semantic divergence* — different readers extracting different invoices from the same accepted bytes, leading to a different `keccak256(canonical)` → different ERC-3009 nonces → payers authorizing transfers they did not see:

| Reject | Error | Why it's a security invariant |
|--------|-------|------------------------------|
| Duplicate TLV tag | `InvalidData("duplicate TLV tag")` | A `last-write-wins` decoder agrees with a `first-write-wins` decoder only by accident. Without this guard, a producer-crafted duplicate-`TLV_TOTAL` payload could make Rust and TS surfaces read different totals — a fund-loss class. |
| Unknown TLV tag (tag ∉ v1 set of 26) | `UnknownExtension(tag)` | v1 has a closed tag set (Constitution IV — schema LOCKED). An unknown tag in an `Ok(Invoice)` payload would be silently dropped by a v1 reader but read by a v2-or-other-platform reader. The BOLT12 odd/even extensibility mechanism activates only from v2+. |
| Non-canonical LEB128 varint | `InvalidData("non-canonical varint")` | Same value encoded as `0x00` vs `0x80 0x00` must not coexist. Defense-in-depth against producers whose receipt-hash consumer hashes received bytes instead of canonical bytes. |

The domain separator (`keccak256("VOIDPAY_INVOICE_V1" || serialized records)`) covers every TLV in the payload — unknown tags cannot be silently appended past the separator. These invariants are tested by the `malformed-unknown-tlv-tag` and `malformed-duplicate-tlv-tag` golden vectors and locked by the parity suite (Rust ↔ TS).

## receiptHash inputs (footgun advisory)

`receiptHash(canonical_bytes)` is keccak-256 over arbitrary input — it hashes whatever bytes you pass it. The ERC-3009 nonce contract requires the hash over the **canonical** form of the invoice. The current API surface accepts a `Uint8Array` rather than an `Invoice`, so callers are responsible for passing the canonical bytes:

- **ALWAYS**: pass the output of `encodeInvoiceCanonical(invoice)`.
- **NEVER**: hash received bytes directly. If you have received bytes (from a URL), decode them and re-encode before hashing. Even though the v1 decoder now rejects non-canonical varints and duplicate tags (above), hashing received bytes makes the nonce depend on the producer's encoder rather than the canonical form.

A type-safe `receiptHash(invoice: Invoice)` surface that performs the canonical encode internally is on the v0.2 roadmap. Until then, treat the byte-input signature as a layer boundary you own.

## Constitution VI

RPC keys are server-side only. `@void-layer/*` packages NEVER contain RPC keys or PII.

## Provenance

All releases from Phase 3+ ship with npm Provenance attestations via Trusted Publishing.
