import type { ChainId } from './network.js';

export interface FrameContext {
  fid: number;
  username: string;
  displayName: string;
}

export interface FrameState {
  invoiceId: string;
  network: ChainId;
}
