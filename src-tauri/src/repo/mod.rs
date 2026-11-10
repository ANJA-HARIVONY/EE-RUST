pub mod appointments;
pub mod clients;
pub mod employees;
pub mod invoices;
pub mod products;
pub mod services;
pub mod settings;

use chrono::{SecondsFormat, Utc};

pub fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn blank_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn require_text(value: &str, label: &str) -> Result<String, crate::error::AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(crate::error::AppError::message(format!(
            "{label} est obligatoire"
        )));
    }
    if trimmed.chars().count() > 120 {
        return Err(crate::error::AppError::message(format!(
            "{label} est trop long"
        )));
    }
    Ok(trimmed.to_string())
}
