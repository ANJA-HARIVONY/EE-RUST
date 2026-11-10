use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub duration_minutes: i64,
    pub price_cents: i64,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInput {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub duration_minutes: i64,
    pub price_cents: i64,
    pub active: bool,
}
