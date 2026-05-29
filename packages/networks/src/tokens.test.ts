import { describe, it, expect } from 'vitest';
import { TOKENS, SUPPORTED_TOKENS, getTokenInfo } from './tokens.js';

describe('TOKENS', () => {
  it('SUPPORTED_TOKENS is alias for TOKENS', () => {
    expect(SUPPORTED_TOKENS).toBe(TOKENS);
  });

  it('has 30 entries matching codec dict count', () => {
    expect(TOKENS).toHaveLength(30);
  });

  it('all entries have required fields', () => {
    for (const token of TOKENS) {
      expect(typeof token.chainId).toBe('number');
      expect(typeof token.address).toBe('string');
      expect(token.address).toMatch(/^0x[0-9a-f]{40}$/);
      expect(typeof token.symbol).toBe('string');
      expect(token.symbol.length).toBeGreaterThan(0);
      expect(typeof token.name).toBe('string');
      expect(typeof token.decimals).toBe('number');
    }
  });

  it('all addresses are lowercase', () => {
    for (const token of TOKENS) {
      expect(token.address).toBe(token.address.toLowerCase());
    }
  });

  it('covers all 5 supported chains', () => {
    const chains = new Set(TOKENS.map((t) => t.chainId));
    expect(chains.has(1)).toBe(true);
    expect(chains.has(8453)).toBe(true);
    expect(chains.has(42161)).toBe(true);
    expect(chains.has(10)).toBe(true);
    expect(chains.has(137)).toBe(true);
  });
});

describe('getTokenInfo', () => {
  it('finds Ethereum USDC', () => {
    const token = getTokenInfo(1, '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48');
    expect(token).toBeDefined();
    expect(token!.symbol).toBe('USDC');
    expect(token!.decimals).toBe(6);
    expect(token!.chainId).toBe(1);
  });

  it('finds token by mixed-case address (lowercases input)', () => {
    const token = getTokenInfo(1, '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48');
    expect(token).toBeDefined();
    expect(token!.symbol).toBe('USDC');
  });

  it('finds Base USDC', () => {
    const token = getTokenInfo(8453, '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913');
    expect(token).toBeDefined();
    expect(token!.symbol).toBe('USDC');
  });

  it('finds Optimism WETH', () => {
    const token = getTokenInfo(10, '0x4200000000000000000000000000000000000006');
    expect(token).toBeDefined();
    expect(token!.symbol).toBe('WETH');
    expect(token!.chainId).toBe(10);
  });

  it('finds Base WETH (same address as Optimism, different chainId)', () => {
    const token = getTokenInfo(8453, '0x4200000000000000000000000000000000000006');
    expect(token).toBeDefined();
    expect(token!.symbol).toBe('WETH');
    expect(token!.chainId).toBe(8453);
  });

  it('returns undefined for unknown address', () => {
    expect(getTokenInfo(1, '0x0000000000000000000000000000000000000000')).toBeUndefined();
  });

  it('returns undefined when chainId does not match (cross-chain check)', () => {
    // Ethereum USDC address on Base should not match
    expect(
      getTokenInfo(8453, '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48'),
    ).toBeUndefined();
  });
});
