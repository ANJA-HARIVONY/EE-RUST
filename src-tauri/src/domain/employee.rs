use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: i64,
    pub code: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Employee {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role_id: i64,
    pub role_code: String,
    pub role_label: String,
    pub color: String,
    pub active: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeInput {
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub role_id: i64,
    pub color: String,
    pub active: bool,
    pub notes: String,
}
