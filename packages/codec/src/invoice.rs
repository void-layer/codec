//! Canonical invoice data structures (v1 schema, LOCKED).

use serde::{Deserialize, Serialize};

/// A single line item in an invoice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceItem {
    /// Human-readable description of the line item.
    pub description: String,
    /// Quantity (may be fractional, e.g. 1.5 hours).
    pub quantity: f64,
    /// Unit rate in atomic token units (BigInt-safe string, e.g. "1000000" for 1 USDC).
    pub rate: String,
}

/// Originator (payee) contact details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceFrom {
    /// Display name of the issuer.
    pub name: String,
    /// EVM wallet address (0x-prefixed hex).
    pub wallet_address: String,
    /// Optional contact email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Optional contact phone number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Optional physical/postal address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_address: Option<String>,
    /// Optional tax identification number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
}

/// Client (payer) contact details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceClient {
    /// Display name of the client.
    pub name: String,
    /// Optional EVM wallet address (0x-prefixed hex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<String>,
    /// Optional contact email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Optional contact phone number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Optional physical/postal address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_address: Option<String>,
    /// Optional tax identification number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
}

/// Canonical invoice data structure (v1 schema, LOCKED).
///
/// All monetary amounts are represented as `String` for BigInt-safe JS boundary
/// (D-B11). Amounts are in atomic token units (e.g. USDC uses 6 decimals).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique invoice identifier (e.g. "INV-001").
    pub invoice_id: String,
    /// Unix timestamp of invoice creation (seconds).
    pub issued_at: u32,
    /// Unix timestamp of payment due date (seconds).
    pub due_at: u32,
    /// EVM chain ID (e.g. 1 = Ethereum, 8453 = Base).
    pub network_id: u32,
    /// Token currency symbol (e.g. "USDC", "ETH").
    pub currency: String,
    /// Token decimals (e.g. 6 for USDC, 18 for ETH).
    pub decimals: u8,
    /// Issuer details (name, wallet address, optional contact info).
    pub from: InvoiceFrom,
    /// Client/payer details.
    pub client: InvoiceClient,
    /// Line items.
    pub items: Vec<InvoiceItem>,
    /// ERC-20 token contract address (None for native ETH/MATIC).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,
    /// Payment notes (max 280 chars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Tax percentage as string (e.g. "10.5").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax: Option<String>,
    /// Discount percentage as string (e.g. "5").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discount: Option<String>,
    /// Total payment amount in atomic units (BigInt-safe string). Includes magic dust if applied.
    pub total: String,
    /// 16-byte random salt for magic dust and domain separator (hex string).
    /// Caller provides this; encoder uses it as-is for deterministic re-encoding.
    pub salt: String,
}
