use super::{blank_to_none, now, require_text};
use crate::config::is_hex_color;
use crate::domain::{Employee, EmployeeInput, Role};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

const SELECT: &str = "
    SELECT e.id, e.first_name, e.last_name, e.email, e.phone,
           e.role_id, r.code, r.label, e.color, e.active, e.notes
    FROM employees e
    JOIN roles r ON r.id = e.role_id
";

pub fn roles(conn: &Connection) -> Result<Vec<Role>, AppError> {
    let mut stmt = conn.prepare("SELECT id, code, label FROM roles ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Role {
            id: row.get(0)?,
            code: row.get(1)?,
            label: row.get(2)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn list(conn: &Connection, include_inactive: bool) -> Result<Vec<Employee>, AppError> {
    let sql = format!(
        "{SELECT} WHERE (?1 = 1 OR e.active = 1) ORDER BY e.active DESC, e.last_name, e.first_name"
    );
    let mut stmt = conn.prepare(&sql)?;
    let flag = if include_inactive { 1 } else { 0 };
    let rows = stmt.query_map(params![flag], map_employee)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get(conn: &Connection, id: i64) -> Result<Employee, AppError> {
    conn.query_row(&format!("{SELECT} WHERE e.id = ?1"), [id], map_employee)
        .optional()?
        .ok_or_else(|| AppError::message("employé introuvable"))
}

pub fn find_current(conn: &Connection, email: &str) -> Result<Employee, AppError> {
    if let Some(email) = blank_to_none(email) {
        if let Some(found) = conn
            .query_row(
                &format!("{SELECT} WHERE lower(e.email) = lower(?1) ORDER BY e.id LIMIT 1"),
                [email],
                map_employee,
            )
            .optional()?
        {
            return Ok(found);
        }
    }
    conn.query_row(
        &format!("{SELECT} WHERE e.active = 1 ORDER BY e.id LIMIT 1"),
        [],
        map_employee,
    )
    .optional()?
    .ok_or_else(|| AppError::message("aucun employé actif"))
}

pub fn save(conn: &Connection, input: &EmployeeInput) -> Result<Employee, AppError> {
    let first = require_text(&input.first_name, "Le prénom")?;
    let last = require_text(&input.last_name, "Le nom")?;
    let color = input.color.trim().to_string();
    if !is_hex_color(&color) {
        return Err(AppError::message(
            "la couleur doit être au format #RRGGBB",
        ));
    }
    let role_ok: Option<i64> = conn
        .query_row("SELECT id FROM roles WHERE id = ?1", [input.role_id], |row| {
            row.get(0)
        })
        .optional()?;
    if role_ok.is_none() {
        return Err(AppError::message("rôle inconnu"));
    }
    if let Some(id) = input.id {
        let current = get(conn, id)?;
        if current.role_code == "admin" && current.active && (!input.active || {
            let new_code: String =
                conn.query_row("SELECT code FROM roles WHERE id = ?1", [input.role_id], |row| {
                    row.get(0)
                })?;
            new_code != "admin"
        }) {
            ensure_admin_remains(conn, Some(id), current.role_id, false)?;
        }
    }
    let email = blank_to_none(&input.email);
    let phone = blank_to_none(&input.phone);
    let notes = blank_to_none(&input.notes);
    let stamp = now();
    let active = if input.active { 1 } else { 0 };
    let id = if let Some(id) = input.id {
        let updated = conn.execute(
            "UPDATE employees
             SET first_name = ?1, last_name = ?2, email = ?3, phone = ?4, role_id = ?5,
                 color = ?6, active = ?7, notes = ?8, updated_at = ?9
             WHERE id = ?10",
            params![first, last, email, phone, input.role_id, color, active, notes, stamp, id],
        )?;
        if updated != 1 {
            return Err(AppError::message("employé introuvable"));
        }
        id
    } else {
        conn.execute(
            "INSERT INTO employees (
                first_name, last_name, email, phone, role_id, color, active, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![first, last, email, phone, input.role_id, color, active, notes, stamp],
        )?;
        conn.last_insert_rowid()
    };
    get(conn, id)
}

pub fn set_active(conn: &Connection, id: i64, active: bool) -> Result<Employee, AppError> {
    let current = get(conn, id)?;
    if !active {
        ensure_admin_remains(conn, Some(id), current.role_id, false)?;
    }
    let stamp = now();
    conn.execute(
        "UPDATE employees SET active = ?1, updated_at = ?2 WHERE id = ?3",
        params![if active { 1 } else { 0 }, stamp, id],
    )?;
    get(conn, id)
}

fn ensure_admin_remains(
    conn: &Connection,
    id: Option<i64>,
    role_id: i64,
    _active: bool,
) -> Result<(), AppError> {
    let code: String = conn.query_row("SELECT code FROM roles WHERE id = ?1", [role_id], |row| {
        row.get(0)
    })?;
    if code != "admin" {
        return Ok(());
    }
    let others: i64 = conn.query_row(
        "SELECT COUNT(*) FROM employees e
         JOIN roles r ON r.id = e.role_id
         WHERE r.code = 'admin' AND e.active = 1 AND e.id != ?1",
        [id.unwrap_or(-1)],
        |row| row.get(0),
    )?;
    if others == 0 {
        return Err(AppError::message(
            "il doit rester au moins un administrateur actif",
        ));
    }
    Ok(())
}

fn map_employee(row: &rusqlite::Row<'_>) -> rusqlite::Result<Employee> {
    Ok(Employee {
        id: row.get(0)?,
        first_name: row.get(1)?,
        last_name: row.get(2)?,
        email: row.get(3)?,
        phone: row.get(4)?,
        role_id: row.get(5)?,
        role_code: row.get(6)?,
        role_label: row.get(7)?,
        color: row.get(8)?,
        active: row.get(9)?,
        notes: row.get(10)?,
    })
}
