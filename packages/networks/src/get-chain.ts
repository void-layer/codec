import type { ChainId } from '@void-layer/types';
import { CHAINS, type ChainConfig } from './chains.js';

export type { ChainConfig };

export function getChainConfig(chainId: ChainId): ChainConfig {
  const config = CHAINS[chainId];
  if (!config) throw new Error(`Unknown chain ID: ${chainId}`);
  return config;
}

export function tryGetChainConfig(chainId: number): ChainConfig | null {
  return (CHAINS as Record<number, ChainConfig | undefined>)[chainId] ?? null;
}
