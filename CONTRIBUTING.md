# Contributing to @void-layer/codec

## Dev Setup

```bash
# Requires Node >=24 and pnpm >=11
pnpm install
```

### Rust / WASM toolchain

- The Rust toolchain is pinned by `packages/codec/rust-toolchain.toml` (1.85.0, `wasm32-unknown-unknown` target); rustup picks it up automatically inside `packages/codec`.
- Run cargo commands from `packages/codec` (not the repo root with `--manifest-path`): any rustup directory override on a parent directory, as set by `setup-rust-toolchain` on CI, bypasses the pin. CI does the same.
- CI installs wasm-pack 0.14.0 (`.github/workflows/ci.yml`). Building wasm-pack 0.14.0 from source needs rustc >= 1.88, so install it with an up-to-date stable toolchain: `rustup update stable && cargo +stable install wasm-pack --version 0.14.0 --locked`.
- The gated WASM size depends on the wasm-pack and wasm-opt (binaryen) versions; wasm-pack uses a wasm-opt found on PATH if present. Treat CI's `scripts/assert-size.sh` output as authoritative.

## Making Changes

Build all packages:
```bash
pnpm build
```

Run tests:
```bash
pnpm test
```

Run lint:
```bash
pnpm lint
```

## Releasing

This repo uses [Changesets](https://github.com/changesets/changesets) for versioning.

Add a changeset for any user-facing change:
```bash
pnpm changeset
```

Then commit the generated `.changeset/*.md` file with your PR.

Maintainers run `pnpm version` to bump versions and `pnpm release` to publish.

## Design Rationale

See [docs/architecture-overview.md](docs/architecture-overview.md).

The decision to rewrite the codec from TypeScript to Rust+WASM is documented in:
`voidpay-ai/agent-memory/advisors/decisions/2026-05-09-kai-cto-codec-rust-supersedes-ts-first.md`

## Schema

v1 schema is LOCKED. Old invoice URLs must decode forever. Never break existing field assignments.
