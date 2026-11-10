use tauri::State;

use crate::db::Db;
use crate::domain::{Product, ProductInput, StockAdjust, StockMovement};
use crate::error::AppError;
use crate::repo::products;
use crate::service::stock;

#[tauri::command]
pub fn products_list(db: State<'_, Db>, include_inactive: bool) -> Result<Vec<Product>, AppError> {
    db.with_conn(|conn| products::list(conn, include_inactive))
}

#[tauri::command]
pub fn products_save(db: State<'_, Db>, input: ProductInput) -> Result<Product, AppError> {
    db.with_conn(|conn| products::save(conn, &input))
}

#[tauri::command]
pub fn products_set_active(db: State<'_, Db>, id: i64, active: bool) -> Result<Product, AppError> {
    db.with_conn(|conn| products::set_active(conn, id, active))
}

#[tauri::command]
pub fn stock_adjust(db: State<'_, Db>, input: StockAdjust) -> Result<Product, AppError> {
    db.with_conn(|conn| stock::adjust(conn, &input))
}

#[tauri::command]
pub fn stock_history(db: State<'_, Db>, product_id: i64) -> Result<Vec<StockMovement>, AppError> {
    db.with_conn(|conn| products::history(conn, product_id))
}
