import type { ChainId } from '@void-layer/types';
import { SUPPORTED_CHAINS } from './chains.js';

export function getPublicRpcUrl(chainId: ChainId): string {
  const chain = SUPPORTED_CHAINS[chainId];
  if (!chain) throw new Error(`Unsupported chainId: ${chainId}`);
  return chain.rpcUrls[0] ?? (() => { throw new Error(`No rpcUrl for chainId: ${chainId}`); })();
}
