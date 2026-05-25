import { defineChain, type Chain } from 'viem';
import { CHAINS } from './chains.js';

export const ethereumWagmi: Chain = defineChain({
  id: 1,
  name: 'Ethereum',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: { default: { http: CHAINS[1].publicRpcUrls as string[] } },
  blockExplorers: { default: { name: 'Etherscan', url: CHAINS[1].blockExplorer } },
});

export const baseWagmi: Chain = defineChain({
  id: 8453,
  name: 'Base',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: { default: { http: CHAINS[8453].publicRpcUrls as string[] } },
  blockExplorers: { default: { name: 'Basescan', url: CHAINS[8453].blockExplorer } },
});

export const arbitrumWagmi: Chain = defineChain({
  id: 42161,
  name: 'Arbitrum One',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: { default: { http: CHAINS[42161].publicRpcUrls as string[] } },
  blockExplorers: { default: { name: 'Arbiscan', url: CHAINS[42161].blockExplorer } },
});

export const optimismWagmi: Chain = defineChain({
  id: 10,
  name: 'Optimism',
  nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  rpcUrls: { default: { http: CHAINS[10].publicRpcUrls as string[] } },
  blockExplorers: { default: { name: 'Optimistic Etherscan', url: CHAINS[10].blockExplorer } },
});

export const polygonWagmi: Chain = defineChain({
  id: 137,
  name: 'Polygon',
  nativeCurrency: { name: 'POL', symbol: 'POL', decimals: 18 },
  rpcUrls: { default: { http: CHAINS[137].publicRpcUrls as string[] } },
  blockExplorers: { default: { name: 'Polygonscan', url: CHAINS[137].blockExplorer } },
});

export const ALL_WAGMI_CHAINS = [
  ethereumWagmi,
  baseWagmi,
  arbitrumWagmi,
  optimismWagmi,
  polygonWagmi,
] as const;
