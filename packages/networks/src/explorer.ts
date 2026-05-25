import type { ChainId } from '@void-layer/types';
import { getChainConfig } from './get-chain.js';

export function getExplorerTxUrl(chainId: ChainId, txHash: string): string {
  const base = getChainConfig(chainId).blockExplorer.replace(/\/$/, '');
  return `${base}/tx/${txHash}`;
}

export function getExplorerAddressUrl(chainId: ChainId, address: string): string {
  const base = getChainConfig(chainId).blockExplorer.replace(/\/$/, '');
  return `${base}/address/${address}`;
}
