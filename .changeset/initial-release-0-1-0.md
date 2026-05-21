---
"@void-layer/codec": minor
"@void-layer/types": minor
"@void-layer/networks": minor
---

Initial 0.1.0 release of the @void-layer monorepo.

- `@void-layer/codec`: Canonical TLV + Brotli wire codec (WASM + JS shim). Includes `encodeInvoiceCanonical`, `decodeInvoiceCanonical`, `encodeInvoiceWire`, `decodeInvoiceWire`, and `receiptHash` (keccak-256 content hash). 18 golden vectors in v4-codec.json schema_version=1.
- `@void-layer/types`: TypeScript type definitions for Invoice, InvoiceItem, InvoiceParty, NetworkConfig, ChainId, FrameContext, FrameState, PaymentProof, PaymentRequiredResponse. Zero runtime dependencies.
- `@void-layer/networks`: Chain configs for 5 EVM networks (Ethereum, Base, Arbitrum, Optimism, Polygon) with public RPC URLs. `SUPPORTED_TOKENS` is empty at 0.1.0 (@alpha — populated in a future release from Uniswap Token List).
