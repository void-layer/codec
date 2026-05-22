# @void-layer/codec

Canonical Invoice codec — TLV + Brotli wire format. v1 schema LOCKED (old invoice URLs decode forever).

## Install

```bash
npm install @void-layer/codec brotli-wasm
```

`brotli-wasm` is a required peer dependency (handles Brotli compression in the JS layer).

## API

### Wire format (async — includes Brotli compression)

```ts
import { encodeInvoiceWire, decodeInvoiceWire } from '@void-layer/codec';

// Invoice → compressed wire bytes (Brotli; falls back to canonical if Brotli expands)
const bytes: Uint8Array = await encodeInvoiceWire(invoice);

// Wire bytes → Invoice (handles both compressed and uncompressed)
const invoice: Invoice = await decodeInvoiceWire(bytes);
```

### Canonical TLV (sync — no compression)

```ts
import { encodeInvoiceCanonical, decodeInvoiceCanonical } from '@void-layer/codec';

// Invoice → canonical TLV bytes (pre-compression, used for payment identity)
const canonical: Uint8Array = encodeInvoiceCanonical(invoice);

// Canonical bytes → Invoice
const invoice: Invoice = decodeInvoiceCanonical(canonical);
```

### Content hash (ERC-3009 nonce)

```ts
import { receiptHash } from '@void-layer/codec';

// keccak-256 of canonical bytes — 32-byte Uint8Array
const hash: Uint8Array = receiptHash(canonical);
```

## Wire format

```
[MAGIC 0x56][VERSION | COMPRESSED_FLAG][brotli([COUNT][TLV records...])]
```

- `COMPRESSED_FLAG = 0x80` — set when body is Brotli-compressed
- Falls back to uncompressed canonical bytes when Brotli would expand the payload
- v1 schema: LOCKED. Old invoice URLs decode forever.

## Packages

| Package | Description |
|---------|-------------|
| `@void-layer/codec` | This package — Rust/WASM codec |
| `@void-layer/types` | TypeScript types (`Invoice`, `InvoiceFrom`, `InvoiceClient`, `InvoiceItem`) |
| `@void-layer/networks` | Chain configs (5 EVM chains) |

## Links

- [TLV Registry](./REGISTRY.md)
- [Bundle Budget](./docs/bundle-budget.md)

## License

MIT
