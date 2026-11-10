use std::collections::HashMap;

use chrono::Datelike;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

use crate::domain::{Invoice, InvoiceCreate, InvoiceLineInput};
use crate::error::AppError;
use crate::repo::{appointments, invoices, products, services, settings};
use crate::repo::now;

pub fn create(conn: &mut Connection, input: &InvoiceCreate) -> Result<Invoice, AppError> {
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
    if let Some(employee_id) = input.employee_id {
        let _ = crate::repo::employees::get(conn, employee_id)?;
    }

    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let salon = settings::get(&tx)?;
    let id = invoices::insert_draft(&tx, input.client_id, input.employee_id, &input.notes, &salon)?;

    if let Some(appointment_id) = input.appointment_id {
        import_appointment(&tx, id, input.client_id, appointment_id)?;
        invoices::recompute(&tx, id, salon.vat_rate_bps)?;
    }
    tx.commit()?;
    invoices::get(conn, id)
}

pub fn add_line(conn: &mut Connection, input: &InvoiceLineInput) -> Result<Invoice, AppError> {
    if input.quantity <= 0 {
        return Err(AppError::message("la quantité doit être positive"));
    }
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_draft(&tx, input.invoice_id)?;
    match input.kind.as_str() {
        "service" => {
            let service_id = input
                .service_id
                .ok_or_else(|| AppError::message("prestation manquante"))?;
            let service = services::get(&tx, service_id)?;
            if !service.active {
                return Err(AppError::message("cette prestation est inactive"));
            }
            invoices::add_service_line(&tx, input.invoice_id, service_id, input.quantity, None)?;
        }
        "product" => {
            let product_id = input
                .product_id
                .ok_or_else(|| AppError::message("produit manquant"))?;
            let product = products::get(&tx, product_id)?;
            if !product.active {
                return Err(AppError::message("ce produit est inactif"));
            }
            invoices::add_product_line(&tx, input.invoice_id, product_id, input.quantity)?;
        }
        _ => return Err(AppError::message("type de ligne inconnu")),
    }
    let salon = settings::get(&tx)?;
    invoices::recompute(&tx, input.invoice_id, salon.vat_rate_bps)?;
    tx.commit()?;
    invoices::get(conn, input.invoice_id)
}

pub fn remove_line(conn: &mut Connection, invoice_id: i64, line_id: i64) -> Result<Invoice, AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    ensure_draft(&tx, invoice_id)?;
    invoices::remove_line(&tx, invoice_id, line_id)?;
    let salon = settings::get(&tx)?;
    invoices::recompute(&tx, invoice_id, salon.vat_rate_bps)?;
    tx.commit()?;
    invoices::get(conn, invoice_id)
}

pub fn set_notes(conn: &Connection, invoice_id: i64, notes: &str) -> Result<Invoice, AppError> {
    invoices::set_notes(conn, invoice_id, notes)?;
    invoices::get(conn, invoice_id)
}

pub fn issue(conn: &mut Connection, invoice_id: i64) -> Result<Invoice, AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let invoice = invoices::get(&tx, invoice_id)?;
    if invoice.status != "draft" {
        return Err(AppError::message("seule une facture brouillon peut être émise"));
    }
    if invoice.lines.is_empty() {
        return Err(AppError::message("la facture ne contient aucune ligne"));
    }
    let salon = settings::get(&tx)?;
    invoices::recompute(&tx, invoice_id, salon.vat_rate_bps)?;
    let invoice = invoices::get(&tx, invoice_id)?;

    let mut needed: HashMap<i64, (i64, String)> = HashMap::new();
    for line in &invoice.lines {
        if line.kind == "product" {
            let product_id = line.product_id.ok_or_else(|| AppError::message("produit manquant"))?;
            let entry = needed
                .entry(product_id)
                .or_insert((0, line.description.clone()));
            entry.0 += line.quantity;
        }
    }
    for (product_id, (quantity, name)) in &needed {
        let product = products::get(&tx, *product_id)?;
        if product.stock_qty < *quantity {
            return Err(AppError::message(format!(
                "stock insuffisant pour « {name} »"
            )));
        }
    }

    let year = chrono::Utc::now().year();
    let number = format!("{}-{}-{:04}", salon.invoice_prefix, year, salon.next_invoice_seq);
    for (product_id, (quantity, _)) in &needed {
        products::apply_delta(&tx, *product_id, -quantity)?;
        products::insert_movement(
            &tx,
            *product_id,
            "sale",
            -quantity,
            Some(format!("vente {number}")),
            invoice.employee_id,
            Some(invoice_id),
        )?;
    }
    let stamp = now();
    tx.execute(
        "UPDATE salon_settings SET next_invoice_seq = next_invoice_seq + 1 WHERE id = 1",
        [],
    )?;
    tx.execute(
        "UPDATE invoices
         SET number = ?1, status = 'issued', issued_at = ?2, currency = ?3, updated_at = ?2
         WHERE id = ?4",
        params![number, stamp, salon.currency, invoice_id],
    )?;

    let mut seen_appointments = HashMap::<i64, ()>::new();
    for line in &invoice.lines {
        if let Some(appointment_id) = line.appointment_id {
            if seen_appointments.insert(appointment_id, ()).is_none() {
                let status: String = tx.query_row(
                    "SELECT status FROM appointments WHERE id = ?1",
                    [appointment_id],
                    |row| row.get(0),
                )?;
                if matches!(status.as_str(), "scheduled" | "confirmed" | "in_progress") {
                    appointments::set_status(&tx, appointment_id, "completed")?;
                }
            }
        }
    }
    tx.commit()?;
    invoices::get(conn, invoice_id)
}

