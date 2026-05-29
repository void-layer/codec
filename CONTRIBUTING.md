# Contributing to @void-layer/codec

## Dev Setup

```bash
# Requires Node >=24 and pnpm >=10
pnpm install
```

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

See [spec 056](https://github.com/ignromanov/voidpay-ai/blob/master/ops/specs/056-void-layer-codec-extraction/spec.md).

The decision to rewrite the codec from TypeScript to Rust+WASM is documented in:
`voidpay-ai/agent-memory/advisors/decisions/2026-05-09-kai-cto-codec-rust-supersedes-ts-first.md`

## Schema

v1 schema is LOCKED. Old invoice URLs must decode forever. Never break existing field assignments.
