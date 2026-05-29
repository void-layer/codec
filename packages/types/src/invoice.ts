import type { ChainId } from './network.js';

/** Originator (payee) contact details. wallet_address is required for the issuer. */
export interface InvoiceFrom {
  name: string;
  wallet_address: string;
  email?: string;
  phone?: string;
  physical_address?: string;
  tax_id?: string;
}

/** Client (payer) contact details. All fields except name are optional. */
export interface InvoiceClient {
  name: string;
  wallet_address?: string;
  email?: string;
  phone?: string;
  physical_address?: string;
  tax_id?: string;
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
  from: InvoiceFrom;
  client: InvoiceClient;
  items: InvoiceItem[];
  total: string;
  salt: string;
  token_address?: string;
  notes?: string;
  tax?: string;
  discount?: string;
}
