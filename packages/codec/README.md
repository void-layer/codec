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
import { encodeInvoiceCanonical, receiptHash } from '@void-layer/codec';

// ALWAYS hash the output of encodeInvoiceCanonical — never received bytes.
const canonical: Uint8Array = encodeInvoiceCanonical(invoice);
const hash: Uint8Array = receiptHash(canonical); // 32-byte Uint8Array
```

> [!IMPORTANT]
> **`receiptHash` accepts arbitrary bytes.** Pass only the output of `encodeInvoiceCanonical(invoice)`. If you have received bytes (from a URL), decode them and re-encode before hashing — never hash received bytes directly. The ERC-3009 nonce contract requires the hash over the canonical form; hashing received bytes makes the nonce dependent on the producer's encoder rather than the canonical form. A type-safe `receiptHash(invoice: Invoice)` surface is on the v0.2 roadmap.

## Wire format

```
[MAGIC 0x56][VERSION | COMPRESSED_FLAG][brotli([COUNT][TLV records...])]
```

- `COMPRESSED_FLAG = 0x80` — set when body is Brotli-compressed
- Falls back to uncompressed canonical bytes when Brotli would expand the payload
- v1 schema: LOCKED. Old invoice URLs decode forever.

## Decoder invariants

The v1 decoder is **fail-loud**: any `Ok(Invoice)` means every byte was read with exactly one interpretation. The following classes of input are rejected to prevent semantic divergence between readers (different readers extracting different invoices from the same accepted bytes would produce different `keccak256(canonical)` → different ERC-3009 nonces):

| Reject | Error variant |
|--------|---------------|
| Duplicate TLV tag | `InvalidData("duplicate TLV tag")` |
| Unknown TLV tag (tag ∉ v1 set of 26) | `UnknownExtension(tag)` |
| Non-canonical LEB128 varint (redundant trailing zero group) | `InvalidData("non-canonical varint")` |
| Salt length ≠ 16 bytes | `ChecksumMismatch` |
| TLV value > 4096 bytes · TLV count > 64 · varint > 37 bytes | `Truncated` / `VarintOverflow` |
| Raw-form encoding of a dict-known chain ID (non-canonical) | `InvalidData("non-canonical chain encoding: …")` |
| Raw-form encoding of a dict-known currency symbol (non-canonical) | `InvalidData("non-canonical currency encoding: …")` |
| Unknown prefix byte (≠ 0x00/0x01) on currency or token-address TLV | `UnknownExtension(prefix)` |
| `TLV_DECIMALS` value length ≠ 1 byte | `InvalidData("non-canonical TLV_DECIMALS length: …")` |
| Per-item quantity scale > 9 (non-canonical; encoder cap is 9) | `InvalidData("non-canonical quantity scale …")` |

<details>
<summary><b>Full <code>CodecError</code> variants</b></summary>

| Variant | Trigger |
|---------|---------|
| `BadMagic` | First byte is not `0x56` |
| `UnsupportedVersion` | Version byte signals an unknown codec version |
| `Truncated { needed, had }` | Buffer ends before a TLV value is fully read |
| `VarintOverflow` | LEB128 continuation bytes exceed `MAX_BYTES = 37` |
| `InvalidData(msg)` | Invalid UTF-8, duplicate TLV tag, non-canonical varint, decode of canonical input with the compressed flag set, etc. |
| `UnknownExtension(tag)` | Unknown TLV tag in a v1 payload, or unknown dict code for chain/currency/token |
| `ChecksumMismatch` | Domain separator validation failed, or salt length ≠ 16 |
| `CompressionFailed` | Brotli decompression error on a wire payload |
| `DictionaryMismatch` | Dict hash in payload does not match compiled dict |
| `InvalidAmount` | Amount string exceeds `U256::MAX`, is not a valid decimal, or `mantissa × 10^zeros` overflows U256 |

The 280-character notes limit is **not** enforced by the codec — it is an application-layer concern. The reference voidpay.xyz implementation validates in Unicode code points before encode; platforms adopting `@void-layer/codec` must apply equivalent validation.

</details>

See [docs/architecture-overview.md](../../docs/architecture-overview.md) for a Mermaid decode-flow diagram and rationale; [docs/architecture.canvas](../../docs/architecture.canvas) for an Obsidian Canvas view of the same.

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
