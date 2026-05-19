# @void-layer/types

Manual TypeScript types for the `@void-layer` ecosystem. Zero runtime dependencies.

## Install

```sh
pnpm add @void-layer/types
```

## Contents

| Module | Exports |
|--------|---------|
| `network` | `ChainId`, `NetworkConfig` |
| `x402` | `PaymentProof`, `PaymentRequiredResponse` |
| `frame` | `FrameContext`, `FrameState` |

## Usage

```ts
import type { ChainId, NetworkConfig } from '@void-layer/types';
import type { PaymentProof } from '@void-layer/types';
import type { FrameContext, FrameState } from '@void-layer/types';
```

## Notes

- Types only — zero runtime code, zero `const`, zero functions
- No dependencies
- Part of the `@void-layer/codec` monorepo — see [spec 056](https://github.com/ignromanov/voidpay-ai) for design rationale

## License

MIT
