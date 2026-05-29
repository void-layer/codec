export type ChainId = 1 | 8453 | 42161 | 10 | 137;

export interface NetworkConfig {
  chainId: ChainId;
  name: string;
  rpcUrls: readonly string[];
  blockExplorer: string;
  nativeCurrency: { name: string; symbol: string; decimals: number };
}
