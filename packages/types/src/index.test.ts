import { describe, it, expectTypeOf } from 'vitest';
import type { Invoice, InvoiceItem, InvoiceParty, ChainId, NetworkConfig } from './index.js';

/**
 * Type-level tests for @void-layer/types.
 * These validate the shape of exported types at compile time via expectTypeOf.
 * No runtime values are exported from this package, so there is nothing to
 * unit-test at runtime beyond confirming the module imports without error.
 */

describe('@void-layer/types — type shapes', () => {
  it('Invoice has required fields with correct types', () => {
    expectTypeOf<Invoice['invoice_id']>().toBeString();
    expectTypeOf<Invoice['issued_at']>().toBeNumber();
    expectTypeOf<Invoice['due_at']>().toBeNumber();
    expectTypeOf<Invoice['currency']>().toBeString();
    expectTypeOf<Invoice['decimals']>().toBeNumber();
    expectTypeOf<Invoice['total']>().toBeString();
    expectTypeOf<Invoice['salt']>().toBeString();
    expectTypeOf<Invoice['items']>().toEqualTypeOf<InvoiceItem[]>();
  });

  it('Invoice has optional fields typed correctly', () => {
    expectTypeOf<Invoice['notes']>().toEqualTypeOf<string | undefined>();
    expectTypeOf<Invoice['tax']>().toEqualTypeOf<string | undefined>();
    expectTypeOf<Invoice['discount']>().toEqualTypeOf<string | undefined>();
  });

  it('InvoiceParty has name required and optional fields', () => {
    expectTypeOf<InvoiceParty['name']>().toBeString();
    expectTypeOf<InvoiceParty['wallet_address']>().toEqualTypeOf<string | undefined>();
    expectTypeOf<InvoiceParty['email']>().toEqualTypeOf<string | undefined>();
  });

  it('InvoiceItem has correct field types', () => {
    expectTypeOf<InvoiceItem['description']>().toBeString();
    expectTypeOf<InvoiceItem['quantity']>().toBeNumber();
    expectTypeOf<InvoiceItem['rate']>().toBeString();
  });

  it('ChainId is a union of supported chain numbers', () => {
    expectTypeOf<ChainId>().toEqualTypeOf<1 | 10 | 137 | 8453 | 42161>();
  });

  it('NetworkConfig has required fields', () => {
    expectTypeOf<NetworkConfig['chainId']>().toEqualTypeOf<ChainId>();
    expectTypeOf<NetworkConfig['name']>().toBeString();
    expectTypeOf<NetworkConfig['blockExplorer']>().toBeString();
  });
});
