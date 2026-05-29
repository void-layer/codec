import type { ChainId, NetworkConfig } from '@void-layer/types';

export interface ChainConfig extends NetworkConfig {
  /** Public RPC fallback list — NO API keys (Constitution VI). */
  publicRpcUrls: readonly string[];
}

export const CHAINS: Record<ChainId, ChainConfig> = {
  1: {
    chainId: 1,
    name: 'Ethereum',
    rpcUrls: ['https://eth.llamarpc.com'],
    publicRpcUrls: [
      'https://eth.llamarpc.com',
      'https://ethereum.publicnode.com',
      'https://rpc.ankr.com/eth',
    ],
    blockExplorer: 'https://etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  8453: {
    chainId: 8453,
    name: 'Base',
    rpcUrls: ['https://base.llamarpc.com'],
    publicRpcUrls: [
      'https://base.llamarpc.com',
      'https://base.publicnode.com',
      'https://rpc.ankr.com/base',
    ],
    blockExplorer: 'https://basescan.org',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  42161: {
    chainId: 42161,
    name: 'Arbitrum One',
    rpcUrls: ['https://arbitrum.llamarpc.com'],
    publicRpcUrls: [
      'https://arbitrum.llamarpc.com',
      'https://arbitrum-one.publicnode.com',
      'https://rpc.ankr.com/arbitrum',
    ],
    blockExplorer: 'https://arbiscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  10: {
    chainId: 10,
    name: 'Optimism',
    rpcUrls: ['https://optimism.llamarpc.com'],
    publicRpcUrls: [
      'https://optimism.llamarpc.com',
      'https://optimism.publicnode.com',
      'https://rpc.ankr.com/optimism',
    ],
    blockExplorer: 'https://optimistic.etherscan.io',
    nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
  },
  137: {
    chainId: 137,
    name: 'Polygon',
    rpcUrls: ['https://polygon.llamarpc.com'],
    publicRpcUrls: [
      'https://polygon.llamarpc.com',
      'https://polygon-bor-rpc.publicnode.com',
      'https://rpc.ankr.com/polygon',
    ],
    blockExplorer: 'https://polygonscan.com',
    nativeCurrency: { name: 'POL', symbol: 'POL', decimals: 18 },
  },
};

/** @deprecated Use CHAINS instead. */
export const SUPPORTED_CHAINS: Record<ChainId, ChainConfig> = CHAINS;
