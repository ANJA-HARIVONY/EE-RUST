use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentServiceLine {
    pub service_id: i64,
    pub name: String,
    pub duration_minutes: i64,
    pub price_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Appointment {
    pub id: i64,
    pub client_id: i64,
    pub client_name: String,
    pub employee_id: i64,
    pub employee_name: String,
    pub employee_color: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub notes: Option<String>,
    pub services: Vec<AppointmentServiceLine>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentInput {
    pub id: Option<i64>,
    pub client_id: i64,
    pub employee_id: i64,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub notes: String,
    pub service_ids: Vec<i64>,
}
