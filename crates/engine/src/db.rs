// crates/engine/src/db.rs
//! Standalone SQLite persistence for engine state (queue, volume, shuffle, repeat).
//! Replaces legacy `crates/library` SQLite tables with a clean, single-table database.

use rusqlite::Connection;
use std::path::PathBuf;

pub fn get_db_path() -> PathBuf {
    if let Ok(p) = std::env::var("RUSTIC_DB_PATH") {
        return PathBuf::from(p);
    }
    let direct = PathBuf::from("rustic_data.db");
    if direct.exists() {
        return direct;
    }
    let parent1 = PathBuf::from("../rustic_data.db");
    if parent1.exists() {
        return parent1;
    }
    let parent2 = PathBuf::from("../../rustic_data.db");
    if parent2.exists() {
        return parent2;
    }
    direct
}

pub fn open_db() -> rusqlite::Result<Connection> {
    Connection::open(get_db_path())
}

pub fn init_db(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}
