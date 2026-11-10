use super::employee::Employee;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SalonSettings {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub siret: Option<String>,
    pub currency: String,
    pub vat_rate_bps: i64,
    pub invoice_prefix: String,
    pub next_invoice_seq: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub siret: String,
    pub currency: String,
    pub vat_rate_bps: i64,
    pub invoice_prefix: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub settings: SalonSettings,
    pub current_user: Employee,
    pub agenda_open_hour: u32,
    pub agenda_close_hour: u32,
    pub agenda_slot_minutes: u32,
    pub locale: String,
    pub low_stock_default: i64,
    pub database_path: String,
    pub low_stock_count: i64,
}
