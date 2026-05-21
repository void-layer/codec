import type { ChainId } from '@void-layer/types';

export interface TokenInfo {
  address: string;
  chainId: ChainId;
  symbol: string;
  decimals: number;
  name: string;
}

/**
 * @alpha Token list is intentionally empty at 0.1.0.
 * Populated from Uniswap Token List in a future minor release.
 */
export const SUPPORTED_TOKENS: readonly TokenInfo[] = [];
