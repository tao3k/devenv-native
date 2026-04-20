//! Lightweight `SQLite` helpers for local client-side persistence.

use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

const SQLITE_BUSY_TIMEOUT_MS: u64 = 5_000;

/// Opens one `SQLite` connection with bounded pragmatic defaults for local
/// client-side persistence.
///
/// # Errors
///
/// Returns any `SQLite` open or pragma-configuration error from `rusqlite`.
pub fn open_sqlite_connection(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    configure_sqlite_connection(&connection)?;
    Ok(connection)
}

fn configure_sqlite_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.busy_timeout(Duration::from_millis(SQLITE_BUSY_TIMEOUT_MS))?;
    connection.execute_batch(
        r"
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
",
    )?;
    Ok(())
}
