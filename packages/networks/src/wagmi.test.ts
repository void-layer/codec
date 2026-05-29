import { describe, it, expect } from 'vitest';
import {
  ethereumWagmi,
  baseWagmi,
  arbitrumWagmi,
  optimismWagmi,
  polygonWagmi,
  ALL_WAGMI_CHAINS,
} from './wagmi.js';

describe('wagmi chain configs', () => {
  const chains = [
    { chain: ethereumWagmi, id: 1, name: 'Ethereum' },
    { chain: baseWagmi, id: 8453, name: 'Base' },
    { chain: arbitrumWagmi, id: 42161, name: 'Arbitrum One' },
    { chain: optimismWagmi, id: 10, name: 'Optimism' },
    { chain: polygonWagmi, id: 137, name: 'Polygon' },
  ];

  it.each(chains)('$name has correct id', ({ chain, id }) => {
    expect(chain.id).toBe(id);
  });

  it.each(chains)('$name has correct name', ({ chain, name }) => {
    expect(chain.name).toBe(name);
  });

  it.each(chains)('$name has nativeCurrency with 18 decimals', ({ chain }) => {
    expect(chain.nativeCurrency.decimals).toBe(18);
  });

  it.each(chains)('$name has at least one rpc http URL', ({ chain }) => {
    const urls = chain.rpcUrls.default.http;
    expect(urls.length).toBeGreaterThanOrEqual(1);
    expect(urls[0]).toMatch(/^https?:\/\//);
  });

  it.each(chains)('$name has block explorer URL', ({ chain }) => {
    const explorer = chain.blockExplorers?.default.url;
    expect(typeof explorer).toBe('string');
    expect(explorer!.length).toBeGreaterThan(0);
  });

  it('Polygon uses POL as native currency symbol', () => {
    expect(polygonWagmi.nativeCurrency.symbol).toBe('POL');
  });

  it('ALL_WAGMI_CHAINS contains all 5 chains', () => {
    expect(ALL_WAGMI_CHAINS).toHaveLength(5);
    const ids = ALL_WAGMI_CHAINS.map((c) => c.id);
    expect(ids).toContain(1);
    expect(ids).toContain(8453);
    expect(ids).toContain(42161);
    expect(ids).toContain(10);
    expect(ids).toContain(137);
  });
});
