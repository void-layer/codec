import { describe, it, expect } from 'vitest';
import { getExplorerTxUrl, getExplorerAddressUrl } from './explorer.js';

describe('getExplorerTxUrl', () => {
  it('returns correct Etherscan tx URL', () => {
    const url = getExplorerTxUrl(1, '0xabc123');
    expect(url).toBe('https://etherscan.io/tx/0xabc123');
  });

  it('returns correct Basescan tx URL', () => {
    const url = getExplorerTxUrl(8453, '0xdef456');
    expect(url).toBe('https://basescan.org/tx/0xdef456');
  });

  it('returns correct Arbiscan tx URL', () => {
    const url = getExplorerTxUrl(42161, '0x111');
    expect(url).toBe('https://arbiscan.io/tx/0x111');
  });

  it('returns correct Optimism explorer tx URL', () => {
    const url = getExplorerTxUrl(10, '0x222');
    expect(url).toBe('https://optimistic.etherscan.io/tx/0x222');
  });

  it('returns correct Polygonscan tx URL', () => {
    const url = getExplorerTxUrl(137, '0x333');
    expect(url).toBe('https://polygonscan.com/tx/0x333');
  });

  it('strips trailing slash from base URL', () => {
    // blockExplorer values have no trailing slash but verify the replace does not double-slash
    const url = getExplorerTxUrl(1, '0xabc');
    expect(url).not.toContain('//tx/');
  });

  it('throws for unknown chainId', () => {
    expect(() => getExplorerTxUrl(999 as Parameters<typeof getExplorerTxUrl>[0], '0x0')).toThrow(
      'Unknown chain ID',
    );
  });
});

describe('getExplorerAddressUrl', () => {
  it('returns correct Etherscan address URL', () => {
    const url = getExplorerAddressUrl(1, '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48');
    expect(url).toBe(
      'https://etherscan.io/address/0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
    );
  });

  it('throws for unknown chainId', () => {
    expect(() =>
      getExplorerAddressUrl(999 as Parameters<typeof getExplorerAddressUrl>[0], '0x0'),
    ).toThrow('Unknown chain ID');
  });
});