pub fn mark_paid(conn: &Connection, invoice_id: i64) -> Result<Invoice, AppError> {
    let updated = conn.execute(
        "UPDATE invoices SET status = 'paid', paid_at = ?1, updated_at = ?1
         WHERE id = ?2 AND status = 'issued'",
        params![now(), invoice_id],
    )?;
    if updated != 1 {
        return Err(AppError::message(
            "seule une facture émise peut être marquée payée",
        ));
    }
    invoices::get(conn, invoice_id)
}

pub fn cancel(conn: &mut Connection, invoice_id: i64) -> Result<Invoice, AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let invoice = invoices::get(&tx, invoice_id)?;
    if invoice.status == "cancelled" {
        return Err(AppError::message("cette facture est déjà annulée"));
    }
    if matches!(invoice.status.as_str(), "issued" | "paid") {
        let mut restore: HashMap<i64, i64> = HashMap::new();
        for line in &invoice.lines {
            if line.kind == "product" {
                if let Some(product_id) = line.product_id {
                    *restore.entry(product_id).or_default() += line.quantity;
                }
            }
        }
        for (product_id, quantity) in restore {
            products::apply_delta(&tx, product_id, quantity)?;
            products::insert_movement(
                &tx,
                product_id,
                "in",
                quantity,
                Some(format!("annulation {}", invoice.number)),
                invoice.employee_id,
                Some(invoice_id),
            )?;
        }
    }
    tx.execute(
        "UPDATE invoices SET status = 'cancelled', updated_at = ?1 WHERE id = ?2",
        params![now(), invoice_id],
    )?;
    tx.commit()?;
    invoices::get(conn, invoice_id)
}

fn ensure_draft(conn: &Connection, invoice_id: i64) -> Result<(), AppError> {
    let status: String = conn
        .query_row(
            "SELECT status FROM invoices WHERE id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| AppError::message("facture introuvable"))?;
    if status != "draft" {
        return Err(AppError::message(
            "seule une facture brouillon peut être modifiée",
        ));
    }
    Ok(())
}

fn import_appointment(
    conn: &Connection,
    invoice_id: i64,
    client_id: i64,
    appointment_id: i64,
) -> Result<(), AppError> {
    let appointment = appointments::get(conn, appointment_id)?;
    if appointment.client_id != client_id {
        return Err(AppError::message(
            "ce rendez-vous n'appartient pas à ce client",
        ));
    }
    if matches!(appointment.status.as_str(), "cancelled" | "no_show") {
        return Err(AppError::message(
            "un rendez-vous annulé ou absent ne peut pas être facturé",
        ));
    }
    if invoices::appointment_already_billed(conn, appointment_id)? {
        return Err(AppError::message("ce rendez-vous est déjà facturé"));
    }
    if appointment.services.is_empty() {
        return Err(AppError::message("ce rendez-vous n'a aucune prestation"));
    }
    for line in &appointment.services {
        invoices::insert_service_line(
            conn,
            invoice_id,
            line.service_id,
            &line.name,
            line.price_cents,
            1,
            Some(appointment_id),
        )?;
    }
    Ok(())
}
