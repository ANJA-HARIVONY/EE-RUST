use crate::error::AppError;

/// TVA arrondie au centime le plus proche.
/// `vat_rate_bps` : 2000 = 20,00 %.
pub fn totals(subtotal_cents: i64, vat_rate_bps: i64) -> Result<(i64, i64), AppError> {
    if subtotal_cents < 0 || !(0..=10_000).contains(&vat_rate_bps) {
        return Err(AppError::message("montant ou taux de TVA invalide"));
    }
    let product = subtotal_cents
        .checked_mul(vat_rate_bps)
        .ok_or_else(|| AppError::message("montant hors limite"))?;
    let vat = product
        .checked_add(5_000)
        .ok_or_else(|| AppError::message("montant hors limite"))?
        / 10_000;
    let total = subtotal_cents
        .checked_add(vat)
        .ok_or_else(|| AppError::message("montant hors limite"))?;
    Ok((vat, total))
}

pub fn line_total(quantity: i64, unit_price_cents: i64) -> Result<i64, AppError> {
    if quantity <= 0 || unit_price_cents < 0 {
        return Err(AppError::message("quantité ou prix invalide"));
    }
    quantity
        .checked_mul(unit_price_cents)
        .ok_or_else(|| AppError::message("montant de ligne hors limite"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vat_rounds_half_up() {
        let (vat, total) = totals(10_00, 2_000).unwrap();
        assert_eq!(vat, 2_00);
        assert_eq!(total, 12_00);

        let (vat, total) = totals(199, 2_000).unwrap();
        assert_eq!(vat, 40);
        assert_eq!(total, 239);
    }
}
