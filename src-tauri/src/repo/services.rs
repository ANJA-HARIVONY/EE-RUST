use super::{blank_to_none, now, require_text};
use crate::domain::{Service, ServiceInput};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

const SELECT: &str =
    "SELECT id, name, description, duration_minutes, price_cents, active FROM services";

pub fn list(conn: &Connection, include_inactive: bool) -> Result<Vec<Service>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE (?1 = 1 OR active = 1) ORDER BY active DESC, name"
    ))?;
    let flag = if include_inactive { 1 } else { 0 };
    let rows = stmt.query_map([flag], map_service)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get(conn: &Connection, id: i64) -> Result<Service, AppError> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], map_service)
        .optional()?
        .ok_or_else(|| AppError::message("prestation introuvable"))
}

pub fn save(conn: &Connection, input: &ServiceInput) -> Result<Service, AppError> {
    let name = require_text(&input.name, "Le nom")?;
    if input.duration_minutes <= 0 {
        return Err(AppError::message("la durée doit être supérieure à 0"));
    }
    if input.price_cents < 0 {
        return Err(AppError::message("le prix ne peut pas être négatif"));
    }
    let description = blank_to_none(&input.description);
    let stamp = now();
    let active = if input.active { 1 } else { 0 };
    let id = if let Some(id) = input.id {
        let updated = conn.execute(
            "UPDATE services
             SET name = ?1, description = ?2, duration_minutes = ?3, price_cents = ?4,
                 active = ?5, updated_at = ?6
             WHERE id = ?7",
            params![
                name,
                description,
                input.duration_minutes,
                input.price_cents,
                active,
                stamp,
                id
            ],
        )?;
        if updated != 1 {
            return Err(AppError::message("prestation introuvable"));
        }
        id
    } else {
        conn.execute(
            "INSERT INTO services (
                name, description, duration_minutes, price_cents, active, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                name,
                description,
                input.duration_minutes,
                input.price_cents,
                active,
                stamp
            ],
        )?;
        conn.last_insert_rowid()
    };
    get(conn, id)
}

pub fn set_active(conn: &Connection, id: i64, active: bool) -> Result<Service, AppError> {
    let stamp = now();
    let updated = conn.execute(
        "UPDATE services SET active = ?1, updated_at = ?2 WHERE id = ?3",
        params![if active { 1 } else { 0 }, stamp, id],
    )?;
    if updated != 1 {
        return Err(AppError::message("prestation introuvable"));
    }
    get(conn, id)
}

fn map_service(row: &rusqlite::Row<'_>) -> rusqlite::Result<Service> {
    Ok(Service {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        duration_minutes: row.get(3)?,
        price_cents: row.get(4)?,
        active: row.get(5)?,
    })
}
