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

## Constitution VI

RPC keys are server-side only. `@void-layer/*` packages NEVER contain RPC keys or PII.

## Provenance

All releases from Phase 3+ ship with npm Provenance attestations via Trusted Publishing.
