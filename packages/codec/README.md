# @void-layer/codec

> **Status**: Phase 1 scaffolding. Rust + WASM implementation lands Phase 2.

Canonical Invoice codec — TLV + Brotli wire format. v1 schema LOCKED (old URLs decode forever).

## Install

```bash
npm install @void-layer/codec brotli-wasm
```

`brotli-wasm` is a required peer dependency.

## API (Phase 2 placeholder)

```ts
import { encode, decode } from '@void-layer/codec';

// encode: Invoice -> Uint8Array (TLV + Brotli compressed)
const bytes = encode(invoice);

// decode: Uint8Array -> Invoice (version-aware, v1 LOCKED)
const invoice = decode(bytes);
```

Full API defined in spec 056 §3.6. TypeScript bindings auto-generated from Rust via `wasm-bindgen` + `tsify`.

## Packages

| Package | Description |
|---------|-------------|
| `@void-layer/codec` | This package — Rust/WASM codec |
| `@void-layer/types` | Manual TypeScript types |
| `@void-layer/networks` | Chain configs (5 EVM chains) |

## Design

- Wire format: TLV (BOLT12-style) + Brotli compression
- Output: `<2B magic> <1B kind> <varint version> <tlv-stream>`
- v1 schema: LOCKED. Old invoice URLs decode forever.
- peerDep strategy: brotli-wasm (runtime branch, see spec §3.16)

## Links

- [Spec 056](https://github.com/ignromanov/voidpay-ai/blob/main/ops/specs/056-void-layer-codec-extraction/spec.md)
- [TLV Registry](./REGISTRY.md)
- [Bundle Budget](./docs/bundle-budget.md)

## License

MIT
