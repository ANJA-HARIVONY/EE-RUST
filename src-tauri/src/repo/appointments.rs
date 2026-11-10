use std::collections::HashMap;

use super::{blank_to_none, now};
use crate::domain::{Appointment, AppointmentServiceLine};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

pub const STATUSES: [&str; 6] = [
    "scheduled",
    "confirmed",
    "in_progress",
    "completed",
    "cancelled",
    "no_show",
];

pub fn list(
    conn: &Connection,
    from: &str,
    to: &str,
    employee_id: Option<i64>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.client_id, c.first_name || ' ' || c.last_name,
                a.employee_id, e.first_name || ' ' || e.last_name, e.color,
                a.starts_at, a.ends_at, a.status, a.notes
         FROM appointments a
         JOIN clients c ON c.id = a.client_id
         JOIN employees e ON e.id = a.employee_id
         WHERE a.starts_at < ?2
           AND a.ends_at > ?1
           AND (?3 IS NULL OR a.employee_id = ?3)
           AND (?4 IS NULL OR a.client_id = ?4)
         ORDER BY a.starts_at",
    )?;
    let rows = stmt.query_map(params![from, to, employee_id, client_id], map_head)?;
    let mut appointments = rows.collect::<Result<Vec<_>, _>>()?;
    attach_services(conn, &mut appointments)?;
    Ok(appointments)
}

pub fn get(conn: &Connection, id: i64) -> Result<Appointment, AppError> {
    let mut appointment = conn
        .query_row(
            "SELECT a.id, a.client_id, c.first_name || ' ' || c.last_name,
                    a.employee_id, e.first_name || ' ' || e.last_name, e.color,
                    a.starts_at, a.ends_at, a.status, a.notes
             FROM appointments a
             JOIN clients c ON c.id = a.client_id
             JOIN employees e ON e.id = a.employee_id
             WHERE a.id = ?1",
            [id],
            map_head,
        )
        .optional()?
        .ok_or_else(|| AppError::message("rendez-vous introuvable"))?;
    let mut list = vec![appointment];
    attach_services(conn, &mut list)?;
    appointment = list.pop().unwrap();
    Ok(appointment)
}

pub fn service_ids(conn: &Connection, appointment_id: i64) -> Result<Vec<i64>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT service_id FROM appointment_services WHERE appointment_id = ?1 ORDER BY id",
    )?;
    let rows = stmt.query_map([appointment_id], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn replace_services(
    conn: &Connection,
    appointment_id: i64,
    service_ids: &[i64],
) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM appointment_services WHERE appointment_id = ?1",
        [appointment_id],
    )?;
    for service_id in service_ids {
        let changed = conn.execute(
            "INSERT INTO appointment_services (
                appointment_id, service_id, name_snapshot, duration_minutes, price_cents
             )
             SELECT ?1, id, name, duration_minutes, price_cents
             FROM services WHERE id = ?2",
            params![appointment_id, service_id],
        )?;
        if changed != 1 {
            return Err(AppError::message("prestation introuvable"));
        }
    }
    Ok(())
}

pub fn has_overlap(
    conn: &Connection,
    employee_id: i64,
    starts_at: &str,
    ends_at: &str,
    ignore_id: Option<i64>,
) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM appointments
         WHERE employee_id = ?1
           AND status NOT IN ('cancelled', 'no_show')
           AND (?2 IS NULL OR id != ?2)
           AND starts_at < ?4
           AND ends_at > ?3",
        params![employee_id, ignore_id, starts_at, ends_at],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn insert(
    conn: &Connection,
    client_id: i64,
    employee_id: i64,
    starts_at: &str,
    ends_at: &str,
    status: &str,
    notes: &str,
) -> Result<i64, AppError> {
    let stamp = now();
    conn.execute(
        "INSERT INTO appointments (
            client_id, employee_id, starts_at, ends_at, status, notes, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            client_id,
            employee_id,
            starts_at,
            ends_at,
            status,
            blank_to_none(notes),
            stamp
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    client_id: i64,
    employee_id: i64,
    starts_at: &str,
    ends_at: &str,
    status: &str,
    notes: &str,
) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE appointments
         SET client_id = ?1, employee_id = ?2, starts_at = ?3, ends_at = ?4,
             status = ?5, notes = ?6, updated_at = ?7
         WHERE id = ?8",
        params![
            client_id,
            employee_id,
            starts_at,
            ends_at,
            status,
            blank_to_none(notes),
            now(),
            id
        ],
    )?;
    if updated != 1 {
        return Err(AppError::message("rendez-vous introuvable"));
    }
    Ok(())
}

pub fn set_status(conn: &Connection, id: i64, status: &str) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE appointments SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status, now(), id],
    )?;
    if updated != 1 {
        return Err(AppError::message("rendez-vous introuvable"));
    }
    Ok(())
}

fn attach_services(conn: &Connection, appointments: &mut [Appointment]) -> Result<(), AppError> {
    if appointments.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = appointments.iter().map(|item| item.id).collect();
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT appointment_id, service_id, name_snapshot, duration_minutes, price_cents
         FROM appointment_services
         WHERE appointment_id IN ({placeholders})
         ORDER BY id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let params = rusqlite::params_from_iter(ids.iter());
    let mut grouped: HashMap<i64, Vec<AppointmentServiceLine>> = HashMap::new();
    let mut rows = stmt.query(params)?;
    while let Some(row) = rows.next()? {
        let appointment_id: i64 = row.get(0)?;
        grouped
            .entry(appointment_id)
            .or_default()
            .push(AppointmentServiceLine {
                service_id: row.get(1)?,
                name: row.get(2)?,
                duration_minutes: row.get(3)?,
                price_cents: row.get(4)?,
            });
    }
    for appointment in appointments.iter_mut() {
        appointment.services = grouped.remove(&appointment.id).unwrap_or_default();
    }
    Ok(())
}

fn map_head(row: &rusqlite::Row<'_>) -> rusqlite::Result<Appointment> {
    Ok(Appointment {
        id: row.get(0)?,
        client_id: row.get(1)?,
        client_name: row.get(2)?,
        employee_id: row.get(3)?,
        employee_name: row.get(4)?,
        employee_color: row.get(5)?,
        starts_at: row.get(6)?,
        ends_at: row.get(7)?,
        status: row.get(8)?,
        notes: row.get(9)?,
        services: Vec::new(),
    })
}
