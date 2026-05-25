import type { ChainId } from '@void-layer/types';

export interface TokenInfo {
  chainId: ChainId;
  /** Lowercase, 0x-prefixed ERC-20 address. */
  address: string;
  symbol: string;
  name: string;
  decimals: number;
  logoURI?: string;
}

const UNISWAP_CDN = 'https://raw.githubusercontent.com/Uniswap/assets/master/blockchains';

/**
 * Curated token list covering every (chainId, address) pair the codec dict knows.
 * Source: Uniswap Token List rows. Logos: Uniswap CDN.
 * NOT a runtime-imported full Uniswap list — only the rows the codec uses.
 */
export const TOKENS: readonly TokenInfo[] = [
  // ── Ethereum (chainId 1, codec codes 1-9) ───────────────────────────────
  {
    chainId: 1,
    address: '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/logo.png`,
  },
  {
    chainId: 1,
    address: '0xdac17f958d2ee523a2206206994597c13d831ec7',
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0xdAC17F958D2ee523a2206206994597C13D831ec7/logo.png`,
  },
  {
    chainId: 1,
    address: '0x6b175474e89094c44da98b954eedeac495271d0f',
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0x6B175474E89094C44Da98b954EedeAC495271d0F/logo.png`,
  },
  {
    chainId: 1,
    address: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
    symbol: 'WETH',
    name: 'Wrapped Ether',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2/logo.png`,
  },
  {
    chainId: 1,
    address: '0x2260fac5e5542a773aa44fbcfedf7c193bc2c599',
    symbol: 'WBTC',
    name: 'Wrapped BTC',
    decimals: 8,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599/logo.png`,
  },
  {
    chainId: 1,
    address: '0x1abaea1f7c830bd89acc67ec4af516284b1bc33c',
    symbol: 'EUROC',
    name: 'Euro Coin',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0x1aBaEA1f7C830bD89Acc67eC4aF516284b1bC33c/logo.png`,
  },
  {
    chainId: 1,
    address: '0x6c96de32cea08842dcc4058c14d3aaad7fa41dee',
    symbol: 'EURC',
    name: 'EURC',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/ethereum/assets/0x6c96de32cea08842dcc4058c14d3aaad7fa41dee/logo.png`,
  },
  // ── Arbitrum One (chainId 42161, codec codes 10-19) ─────────────────────
  {
    chainId: 42161,
    address: '0xaf88d065e77c8cc2239327c5edb3a432268e5831',
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/arbitrum/assets/0xaf88d065e77c8cC2239327C5EDb3A432268e5831/logo.png`,
  },
  {
    chainId: 42161,
    address: '0xff970a61a04b1ca14834a43f5de4533ebddb5cc8',
    symbol: 'USDC.e',
    name: 'Bridged USDC',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/arbitrum/assets/0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8/logo.png`,
  },
  {
    chainId: 42161,
    address: '0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9',
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/arbitrum/assets/0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9/logo.png`,
  },
  {
    chainId: 42161,
    address: '0xda10009cbd5d07dd0cecc66161fc93d7c9000da1',
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/arbitrum/assets/0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1/logo.png`,
  },
  {
    chainId: 42161,
    address: '0x82af49447d8a07e3bd95bd0d56f35241523fbab1',
    symbol: 'WETH',
    name: 'Wrapped Ether',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/arbitrum/assets/0x82aF49447D8a07e3bd95BD0d56f35241523fBab1/logo.png`,
  },
  {
    chainId: 42161,
    address: '0x2f2a2543b76a4166549f7aab2e75bef0aefc5b0f',
    symbol: 'WBTC',
    name: 'Wrapped BTC',
    decimals: 8,
    logoURI: `${UNISWAP_CDN}/arbitrum/assets/0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f/logo.png`,
  },
  // ── Optimism (chainId 10, codec codes 20-29) ─────────────────────────────
  {
    chainId: 10,
    address: '0x0b2c639c533813f4aa9d7837caf62653d097ff85',
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/optimism/assets/0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85/logo.png`,
  },
  {
    chainId: 10,
    address: '0x7f5c764cbc14f9669b88837ca1490cca17c31607',
    symbol: 'USDC.e',
    name: 'Bridged USDC',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/optimism/assets/0x7F5c764cBc14f9669B88837ca1490cCa17c31607/logo.png`,
  },
  {
    chainId: 10,
    address: '0x94b008aa00579c1307b0ef2c499ad98a8ce58e58',
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/optimism/assets/0x94b008aA00579c1307B0EF2c499aD98a8ce58e58/logo.png`,
  },
  {
    chainId: 10,
    address: '0x4200000000000000000000000000000000000006',
    symbol: 'WETH',
    name: 'Wrapped Ether',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/optimism/assets/0x4200000000000000000000000000000000000006/logo.png`,
  },
  {
    chainId: 10,
    address: '0x68f180fcce6836688e9084f035309e29bf0a2095',
    symbol: 'WBTC',
    name: 'Wrapped BTC',
    decimals: 8,
    logoURI: `${UNISWAP_CDN}/optimism/assets/0x68f180fcCe6836688e9084f035309E29Bf0A2095/logo.png`,
  },
  // ── Polygon (chainId 137, codec codes 30-39) ─────────────────────────────
  {
    chainId: 137,
    address: '0x3c499c542cef5e3811e1192ce70d8cc03d5c3359',
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/polygon/assets/0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359/logo.png`,
  },
  {
    chainId: 137,
    address: '0x2791bca1f2de4661ed88a30c99a7a9449aa84174',
    symbol: 'USDC.e',
    name: 'Bridged USDC',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/polygon/assets/0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174/logo.png`,
  },
  {
    chainId: 137,
    address: '0xc2132d05d31c914a87c6611c10748aeb04b58e8f',
    symbol: 'USDT',
    name: 'Tether USD',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/polygon/assets/0xc2132D05D31c914a87C6611C10748AEb04B58e8F/logo.png`,
  },
  {
    chainId: 137,
    address: '0x8f3cf7ad23cd3cadbd9735aff958023239c6a063',
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/polygon/assets/0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063/logo.png`,
  },
  {
    chainId: 137,
    address: '0x7ceb23fd6bc0add59e62ac25578270cff1b9f619',
    symbol: 'WETH',
    name: 'Wrapped Ether',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/polygon/assets/0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619/logo.png`,
  },
  {
    chainId: 137,
    address: '0x1bfd67037b42cf73acf2047067bd4f2c47d9bfd6',
    symbol: 'WBTC',
    name: 'Wrapped BTC',
    decimals: 8,
    logoURI: `${UNISWAP_CDN}/polygon/assets/0x1BFD67037B42Cf73acf2047067bd4F2C47D9BfD6/logo.png`,
  },
  // ── Base (chainId 8453, codec codes 40-49) ───────────────────────────────
  {
    chainId: 8453,
    address: '0x833589fcd6edb6e08f4c7c32d4f71b54bda02913',
    symbol: 'USDC',
    name: 'USD Coin',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/base/assets/0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913/logo.png`,
  },
  {
    chainId: 8453,
    address: '0xd9aaec86b65d86f6a7b5b1b0c42ffa531710b6ca',
    symbol: 'USDbC',
    name: 'Bridged USDC',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/base/assets/0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA/logo.png`,
  },
  {
    chainId: 8453,
    address: '0x50c5725949a6f0c72e6c4a641f24049a917db0cb',
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/base/assets/0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb/logo.png`,
  },
  {
    chainId: 8453,
    address: '0x4200000000000000000000000000000000000006',
    symbol: 'WETH',
    name: 'Wrapped Ether',
    decimals: 18,
    logoURI: `${UNISWAP_CDN}/base/assets/0x4200000000000000000000000000000000000006/logo.png`,
  },
  {
    chainId: 8453,
    address: '0x0555e30da8f98308edb960aa94c0ed47230d2b9c',
    symbol: 'cbBTC',
    name: 'Coinbase Wrapped BTC',
    decimals: 8,
    logoURI: `${UNISWAP_CDN}/base/assets/0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf/logo.png`,
  },
  {
    chainId: 8453,
    address: '0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42',
    symbol: 'EURC',
    name: 'EURC',
    decimals: 6,
    logoURI: `${UNISWAP_CDN}/base/assets/0x60a3E35Cc302bFA44Cb288Bc5a4F316Fdb1adb42/logo.png`,
  },
];

export function getTokenInfo(chainId: ChainId, address: string): TokenInfo | undefined {
  const target = address.toLowerCase();
  return TOKENS.find((t) => t.chainId === chainId && t.address === target);
}

/** @deprecated Use TOKENS instead. */
export const SUPPORTED_TOKENS: readonly TokenInfo[] = TOKENS;
