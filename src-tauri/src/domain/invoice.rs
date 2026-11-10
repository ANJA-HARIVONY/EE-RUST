use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLine {
    pub id: i64,
    pub kind: String,
    pub service_id: Option<i64>,
    pub product_id: Option<i64>,
    pub appointment_id: Option<i64>,
    pub description: String,
    pub quantity: i64,
    pub unit_price_cents: i64,
    pub line_total_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i64,
    pub number: String,
    pub client_id: i64,
    pub client_name: String,
    pub employee_id: Option<i64>,
    pub employee_name: Option<String>,
    pub status: String,
    pub issued_at: Option<String>,
    pub paid_at: Option<String>,
    pub notes: Option<String>,
    pub currency: String,
    pub vat_rate_bps: i64,
    pub subtotal_cents: i64,
    pub vat_cents: i64,
    pub total_cents: i64,
    pub lines: Vec<InvoiceLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceSummary {
    pub id: i64,
    pub number: String,
    pub client_name: String,
    pub status: String,
    pub total_cents: i64,
    pub currency: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceCreate {
    pub client_id: i64,
    pub employee_id: Option<i64>,
    pub appointment_id: Option<i64>,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineInput {
    pub invoice_id: i64,
    pub kind: String,
    pub service_id: Option<i64>,
    pub product_id: Option<i64>,
    pub quantity: i64,
}
