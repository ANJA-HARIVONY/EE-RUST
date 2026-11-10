use tauri::State;

use crate::db::Db;
use crate::domain::{Service, ServiceInput};
use crate::error::AppError;
use crate::repo::services;

#[tauri::command]
pub fn services_list(db: State<'_, Db>, include_inactive: bool) -> Result<Vec<Service>, AppError> {
    db.with_conn(|conn| services::list(conn, include_inactive))
}

#[tauri::command]
pub fn services_save(db: State<'_, Db>, input: ServiceInput) -> Result<Service, AppError> {
    db.with_conn(|conn| services::save(conn, &input))
}

#[tauri::command]
pub fn services_set_active(db: State<'_, Db>, id: i64, active: bool) -> Result<Service, AppError> {
    db.with_conn(|conn| services::set_active(conn, id, active))
}
