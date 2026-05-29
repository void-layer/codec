# @void-layer/types

Manual TypeScript types for the `@void-layer` ecosystem. Zero runtime dependencies.

## Install

```sh
pnpm add @void-layer/types
```

## Contents

| Module | Exports |
|--------|---------|
| `invoice` | `Invoice`, `InvoiceFrom`, `InvoiceClient`, `InvoiceItem` |
| `network` | `ChainId`, `NetworkConfig` |
| `x402` | `PaymentProof`, `PaymentRequiredResponse` |
| `frame` | `FrameContext`, `FrameState` |

## Usage

```ts
import type { Invoice, InvoiceFrom, InvoiceClient, InvoiceItem } from '@void-layer/types';
import type { ChainId, NetworkConfig } from '@void-layer/types';
import type { PaymentProof } from '@void-layer/types';
import type { FrameContext, FrameState } from '@void-layer/types';
```

### Invoice types example

```ts
import type { Invoice, InvoiceFrom, InvoiceClient, InvoiceItem } from '@void-layer/types';

const from: InvoiceFrom = {
  name: 'Acme Corp',
  wallet_address: '0xabc...',
  email: 'billing@acme.com',
};

const client: InvoiceClient = {
  name: 'Bob',
  wallet_address: '0xdef...',
};

const item: InvoiceItem = {
  description: 'Consulting',
  quantity: 10,
  rate: '150.00',
};

const invoice: Invoice = {
  invoice_id: 'INV-001',
  issued_at: 1716000000,
  due_at: 1718592000,
  network_id: 1,
  currency: 'USDC',
  decimals: 6,
  from,
  client,
  items: [item],
  total: '1500.00',
  salt: 'abc123',
};
```

## Notes

- Types only — zero runtime code, zero `const`, zero functions
- No dependencies
- Part of the `@void-layer/codec` monorepo — see [spec 056](https://github.com/ignromanov/voidpay-ai) for design rationale

## License

MIT
