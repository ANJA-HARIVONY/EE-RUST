use tauri::State;

use crate::db::Db;
use crate::domain::{Employee, EmployeeInput, Role};
use crate::error::AppError;
use crate::repo::employees;

#[tauri::command]
pub fn roles_list(db: State<'_, Db>) -> Result<Vec<Role>, AppError> {
    db.with_conn(|conn| employees::roles(conn))
}

#[tauri::command]
pub fn employees_list(db: State<'_, Db>, include_inactive: bool) -> Result<Vec<Employee>, AppError> {
    db.with_conn(|conn| employees::list(conn, include_inactive))
}

#[tauri::command]
pub fn employees_save(db: State<'_, Db>, input: EmployeeInput) -> Result<Employee, AppError> {
    db.with_conn(|conn| employees::save(conn, &input))
}

#[tauri::command]
pub fn employees_set_active(
    db: State<'_, Db>,
    id: i64,
    active: bool,
) -> Result<Employee, AppError> {
    db.with_conn(|conn| employees::set_active(conn, id, active))
}
