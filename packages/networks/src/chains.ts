import type { ChainId, NetworkConfig } from '@void-layer/types';

export const SUPPORTED_CHAINS: Record<ChainId, NetworkConfig> = {
  1: {
    chainId: 1,
    name: 'Ethereum',
    rpcUrls: ['https://eth.llamarpc.com'],
    blockExplorer: 'https://etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  8453: {
    chainId: 8453,
    name: 'Base',
    rpcUrls: ['https://base.llamarpc.com'],
    blockExplorer: 'https://basescan.org',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  42161: {
    chainId: 42161,
    name: 'Arbitrum One',
    rpcUrls: ['https://arbitrum.llamarpc.com'],
    blockExplorer: 'https://arbiscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  10: {
    chainId: 10,
    name: 'Optimism',
    rpcUrls: ['https://optimism.llamarpc.com'],
    blockExplorer: 'https://optimistic.etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  137: {
    chainId: 137,
    name: 'Polygon',
    rpcUrls: ['https://polygon.llamarpc.com'],
    blockExplorer: 'https://polygonscan.com',
    nativeCurrency: { name: 'POL', symbol: 'POL', decimals: 18 },
  },
};
