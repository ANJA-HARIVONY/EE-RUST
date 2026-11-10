use crate::error::AppError;
use crate::repo::now;
use rusqlite::{Connection, OptionalExtension};

pub fn migrate(conn: &mut Connection) -> Result<(), AppError> {
    let current = current_version(conn)?;
    if current < 1 {
        let tx = conn.transaction()?;
        tx.execute_batch(include_str!("sql/001_init.sql"))?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (1, 'init', ?1)",
            [now()],
        )?;
        tx.commit()?;
    }
    Ok(())
}

fn current_version(conn: &Connection) -> Result<i64, AppError> {
    let exists: Option<String> = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Ok(0);
    }
    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    Ok(version)
}
