use super::{blank_to_none, now, require_text};
use crate::domain::{Product, ProductInput, StockMovement};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

const SELECT: &str = "
    SELECT id, sku, name, description, price_cents, stock_qty, low_stock_threshold, active
    FROM products
";

pub fn list(conn: &Connection, include_inactive: bool) -> Result<Vec<Product>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} WHERE (?1 = 1 OR active = 1) ORDER BY active DESC, name"
    ))?;
    let flag = if include_inactive { 1 } else { 0 };
    let rows = stmt.query_map([flag], map_product)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get(conn: &Connection, id: i64) -> Result<Product, AppError> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], map_product)
        .optional()?
        .ok_or_else(|| AppError::message("produit introuvable"))
}

pub fn low_stock_count(conn: &Connection) -> Result<i64, AppError> {
    let count = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE active = 1 AND stock_qty <= low_stock_threshold",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn save(conn: &Connection, input: &ProductInput) -> Result<Product, AppError> {
    let name = require_text(&input.name, "Le nom")?;
    if input.price_cents < 0 {
        return Err(AppError::message("le prix ne peut pas être négatif"));
    }
    if input.low_stock_threshold < 0 {
        return Err(AppError::message("le seuil de stock ne peut pas être négatif"));
    }
    let sku = blank_to_none(&input.sku);
    let description = blank_to_none(&input.description);
    let stamp = now();
    let active = if input.active { 1 } else { 0 };
    let id = if let Some(id) = input.id {
        let updated = conn.execute(
            "UPDATE products
             SET sku = ?1, name = ?2, description = ?3, price_cents = ?4,
                 low_stock_threshold = ?5, active = ?6, updated_at = ?7
             WHERE id = ?8",
            params![
                sku,
                name,
                description,
                input.price_cents,
                input.low_stock_threshold,
                active,
                stamp,
                id
            ],
        )?;
        if updated != 1 {
            return Err(AppError::message("produit introuvable"));
        }
        id
    } else {
        conn.execute(
            "INSERT INTO products (
                sku, name, description, price_cents, stock_qty, low_stock_threshold, active, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?7)",
            params![
                sku,
                name,
                description,
                input.price_cents,
                input.low_stock_threshold,
                active,
                stamp
            ],
        )?;
        conn.last_insert_rowid()
    };
    get(conn, id)
}

pub fn set_active(conn: &Connection, id: i64, active: bool) -> Result<Product, AppError> {
    let updated = conn.execute(
        "UPDATE products SET active = ?1, updated_at = ?2 WHERE id = ?3",
        params![if active { 1 } else { 0 }, now(), id],
    )?;
    if updated != 1 {
        return Err(AppError::message("produit introuvable"));
    }
    get(conn, id)
}

pub fn apply_delta(conn: &Connection, id: i64, delta: i64) -> Result<(), AppError> {
    let updated = conn.execute(
        "UPDATE products
         SET stock_qty = stock_qty + ?1, updated_at = ?2
         WHERE id = ?3 AND stock_qty + ?1 >= 0",
        params![delta, now(), id],
    )?;
    if updated != 1 {
        return Err(AppError::message("stock insuffisant"));
    }
    Ok(())
}

pub fn insert_movement(
    conn: &Connection,
    product_id: i64,
    kind: &str,
    quantity: i64,
    reason: Option<String>,
    employee_id: Option<i64>,
    invoice_id: Option<i64>,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO stock_movements (
            product_id, kind, quantity, reason, employee_id, invoice_id, created_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![product_id, kind, quantity, reason, employee_id, invoice_id, now()],
    )?;
    Ok(())
}

pub fn history(conn: &Connection, product_id: i64) -> Result<Vec<StockMovement>, AppError> {
    let _ = get(conn, product_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, kind, quantity, reason, created_at
         FROM stock_movements
         WHERE product_id = ?1
         ORDER BY id DESC
         LIMIT 30",
    )?;
    let rows = stmt.query_map([product_id], |row| {
        Ok(StockMovement {
            id: row.get(0)?,
            kind: row.get(1)?,
            quantity: row.get(2)?,
            reason: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn map_product(row: &rusqlite::Row<'_>) -> rusqlite::Result<Product> {
    let stock_qty: i64 = row.get(5)?;
    let threshold: i64 = row.get(6)?;
    let active: bool = row.get(7)?;
    Ok(Product {
        id: row.get(0)?,
        sku: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        price_cents: row.get(4)?,
        stock_qty,
        low_stock_threshold: threshold,
        active,
        low_stock: active && stock_qty <= threshold,
    })
}
