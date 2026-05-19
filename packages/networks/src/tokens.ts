import type { ChainId } from '@void-layer/types';

export interface TokenInfo {
  address: string;
  chainId: ChainId;
  symbol: string;
  decimals: number;
  name: string;
}

// Phase 2 populates from Uniswap Token List
export const SUPPORTED_TOKENS: readonly TokenInfo[] = [];
