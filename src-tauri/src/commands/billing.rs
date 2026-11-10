use tauri::State;

use crate::db::Db;
use crate::domain::{Invoice, InvoiceCreate, InvoiceLineInput, InvoiceSummary};
use crate::error::AppError;
use crate::repo::invoices;
use crate::service::billing;

#[tauri::command]
pub fn invoices_list(db: State<'_, Db>) -> Result<Vec<InvoiceSummary>, AppError> {
    db.with_conn(|conn| invoices::list(conn))
}

#[tauri::command]
pub fn invoices_get(db: State<'_, Db>, id: i64) -> Result<Invoice, AppError> {
    db.with_conn(|conn| invoices::get(conn, id))
}

#[tauri::command]
pub fn invoices_create(db: State<'_, Db>, input: InvoiceCreate) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::create(conn, &input))
}

#[tauri::command]
pub fn invoices_add_line(db: State<'_, Db>, input: InvoiceLineInput) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::add_line(conn, &input))
}

#[tauri::command]
pub fn invoices_remove_line(
    db: State<'_, Db>,
    invoice_id: i64,
    line_id: i64,
) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::remove_line(conn, invoice_id, line_id))
}

#[tauri::command]
pub fn invoices_set_notes(
    db: State<'_, Db>,
    invoice_id: i64,
    notes: String,
) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::set_notes(conn, invoice_id, &notes))
}

#[tauri::command]
pub fn invoices_issue(db: State<'_, Db>, id: i64) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::issue(conn, id))
}

#[tauri::command]
pub fn invoices_mark_paid(db: State<'_, Db>, id: i64) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::mark_paid(conn, id))
}

#[tauri::command]
pub fn invoices_cancel(db: State<'_, Db>, id: i64) -> Result<Invoice, AppError> {
    db.with_conn(|conn| billing::cancel(conn, id))
}
