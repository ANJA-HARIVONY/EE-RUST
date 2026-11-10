use super::{blank_to_none, now, require_text};
use crate::domain::{Client, ClientInput};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

const SELECT: &str = "SELECT id, first_name, last_name, email, phone, notes FROM clients";

pub fn list(conn: &Connection, query: &str) -> Result<Vec<Client>, AppError> {
    let q = query.trim();
    if q.is_empty() {
        let mut stmt = conn.prepare(&format!("{SELECT} ORDER BY last_name, first_name"))?;
        let rows = stmt.query_map([], map_client)?;
        return rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from);
    }
    let pattern = like_pattern(q);
    let mut stmt = conn.prepare(&format!(
        "{SELECT}
         WHERE first_name LIKE ?1 ESCAPE '\\'
            OR last_name LIKE ?1 ESCAPE '\\'
            OR IFNULL(phone, '') LIKE ?1 ESCAPE '\\'
            OR IFNULL(email, '') LIKE ?1 ESCAPE '\\'
         ORDER BY last_name, first_name"
    ))?;
    let rows = stmt.query_map(params![pattern], map_client)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get(conn: &Connection, id: i64) -> Result<Client, AppError> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], map_client)
        .optional()?
        .ok_or_else(|| AppError::message("client introuvable"))
}

pub fn save(conn: &Connection, input: &ClientInput) -> Result<Client, AppError> {
    let first = require_text(&input.first_name, "Le prénom")?;
    let last = require_text(&input.last_name, "Le nom")?;
    let email = blank_to_none(&input.email);
    let phone = blank_to_none(&input.phone);
    let notes = blank_to_none(&input.notes);
    let stamp = now();
    let id = if let Some(id) = input.id {
        let updated = conn.execute(
            "UPDATE clients
             SET first_name = ?1, last_name = ?2, email = ?3, phone = ?4, notes = ?5, updated_at = ?6
             WHERE id = ?7",
            params![first, last, email, phone, notes, stamp, id],
        )?;
        if updated != 1 {
            return Err(AppError::message("client introuvable"));
        }
        id
    } else {
        conn.execute(
            "INSERT INTO clients (first_name, last_name, email, phone, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![first, last, email, phone, notes, stamp],
        )?;
        conn.last_insert_rowid()
    };
    get(conn, id)
}

fn map_client(row: &rusqlite::Row<'_>) -> rusqlite::Result<Client> {
    Ok(Client {
        id: row.get(0)?,
        first_name: row.get(1)?,
        last_name: row.get(2)?,
        email: row.get(3)?,
        phone: row.get(4)?,
        notes: row.get(5)?,
    })
}

fn like_pattern(query: &str) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}
