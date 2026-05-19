# @void-layer Architecture Overview

## Monorepo Structure

```
packages/
├─ codec/        # @void-layer/codec — Rust + WASM canonical TLV codec
├─ types/        # @void-layer/types — manual TS types (zero runtime deps)
└─ networks/     # @void-layer/networks — chain configs + token list (no RPC keys)
```

## Dependency Rules (Immutable)

- `@void-layer/codec` depends on: **nothing** (pure Rust + auto-gen TS bindings)
- `@void-layer/types` depends on: **nothing** (pure TS, no runtime deps)
- `@void-layer/networks` depends on: `@void-layer/types` only
- Downstream packages (agent, merchant, frame) depend on codec + types + networks
- Auto-generated types from `wasm-bindgen` + `tsify` live in `@void-layer/codec/types` subpath export — NOT in `@void-layer/types`

## Build Pipeline (Phase 2+)

```
src/*.rs → cargo + wasm-pack → pkg/
                              ├─ codec.js (ESM)
                              ├─ codec.d.ts (auto-gen TS bindings via tsify)
                              └─ codec_bg.wasm

CJS wrapper hand-authored: cjs/index.js (await init() guard)
```

## Schema Versioning

- **v1 LOCKED** (Constitution IV). Old URLs decode forever.
- **v2 additive** via TLV odd/even rule + `extensions` map (BOLT12 import).
- **Receipt-hash**: `keccak256(canonical_binary_PRE_compression)` (algo-agnostic).

## Compression

- **Wire format v1**: Brotli q11 whole-payload, signaled by `VERSION & 0x80` (LOCKED).
- **v2 runtime branch** (B-iv per spec §3.16):
  1. `'brotli' in CompressionStream.supportedFormats` → native (zero bundle cost)
  2. Else → `brotli-wasm` peerDep fallback (current shipping pattern)

## Encoding

- URL hash fragment: `base64url` (LOCKED v1; default v2)
- QR alphanumeric: `Crockford32` (v1.3+, gated on >15% QR share analytics)
- EVM calldata: `hex`
- Solana account data: `base58`

## Hard Limits

- WASM blob: <80 KB
- npm package total: <200 KB
- URL max: 2000 bytes compressed
- Notes max: 280 chars

## References

- Full spec: `voidpay-ai/ops/specs/056-void-layer-codec-extraction/spec.md`
- ADR-supersession: `voidpay-ai/agent-memory/advisors/decisions/2026-05-09-kai-cto-codec-rust-supersedes-ts-first.md`
- Constitution: VoidPay Principle IV (Perpetual + Schema versioning)
- TLV Registry: [`packages/codec/REGISTRY.md`](../packages/codec/REGISTRY.md)
- TLV contribution guide: [`contributing-tlv-registry.md`](./contributing-tlv-registry.md)
