# @void-layer/networks

Chain configs + token list for the `@void-layer` ecosystem.

## Install

```bash
pnpm add @void-layer/networks
```

## Usage

```typescript
import { SUPPORTED_CHAINS, getPublicRpcUrl } from '@void-layer/networks';

const eth = SUPPORTED_CHAINS[1];
// { chainId: 1, name: 'Ethereum', rpcUrls: [...], ... }

const url = getPublicRpcUrl(1);
// 'https://eth.llamarpc.com'
```

## Privacy note

**NO RPC KEYS in this package.** All URLs are public endpoints (llamarpc.com).
Server-side API keys (Alchemy, Infura, etc.) live in `voidpay.xyz` only — never shipped in client bundles.

`SUPPORTED_TOKENS` is empty in Phase 1. Phase 2 populates from Uniswap Token List.

## Supported chains

Ethereum (1), Base (8453), Arbitrum One (42161), Optimism (10), Polygon (137).

## Reference

Design: [spec 056](https://github.com/ignromanov/voidpay-ai/tree/main/ops/specs/056-void-layer-codec-extraction)
