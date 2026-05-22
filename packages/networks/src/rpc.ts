import type { ChainId } from '@void-layer/types';
import { SUPPORTED_CHAINS } from './chains.js';

export function getPublicRpcUrl(chainId: ChainId): string {
  const chain = SUPPORTED_CHAINS[chainId];
  if (!chain) throw new Error(`Unsupported chainId: ${chainId}`);
  const url = chain.rpcUrls[0];
  /* v8 ignore next -- defensive: every SUPPORTED_CHAINS entry has a non-empty rpcUrls */
  if (!url) throw new Error(`No rpcUrl for chainId: ${chainId}`);
  return url;
}
