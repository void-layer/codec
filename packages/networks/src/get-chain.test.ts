import { describe, it, expect } from 'vitest';
import { getChainConfig, tryGetChainConfig } from './get-chain.js';

describe('getChainConfig', () => {
  it('returns Ethereum config for chainId 1', () => {
    const config = getChainConfig(1);
    expect(config.chainId).toBe(1);
    expect(config.name).toBe('Ethereum');
    expect(config.nativeCurrency.symbol).toBe('ETH');
    expect(config.blockExplorer).toBe('https://etherscan.io');
  });

  it('returns Base config for chainId 8453', () => {
    const config = getChainConfig(8453);
    expect(config.chainId).toBe(8453);
    expect(config.name).toBe('Base');
  });

  it('returns Arbitrum config for chainId 42161', () => {
    const config = getChainConfig(42161);
    expect(config.chainId).toBe(42161);
    expect(config.name).toBe('Arbitrum One');
  });

  it('returns Optimism config for chainId 10', () => {
    const config = getChainConfig(10);
    expect(config.chainId).toBe(10);
    expect(config.name).toBe('Optimism');
  });

  it('returns Polygon config for chainId 137', () => {
    const config = getChainConfig(137);
    expect(config.chainId).toBe(137);
    expect(config.nativeCurrency.symbol).toBe('POL');
  });

  it('throws on unknown chainId 999', () => {
    expect(() => getChainConfig(999 as Parameters<typeof getChainConfig>[0])).toThrow(
      'Unknown chain ID: 999',
    );
  });
});

describe('tryGetChainConfig', () => {
  it('returns config for known chainId', () => {
    const config = tryGetChainConfig(1);
    expect(config).not.toBeNull();
    expect(config!.chainId).toBe(1);
  });

  it('returns null for unknown chainId', () => {
    expect(tryGetChainConfig(999)).toBeNull();
  });

  it('returns null for chainId 0', () => {
    expect(tryGetChainConfig(0)).toBeNull();
  });
});
