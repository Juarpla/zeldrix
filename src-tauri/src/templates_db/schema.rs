use rusqlite::{Connection, Result};

/// Initialize the templates database by creating the `templates` table.
/// Safe to call multiple times — uses `CREATE TABLE IF NOT EXISTS`.
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS templates (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT NOT NULL,
            category      TEXT NOT NULL,
            variables     TEXT NOT NULL DEFAULT '[]',
            system_prompt TEXT NOT NULL,
            base_text     TEXT NOT NULL,
            created_at    TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_templates_table() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // Verify table exists by querying sqlite_master
        let table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='templates')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(table_exists, "templates table should exist after init_db");
    }

    #[test]
    fn init_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        // Second call should not error
        init_db(&conn).unwrap();
    }
}
