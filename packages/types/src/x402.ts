export interface PaymentProof {
  version: string;
  invoiceHash: string;
  signature: string;
  chainId: number;
  expiry: number;
  payer: string;
}

export interface PaymentRequiredResponse {
  invoiceUrl: string;
  paymentProof?: PaymentProof;
}
