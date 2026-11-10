use tauri::State;

use crate::db::Db;
use crate::domain::{Appointment, AppointmentInput};
use crate::error::AppError;
use crate::repo::appointments;
use crate::service::agenda;

#[tauri::command]
pub fn appointments_list(
    db: State<'_, Db>,
    from: String,
    to: String,
    employee_id: Option<i64>,
    client_id: Option<i64>,
) -> Result<Vec<Appointment>, AppError> {
    db.with_conn(|conn| appointments::list(conn, &from, &to, employee_id, client_id))
}

#[tauri::command]
pub fn appointments_save(
    db: State<'_, Db>,
    input: AppointmentInput,
) -> Result<Appointment, AppError> {
    db.with_conn(|conn| agenda::save(conn, &input))
}

#[tauri::command]
pub fn appointments_set_status(
    db: State<'_, Db>,
    id: i64,
    status: String,
) -> Result<Appointment, AppError> {
    db.with_conn(|conn| agenda::set_status(conn, id, &status))
}
