use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS people (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL,
            role       TEXT    NOT NULL DEFAULT '',
            created_at TEXT    NOT NULL
        );

        CREATE TABLE IF NOT EXISTS interruptions (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            person_id        INTEGER,
            person_name      TEXT    NOT NULL,
            start_time       TEXT    NOT NULL,
            end_time         TEXT,
            duration_seconds INTEGER,
            mouse_clicks     INTEGER,
            active_window    TEXT,
            notes            TEXT,
            created_at       TEXT    NOT NULL
        );
        ",
    )?;

    Ok(conn)
}
