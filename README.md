# @void-layer/codec

Canonical Invoice codec — TLV + Brotli wire format. v1 schema LOCKED forever.

[![npm](https://img.shields.io/npm/v/@void-layer/codec.svg)](https://npmjs.com/package/@void-layer/codec) [![CI](https://github.com/void-layer/codec/actions/workflows/ci.yml/badge.svg)](https://github.com/void-layer/codec/actions) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Status

🚧 Phase 1 scaffolding (May 2026) — Rust impl lands Phase 2

## Packages

| Package | Status | Description |
|---------|--------|-------------|
| `@void-layer/codec` | Phase 1 | Rust + WASM canonical TLV codec |
| `@void-layer/types` | Phase 1 | Manual TypeScript types (zero runtime deps) |
| `@void-layer/networks` | Phase 1 | EVM chain configs + token list (no RPC keys) |

## Quick Install

```bash
pnpm add @void-layer/codec
```

> Not yet published — Phase 3

## Why

- Third-party developers building on top of VoidPay need a stable, versioned codec they can depend on
- MCP servers, Farcaster Frames, and AI agents all depend on a common wire format — language-agnostic TLV is the right primitive
- Version-controlled schema means consumers can pin to v1 and get backward-compat guarantees forever
- Language-agnostic TLV encoding allows Rust, Go, Python, and JS implementations to interoperate on the same wire format

## Constitution IV — Perpetual

> Schema v1 LOCKED. Old URLs decode forever.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md)

## Security

See [SECURITY.md](SECURITY.md)

## Architecture

See [docs/architecture-overview.md](docs/architecture-overview.md)

## Spec

Full design: [spec 056 in voidpay-ai](https://github.com/ignromanov/voidpay-ai/blob/master/ops/specs/056-void-layer-codec-extraction/spec.md) (private — internal reference)

---

Built by [Ignat Romanov](https://github.com/ignromanov) · MIT License
