use tauri::State;

use crate::db::Db;
use crate::domain::{Bootstrap, SalonSettings, SettingsInput};
use crate::error::AppError;
use crate::repo::{employees, products, settings};

#[tauri::command]
pub fn app_bootstrap(db: State<'_, Db>) -> Result<Bootstrap, AppError> {
    let email = db.config.default_user_email.clone();
    let agenda_open_hour = db.config.agenda_open_hour;
    let agenda_close_hour = db.config.agenda_close_hour;
    let agenda_slot_minutes = db.config.agenda_slot_minutes;
    let locale = db.config.locale.clone();
    let low_stock_default = db.config.low_stock_default;
    let database_path = db.database_path.clone();
    db.with_conn(|conn| {
        Ok(Bootstrap {
            settings: settings::get(conn)?,
            current_user: employees::find_current(conn, &email)?,
            agenda_open_hour,
            agenda_close_hour,
            agenda_slot_minutes,
            locale,
            low_stock_default,
            database_path,
            low_stock_count: products::low_stock_count(conn)?,
        })
    })
}

#[tauri::command]
pub fn settings_save(db: State<'_, Db>, input: SettingsInput) -> Result<SalonSettings, AppError> {
    db.with_conn(|conn| settings::save(conn, &input))
}

#[tauri::command]
pub fn low_stock_count(db: State<'_, Db>) -> Result<i64, AppError> {
    db.with_conn(|conn| products::low_stock_count(conn))
}
