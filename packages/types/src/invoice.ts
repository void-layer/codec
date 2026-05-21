import type { ChainId } from './network.js';

export interface InvoiceParty {
  name: string;
  wallet_address?: string;
  email?: string;
}

export interface InvoiceItem {
  description: string;
  quantity: number;
  rate: string;
}

export interface Invoice {
  invoice_id: string;
  issued_at: number;
  due_at: number;
  network_id: ChainId;
  currency: string;
  decimals: number;
  from: InvoiceParty;
  client: InvoiceParty;
  items: InvoiceItem[];
  total: string;
  salt: string;
  notes?: string;
  tax?: string;
  discount?: string;
}
