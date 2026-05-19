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

## Constitution VI

RPC keys are server-side only. `@void-layer/*` packages NEVER contain RPC keys or PII.

## Provenance

All releases from Phase 3+ ship with npm Provenance attestations via Trusted Publishing.
