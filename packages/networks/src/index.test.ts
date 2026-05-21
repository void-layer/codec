import { describe, it, expect } from 'vitest';
import { SUPPORTED_CHAINS, SUPPORTED_TOKENS, getPublicRpcUrl } from './index.js';

describe('SUPPORTED_CHAINS', () => {
  const CHAIN_IDS = [1, 8453, 42161, 10, 137] as const;

  it('has exactly 5 supported chains', () => {
    expect(Object.keys(SUPPORTED_CHAINS)).toHaveLength(5);
  });

  it.each(CHAIN_IDS)('chain %i has required shape fields', (id) => {
    const chain = SUPPORTED_CHAINS[id];
    expect(chain).toBeDefined();
    expect(typeof chain.name).toBe('string');
    expect(chain.name.length).toBeGreaterThan(0);
    expect(Array.isArray(chain.rpcUrls)).toBe(true);
    expect(typeof chain.blockExplorer).toBe('string');
    expect(typeof chain.nativeCurrency.symbol).toBe('string');
    expect(chain.nativeCurrency.decimals).toBe(18);
  });
});

describe('SUPPORTED_TOKENS', () => {
  it('is an array', () => {
    expect(Array.isArray(SUPPORTED_TOKENS)).toBe(true);
  });

  it('is empty at 0.1.0 (@alpha stub)', () => {
    expect(SUPPORTED_TOKENS).toHaveLength(0);
  });
});

describe('getPublicRpcUrl', () => {
  it.each([1, 8453, 42161, 10, 137] as const)(
    'returns a non-empty URL for chainId %i',
    (id) => {
      const url = getPublicRpcUrl(id);
      expect(typeof url).toBe('string');
      expect(url.length).toBeGreaterThan(0);
      expect(url).toMatch(/^https?:\/\//);
    },
  );

  it('throws for unknown chainId (numeric cast)', () => {
    // Cast through unknown to simulate a caller passing an unsupported id
    expect(() => getPublicRpcUrl(999 as Parameters<typeof getPublicRpcUrl>[0])).toThrow(
      'Unsupported chainId',
    );
  });

  it('throws for unknown chainId (zero)', () => {
    expect(() => getPublicRpcUrl(0 as Parameters<typeof getPublicRpcUrl>[0])).toThrow(
      'Unsupported chainId',
    );
  });
});
