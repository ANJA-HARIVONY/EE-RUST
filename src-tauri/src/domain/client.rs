use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInput {
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
}
