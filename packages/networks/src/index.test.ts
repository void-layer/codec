import { describe, it, expect } from 'vitest';
import { SUPPORTED_CHAINS, CHAINS, getPublicRpcUrl } from './index.js';

describe('CHAINS / SUPPORTED_CHAINS', () => {
  const CHAIN_IDS = [1, 8453, 42161, 10, 137] as const;

  it('has exactly 5 supported chains', () => {
    expect(Object.keys(CHAINS)).toHaveLength(5);
  });

  it('SUPPORTED_CHAINS is an alias for CHAINS', () => {
    expect(SUPPORTED_CHAINS).toBe(CHAINS);
  });

  it.each(CHAIN_IDS)('chain %i has required shape fields', (id) => {
    const chain = CHAINS[id];
    expect(chain).toBeDefined();
    expect(typeof chain.name).toBe('string');
    expect(chain.name.length).toBeGreaterThan(0);
    expect(Array.isArray(chain.rpcUrls)).toBe(true);
    expect(typeof chain.blockExplorer).toBe('string');
    expect(typeof chain.nativeCurrency.symbol).toBe('string');
    expect(chain.nativeCurrency.decimals).toBe(18);
  });

  it.each(CHAIN_IDS)('chain %i has publicRpcUrls with 2+ entries', (id) => {
    const chain = CHAINS[id];
    expect(Array.isArray(chain.publicRpcUrls)).toBe(true);
    expect(chain.publicRpcUrls.length).toBeGreaterThanOrEqual(2);
    for (const url of chain.publicRpcUrls) {
      expect(url).toMatch(/^https?:\/\//);
    }
  });

  it.each(CHAIN_IDS)('chain %i publicRpcUrls contains no API keys', (id) => {
    const chain = CHAINS[id];
    for (const url of chain.publicRpcUrls) {
      expect(url).not.toMatch(/alchemy|infura|quicknode/i);
    }
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
