use crate::config::AppConfig;
use crate::db::Db;
use crate::domain::{
    AppointmentInput, ClientInput, EmployeeInput, InvoiceCreate, InvoiceLineInput, ProductInput,
    ServiceInput, StockAdjust,
};
use crate::repo::{self, clients, employees, products, services};
use crate::service::{agenda, billing, stock};

fn config() -> AppConfig {
    AppConfig {
        salon_name: "Mon Salon de Beauté".into(),
        salon_address: "12 rue des Fleurs".into(),
        salon_phone: "01 23 45 67 89".into(),
        salon_email: "contact@monsalon.fr".into(),
        salon_siret: String::new(),
        currency: "EUR".into(),
        vat_rate_bps: 2_000,
        invoice_prefix: "FAC".into(),
        default_user_first_name: "Admin".into(),
        default_user_last_name: "Salon".into(),
        default_user_email: "admin@salon.local".into(),
        default_user_phone: String::new(),
        default_user_role: "admin".into(),
        default_user_color: "#2F5F8A".into(),
        database_filename: "salon.db".into(),
        agenda_open_hour: 8,
        agenda_close_hour: 20,
        agenda_slot_minutes: 15,
        locale: "fr-FR".into(),
        low_stock_default: 3,
    }
}

fn temp_db() -> (Db, std::path::PathBuf) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("ee-salon-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir).unwrap();
    let db = Db::open(&dir.join("salon.db"), config()).unwrap();
    (db, dir)
}

#[test]
fn first_launch_seeds_env_values_and_billing_keeps_stock_consistent() {
    let (db, dir) = temp_db();
    let result = db.with_conn(|conn| {
        let settings = repo::settings::get(conn)?;
        assert_eq!(settings.name, "Mon Salon de Beauté");
        assert_eq!(settings.currency, "EUR");
        assert_eq!(settings.vat_rate_bps, 2_000);
        assert_eq!(settings.invoice_prefix, "FAC");
        assert_eq!(settings.next_invoice_seq, 1);

        let people = employees::list(conn, true)?;
        assert_eq!(people.len(), 1);
        assert_eq!(people[0].first_name, "Admin");
        assert_eq!(people[0].role_code, "admin");
        assert!(people[0].active);

        let stylist_role = employees::roles(conn)?
            .into_iter()
            .find(|role| role.code == "stylist")
            .unwrap();
        let stylist = employees::save(
            conn,
            &EmployeeInput {
                id: None,
                first_name: "Léa".into(),
                last_name: "Bernard".into(),
                email: String::new(),
                phone: String::new(),
                role_id: stylist_role.id,
                color: "#8A4B2F".into(),
                active: true,
                notes: String::new(),
            },
        )?;
        let client = clients::save(
            conn,
            &ClientInput {
                id: None,
                first_name: "Inès".into(),
                last_name: "Morel".into(),
                email: String::new(),
                phone: "06 00 00 00 00".into(),
                notes: String::new(),
            },
        )?;
        let service = services::save(
            conn,
            &ServiceInput {
                id: None,
                name: "Coupe femme".into(),
                description: String::new(),
                duration_minutes: 60,
                price_cents: 3_500,
                active: true,
            },
        )?;
        let product = products::save(
            conn,
            &ProductInput {
                id: None,
                sku: "SH-01".into(),
                name: "Shampooing".into(),
                description: String::new(),
                price_cents: 1_500,
                low_stock_threshold: 2,
                active: true,
            },
        )?;
        let product = stock::adjust(
            conn,
            &StockAdjust {
                product_id: product.id,
                kind: "in".into(),
                quantity: 5,
                reason: "réception".into(),
                employee_id: Some(people[0].id),
            },
        )?;
        assert_eq!(product.stock_qty, 5);

        let start = "2026-11-10T09:00:00Z";
        let end = "2026-11-10T10:00:00Z";
        let appointment = agenda::save(
            conn,
            &AppointmentInput {
                id: None,
                client_id: client.id,
                employee_id: stylist.id,
                starts_at: start.into(),
                ends_at: end.into(),
                status: "confirmed".into(),
                notes: String::new(),
                service_ids: vec![service.id],
            },
        )?;
        let overlap = agenda::save(
            conn,
            &AppointmentInput {
                id: None,
                client_id: client.id,
                employee_id: stylist.id,
                starts_at: "2026-11-10T09:30:00Z".into(),
                ends_at: "2026-11-10T10:30:00Z".into(),
                status: "scheduled".into(),
                notes: String::new(),
                service_ids: vec![service.id],
            },
        );
        assert!(overlap.is_err());

        let invoice = billing::create(
            conn,
            &InvoiceCreate {
                client_id: client.id,
                employee_id: Some(stylist.id),
                appointment_id: Some(appointment.id),
                notes: String::new(),
            },
        )?;
        assert_eq!(invoice.lines.len(), 1);
        assert_eq!(invoice.lines[0].line_total_cents, 3_500);

        let invoice = billing::add_line(
            conn,
            &InvoiceLineInput {
                invoice_id: invoice.id,
                kind: "product".into(),
                service_id: None,
                product_id: Some(product.id),
                quantity: 2,
            },
        )?;
        assert_eq!(invoice.subtotal_cents, 6_500);
        assert_eq!(invoice.vat_cents, 1_300);
        assert_eq!(invoice.total_cents, 7_800);

        let issued = billing::issue(conn, invoice.id)?;
        assert_eq!(issued.status, "issued");
        assert!(issued.number.starts_with("FAC-2026-"));
        let stock = products::get(conn, product.id)?;
        assert_eq!(stock.stock_qty, 3);
        let done = repo::appointments::get(conn, appointment.id)?;
        assert_eq!(done.status, "completed");

        let paid = billing::mark_paid(conn, invoice.id)?;
        assert_eq!(paid.status, "paid");
        let cancelled = billing::cancel(conn, invoice.id)?;
        assert_eq!(cancelled.status, "cancelled");
        let restored = products::get(conn, product.id)?;
        assert_eq!(restored.stock_qty, 5);
        Ok::<(), crate::error::AppError>(())
    });
    result.unwrap();

    let reopened = Db::open(&dir.join("salon.db"), config()).unwrap();
    let (employees, admins): (i64, i64) = reopened
        .with_conn(|conn| {
            let employees = conn.query_row("SELECT COUNT(*) FROM employees", [], |row| row.get(0))?;
            let admins = conn.query_row(
                "SELECT COUNT(*) FROM employees e JOIN roles r ON r.id = e.role_id WHERE r.code = 'admin'",
                [],
                |row| row.get(0),
            )?;
            Ok((employees, admins))
        })
        .unwrap();
    assert_eq!(employees, 2);
    assert_eq!(admins, 1);
    drop(db);
    drop(reopened);
    let _ = std::fs::remove_dir_all(&dir);
}
