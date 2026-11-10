use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior};

use crate::domain::{Appointment, AppointmentInput};
use crate::error::AppError;
use crate::repo::appointments::{self, STATUSES};
use crate::repo::{employees, services};

pub fn save(conn: &mut Connection, input: &AppointmentInput) -> Result<Appointment, AppError> {
    if input.service_ids.is_empty() {
        return Err(AppError::message("choisissez au moins une prestation"));
    }
    let status = if input.status.trim().is_empty() {
        "scheduled".to_string()
    } else {
        input.status.trim().to_string()
    };
    if !STATUSES.contains(&status.as_str()) {
        return Err(AppError::message("statut de rendez-vous inconnu"));
    }
    let starts = parse_time(&input.starts_at)?;
    let ends = parse_time(&input.ends_at)?;
    if ends <= starts {
        return Err(AppError::message(
            "l'heure de fin doit être après l'heure de début",
        ));
    }

    let client_exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM clients WHERE id = ?1",
            [input.client_id],
            |row| row.get(0),
        )
        .optional()?;
    if client_exists.is_none() {
        return Err(AppError::message("client introuvable"));
    }

    let employee = employees::get(conn, input.employee_id)?;
    let previous_services: HashSet<i64> = if let Some(id) = input.id {
        appointments::service_ids(conn, id)?.into_iter().collect()
    } else {
        HashSet::new()
    };
    if !employee.active {
        let same_employee = input
            .id
            .map(|id| appointments::get(conn, id).map(|apt| apt.employee_id == input.employee_id))
            .transpose()?
            .unwrap_or(false);
        if !same_employee {
            return Err(AppError::message("cet employé est inactif"));
        }
    }

    let mut seen = HashSet::new();
    for service_id in &input.service_ids {
        if !seen.insert(*service_id) {
            return Err(AppError::message(
                "une prestation ne peut figurer qu'une fois",
            ));
        }
        let service = services::get(conn, *service_id)?;
        if !service.active && !previous_services.contains(service_id) {
            return Err(AppError::message(format!(
                "la prestation « {} » est inactive",
                service.name
            )));
        }
    }

    let blocks = !matches!(status.as_str(), "cancelled" | "no_show");
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if blocks
        && appointments::has_overlap(
            &tx,
            input.employee_id,
            &input.starts_at,
            &input.ends_at,
            input.id,
        )?
    {
        return Err(AppError::message(
            "cet employé a déjà un rendez-vous sur ce créneau",
        ));
    }

    let id = if let Some(id) = input.id {
        appointments::update(
            &tx,
            id,
            input.client_id,
            input.employee_id,
            &input.starts_at,
            &input.ends_at,
            &status,
            &input.notes,
        )?;
        id
    } else {
        appointments::insert(
            &tx,
            input.client_id,
            input.employee_id,
            &input.starts_at,
            &input.ends_at,
            &status,
            &input.notes,
        )?
    };
    appointments::replace_services(&tx, id, &input.service_ids)?;
    tx.commit()?;
    appointments::get(conn, id)
}

pub fn set_status(conn: &mut Connection, id: i64, status: &str) -> Result<Appointment, AppError> {
    if !STATUSES.contains(&status) {
        return Err(AppError::message("statut de rendez-vous inconnu"));
    }
    let current = appointments::get(conn, id)?;
    let blocks = !matches!(status, "cancelled" | "no_show");
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if blocks
        && appointments::has_overlap(
            &tx,
            current.employee_id,
            &current.starts_at,
            &current.ends_at,
            Some(id),
        )?
    {
        return Err(AppError::message(
            "cet employé a déjà un rendez-vous sur ce créneau",
        ));
    }
    appointments::set_status(&tx, id, status)?;
    tx.commit()?;
    appointments::get(conn, id)
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.with_timezone(&Utc))
        .map_err(|_| AppError::message("date ou heure invalide"))
}
