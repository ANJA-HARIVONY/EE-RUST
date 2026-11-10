use rusqlite::{Connection, TransactionBehavior};

use crate::domain::{Product, StockAdjust};
use crate::error::AppError;
use crate::repo::{blank_to_none, products};

pub fn adjust(conn: &mut Connection, input: &StockAdjust) -> Result<Product, AppError> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let product = products::get(&tx, input.product_id)?;
    let (delta, kind) = match input.kind.as_str() {
        "in" => {
            if input.quantity <= 0 {
                return Err(AppError::message("la quantité entrée doit être positive"));
            }
            (input.quantity, "in")
        }
        "out" => {
            if input.quantity <= 0 {
                return Err(AppError::message("la quantité sortie doit être positive"));
            }
            if product.stock_qty < input.quantity {
                return Err(AppError::message("stock insuffisant"));
            }
            (-input.quantity, "out")
        }
        "adjustment" => {
            if input.quantity < 0 {
                return Err(AppError::message("la quantité cible ne peut pas être négative"));
            }
            let delta = input.quantity - product.stock_qty;
            if delta == 0 {
                return Ok(product);
            }
            (delta, "adjustment")
        }
        _ => return Err(AppError::message("type de mouvement inconnu")),
    };

    products::apply_delta(&tx, product.id, delta)?;
    products::insert_movement(
        &tx,
        product.id,
        kind,
        delta,
        blank_to_none(&input.reason),
        input.employee_id,
        None,
    )?;
    tx.commit()?;
    products::get(conn, product.id)
}
