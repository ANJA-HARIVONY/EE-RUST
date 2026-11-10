use super::{blank_to_none, now};
use crate::domain::money::{self, line_total};
use crate::domain::{Invoice, InvoiceLine, InvoiceSummary, SalonSettings};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

pub fn list(conn: &Connection) -> Result<Vec<InvoiceSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.number, c.first_name || ' ' || c.last_name, i.status,
                i.total_cents, i.currency, i.updated_at
         FROM invoices i
         JOIN clients c ON c.id = i.client_id
         ORDER BY i.id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(InvoiceSummary {
            id: row.get(0)?,
            number: row.get(1)?,
            client_name: row.get(2)?,
            status: row.get(3)?,
            total_cents: row.get(4)?,
            currency: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get(conn: &Connection, id: i64) -> Result<Invoice, AppError> {
    let mut invoice = conn
        .query_row(
            "SELECT i.id, i.number, i.client_id, c.first_name || ' ' || c.last_name,
                    i.employee_id,
                    CASE WHEN e.id IS NULL THEN NULL ELSE e.first_name || ' ' || e.last_name END,
                    i.status, i.issued_at, i.paid_at, i.notes, i.currency, i.vat_rate_bps,
                    i.subtotal_cents, i.vat_cents, i.total_cents
             FROM invoices i
             JOIN clients c ON c.id = i.client_id
             LEFT JOIN employees e ON e.id = i.employee_id
             WHERE i.id = ?1",
            [id],
            |row| {
                Ok(Invoice {
                    id: row.get(0)?,
                    number: row.get(1)?,
                    client_id: row.get(2)?,
                    client_name: row.get(3)?,
                    employee_id: row.get(4)?,
                    employee_name: row.get(5)?,
                    status: row.get(6)?,
                    issued_at: row.get(7)?,
                    paid_at: row.get(8)?,
                    notes: row.get(9)?,
                    currency: row.get(10)?,
                    vat_rate_bps: row.get(11)?,
                    subtotal_cents: row.get(12)?,
                    vat_cents: row.get(13)?,
                    total_cents: row.get(14)?,
                    lines: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or_else(|| AppError::message("facture introuvable"))?;
    invoice.lines = lines(conn, id)?;
    Ok(invoice)
}

pub fn lines(conn: &Connection, invoice_id: i64) -> Result<Vec<InvoiceLine>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, service_id, product_id, appointment_id, description,
                quantity, unit_price_cents, line_total_cents
         FROM invoice_lines
         WHERE invoice_id = ?1
         ORDER BY id",
    )?;
    let rows = stmt.query_map([invoice_id], |row| {
        Ok(InvoiceLine {
            id: row.get(0)?,
            kind: row.get(1)?,
            service_id: row.get(2)?,
            product_id: row.get(3)?,
            appointment_id: row.get(4)?,
            description: row.get(5)?,
            quantity: row.get(6)?,
            unit_price_cents: row.get(7)?,
            line_total_cents: row.get(8)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn insert_draft(
    conn: &Connection,
    client_id: i64,
    employee_id: Option<i64>,
    notes: &str,
    settings: &SalonSettings,
) -> Result<i64, AppError> {
    let stamp = now();
    let temporary = format!("TMP-{stamp}");
    conn.execute(
        "INSERT INTO invoices (
            number, client_id, employee_id, status, notes, currency, vat_rate_bps,
            subtotal_cents, vat_cents, total_cents, created_at, updated_at
         ) VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, 0, 0, 0, ?7, ?7)",
        params![
            temporary,
            client_id,
            employee_id,
            blank_to_none(notes),
            settings.currency,
            settings.vat_rate_bps,
            stamp
        ],
    )?;
    let id = conn.last_insert_rowid();
    conn.execute(
        "UPDATE invoices SET number = ?1 WHERE id = ?2",
        params![format!("BR-{id:04}"), id],
    )?;
    Ok(id)
}

pub fn set_notes(conn: &Connection, id: i64, notes: &str) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE invoices SET notes = ?1, updated_at = ?2 WHERE id = ?3 AND status = 'draft'",
        params![blank_to_none(notes), now(), id],
    )?;
    if updated != 1 {
        return Err(AppError::message(
            "seule une facture brouillon peut être modifiée",
        ));
    }
    Ok(())
}

pub fn add_service_line(
    conn: &Connection,
    invoice_id: i64,
    service_id: i64,
    quantity: i64,
    appointment_id: Option<i64>,
) -> Result<(), AppError> {
    let (name, price): (String, i64) = conn
        .query_row(
            "SELECT name, price_cents FROM services WHERE id = ?1",
            [service_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::message("prestation introuvable"))?;
    insert_service_line(conn, invoice_id, service_id, &name, price, quantity, appointment_id)
}

pub fn insert_service_line(
    conn: &Connection,
    invoice_id: i64,
    service_id: i64,
    name: &str,
    price: i64,
    quantity: i64,
    appointment_id: Option<i64>,
) -> Result<(), AppError> {
    let total = line_total(quantity, price)?;
    conn.execute(
        "INSERT INTO invoice_lines (
            invoice_id, kind, service_id, product_id, appointment_id, description,
            quantity, unit_price_cents, line_total_cents
         ) VALUES (?1, 'service', ?2, NULL, ?3, ?4, ?5, ?6, ?7)",
        params![invoice_id, service_id, appointment_id, name, quantity, price, total],
    )?;
    Ok(())
}

pub fn add_product_line(
    conn: &Connection,
    invoice_id: i64,
    product_id: i64,
    quantity: i64,
) -> Result<(), AppError> {
    let (name, price): (String, i64) = conn
        .query_row(
            "SELECT name, price_cents FROM products WHERE id = ?1",
            [product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::message("produit introuvable"))?;
    let total = line_total(quantity, price)?;
    conn.execute(
        "INSERT INTO invoice_lines (
            invoice_id, kind, service_id, product_id, appointment_id, description,
            quantity, unit_price_cents, line_total_cents
         ) VALUES (?1, 'product', NULL, ?2, NULL, ?3, ?4, ?5, ?6)",
        params![invoice_id, product_id, name, quantity, price, total],
    )?;
    Ok(())
}

pub fn remove_line(conn: &Connection, invoice_id: i64, line_id: i64) -> Result<(), AppError> {
    let removed = conn.execute(
        "DELETE FROM invoice_lines WHERE id = ?1 AND invoice_id = ?2",
        params![line_id, invoice_id],
    )?;
    if removed != 1 {
        return Err(AppError::message("ligne introuvable"));
    }
    Ok(())
}

pub fn recompute(conn: &Connection, invoice_id: i64, vat_rate_bps: i64) -> Result<(), AppError> {
    let subtotal: i64 = conn.query_row(
        "SELECT COALESCE(SUM(line_total_cents), 0) FROM invoice_lines WHERE invoice_id = ?1",
        [invoice_id],
        |row| row.get(0),
    )?;
    let (vat, total) = money::totals(subtotal, vat_rate_bps)?;
    conn.execute(
        "UPDATE invoices
         SET subtotal_cents = ?1, vat_cents = ?2, total_cents = ?3, vat_rate_bps = ?4, updated_at = ?5
         WHERE id = ?6",
        params![subtotal, vat, total, vat_rate_bps, now(), invoice_id],
    )?;
    Ok(())
}

pub fn appointment_already_billed(conn: &Connection, appointment_id: i64) -> Result<bool, AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM invoice_lines l
         JOIN invoices i ON i.id = l.invoice_id
         WHERE l.appointment_id = ?1 AND i.status != 'cancelled'",
        [appointment_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
