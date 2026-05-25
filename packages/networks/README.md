# @void-layer/networks

Chain configs, token list, block explorer helpers, and wagmi chain configs for the `@void-layer` ecosystem.

**Constitution VI compliant** — NO API keys, NO paid RPCs. All URLs are public endpoints only.

## Install

```bash
pnpm add @void-layer/networks
```

For wagmi chain configs, `viem` is a peer dependency:

```bash
pnpm add @void-layer/networks viem
```

## Exports

### Chain config

```typescript
import { CHAINS, getChainConfig, tryGetChainConfig } from '@void-layer/networks';

// Direct lookup
const eth = CHAINS[1];
// { chainId: 1, name: 'Ethereum', publicRpcUrls: [...], blockExplorer: '...', ... }

// Type-safe getter (throws on unknown chain)
const base = getChainConfig(8453);

// Safe getter (returns null on unknown chain)
const unknown = tryGetChainConfig(999); // null
```

### Block explorer URLs

```typescript
import { getExplorerTxUrl, getExplorerAddressUrl } from '@void-layer/networks';
// or from subpath:
import { getExplorerTxUrl } from '@void-layer/networks/explorer';

const txUrl = getExplorerTxUrl(1, '0xabc...');
// 'https://etherscan.io/tx/0xabc...'

const addrUrl = getExplorerAddressUrl(8453, '0x833...');
// 'https://basescan.org/address/0x833...'
```

### Token list

```typescript
import { TOKENS, getTokenInfo } from '@void-layer/networks';
// or from subpath:
import { getTokenInfo } from '@void-layer/networks/tokens';

const usdc = getTokenInfo(1, '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48');
// { chainId: 1, symbol: 'USDC', decimals: 6, ... }
```

Covers all 30 (chainId, address) pairs the codec wire-format dict knows about.
Does NOT duplicate codec's wire-format constants — this package owns display metadata only.

### wagmi chain configs

```typescript
import { ethereumWagmi, baseWagmi, ALL_WAGMI_CHAINS } from '@void-layer/networks/wagmi';

// Use with wagmi createConfig:
const config = createConfig({ chains: ALL_WAGMI_CHAINS, ... });
```

Available exports: `ethereumWagmi`, `baseWagmi`, `arbitrumWagmi`, `optimismWagmi`, `polygonWagmi`, `ALL_WAGMI_CHAINS`.

### Public RPC URL

```typescript
import { getPublicRpcUrl } from '@void-layer/networks';

const url = getPublicRpcUrl(1);
// 'https://eth.llamarpc.com'
```

Each chain has 2–3 public RPC fallback URLs in `publicRpcUrls` (llamarpc.com, publicnode.com, ankr.com).

## Supported chains

Ethereum (1), Base (8453), Arbitrum One (42161), Optimism (10), Polygon (137).

## Privacy note

**NO RPC KEYS in this package.** All URLs are public endpoints (llamarpc.com, publicnode.com, ankr.com).
Server-side API keys (Alchemy, Infura, etc.) live in `voidpay.xyz` only — never shipped in client bundles.

## Codec/networks separation

`@void-layer/codec` owns wire-format constants (TLV dict codes, chain code ranges — append-only per Constitution IV).
`@void-layer/networks` owns display/runtime metadata (names, explorer URLs, logos, wagmi configs).
These packages are intentionally decoupled: `networks` does NOT import from `codec` at runtime.

## Reference

Design: [spec 056](https://github.com/ignromanov/voidpay-ai/tree/main/ops/specs/056-void-layer-codec-extraction)
