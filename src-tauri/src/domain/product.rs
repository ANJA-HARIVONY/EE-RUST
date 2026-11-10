use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: i64,
    pub sku: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub stock_qty: i64,
    pub low_stock_threshold: i64,
    pub active: bool,
    pub low_stock: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductInput {
    pub id: Option<i64>,
    pub sku: String,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub low_stock_threshold: i64,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAdjust {
    pub product_id: i64,
    pub kind: String,
    pub quantity: i64,
    pub reason: String,
    pub employee_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovement {
    pub id: i64,
    pub kind: String,
    pub quantity: i64,
    pub reason: Option<String>,
    pub created_at: String,
}
