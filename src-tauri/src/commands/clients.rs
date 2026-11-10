use tauri::State;

use crate::db::Db;
use crate::domain::{Client, ClientInput};
use crate::error::AppError;
use crate::repo::clients;

#[tauri::command]
pub fn clients_list(db: State<'_, Db>, query: String) -> Result<Vec<Client>, AppError> {
    db.with_conn(|conn| clients::list(conn, &query))
}

#[tauri::command]
pub fn clients_save(db: State<'_, Db>, input: ClientInput) -> Result<Client, AppError> {
    db.with_conn(|conn| clients::save(conn, &input))
}
