mod migrate;

use crate::config::AppConfig;
use crate::error::AppError;
use crate::repo::{self, now};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

pub struct Db {
    conn: Mutex<Connection>,
    pub config: AppConfig,
    pub database_path: String,
}

impl Db {
    pub fn open(path: &Path, config: AppConfig) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                AppError::message(format!("impossible de créer le dossier de données : {err}"))
            })?;
        }
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        prepare(&conn)?;
        migrate::migrate(&mut conn)?;
        seed(&conn, &config)?;
        Ok(Self {
            conn: Mutex::new(conn),
            config,
            database_path: path.display().to_string(),
        })
    }

    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| AppError::message("accès à la base interrompu"))?;
        f(&mut conn)
    }
}

pub fn prepare(conn: &Connection) -> Result<(), AppError> {
    conn.query_row("PRAGMA journal_mode = WAL", [], |_| Ok(()))?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(())
}

pub fn seed(conn: &Connection, cfg: &AppConfig) -> Result<(), AppError> {
    let settings_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM salon_settings", [], |row| row.get(0))?;
    if settings_count == 0 {
        conn.execute(
            "INSERT INTO salon_settings (
                id, name, address, phone, email, siret, currency, vat_rate_bps, invoice_prefix, next_invoice_seq
             ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1)",
            rusqlite::params![
                cfg.salon_name.trim(),
                repo::blank_to_none(&cfg.salon_address),
                repo::blank_to_none(&cfg.salon_phone),
                repo::blank_to_none(&cfg.salon_email),
                repo::blank_to_none(&cfg.salon_siret),
                cfg.currency,
                cfg.vat_rate_bps,
                cfg.invoice_prefix,
            ],
        )?;
    }

    let employees: i64 =
        conn.query_row("SELECT COUNT(*) FROM employees", [], |row| row.get(0))?;
    if employees == 0 {
        let role_id: i64 = conn
            .query_row(
                "SELECT id FROM roles WHERE code = ?1",
                [&cfg.default_user_role],
                |row| row.get(0),
            )
            .map_err(|_| {
                AppError::message(format!(
                    "rôle inconnu dans DEFAULT_USER_ROLE : {}",
                    cfg.default_user_role
                ))
            })?;
        let stamp = now();
        conn.execute(
            "INSERT INTO employees (
                first_name, last_name, email, phone, role_id, color, active, notes, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, NULL, ?7, ?7)",
            rusqlite::params![
                cfg.default_user_first_name.trim(),
                cfg.default_user_last_name.trim(),
                repo::blank_to_none(&cfg.default_user_email),
                repo::blank_to_none(&cfg.default_user_phone),
                role_id,
                cfg.default_user_color,
                stamp,
            ],
        )?;
    }
    Ok(())
}
