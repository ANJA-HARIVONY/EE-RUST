use super::{blank_to_none, require_text};
use crate::domain::{SalonSettings, SettingsInput};
use crate::error::AppError;
use rusqlite::{params, Connection};

pub fn get(conn: &Connection) -> Result<SalonSettings, AppError> {
    conn.query_row(
        "SELECT name, address, phone, email, siret, currency, vat_rate_bps, invoice_prefix, next_invoice_seq
         FROM salon_settings WHERE id = 1",
        [],
        |row| {
            Ok(SalonSettings {
                name: row.get(0)?,
                address: row.get(1)?,
                phone: row.get(2)?,
                email: row.get(3)?,
                siret: row.get(4)?,
                currency: row.get(5)?,
                vat_rate_bps: row.get(6)?,
                invoice_prefix: row.get(7)?,
                next_invoice_seq: row.get(8)?,
            })
        },
    )
    .map_err(|_| AppError::message("réglages du salon introuvables"))
}

pub fn save(conn: &Connection, input: &SettingsInput) -> Result<SalonSettings, AppError> {
    let name = require_text(&input.name, "Le nom du salon")?;
    let currency = input.currency.trim().to_uppercase();
    if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(AppError::message(
            "la devise doit être un code ISO à 3 lettres, par exemple EUR",
        ));
    }
    if !(0..=10_000).contains(&input.vat_rate_bps) {
        return Err(AppError::message(
            "le taux de TVA doit être compris entre 0 et 10000 points de base",
        ));
    }
    let prefix = input.invoice_prefix.trim().to_uppercase();
    if prefix.is_empty() || prefix.len() > 8 || !prefix.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Err(AppError::message(
            "le préfixe de facture doit contenir 1 à 8 lettres ou chiffres",
        ));
    }
    let updated = conn.execute(
        "UPDATE salon_settings
         SET name = ?1, address = ?2, phone = ?3, email = ?4, siret = ?5,
             currency = ?6, vat_rate_bps = ?7, invoice_prefix = ?8
         WHERE id = 1",
        params![
            name,
            blank_to_none(&input.address),
            blank_to_none(&input.phone),
            blank_to_none(&input.email),
            blank_to_none(&input.siret),
            currency,
            input.vat_rate_bps,
            prefix,
        ],
    )?;
    if updated != 1 {
        return Err(AppError::message("réglages du salon introuvables"));
    }
    get(conn)
}
