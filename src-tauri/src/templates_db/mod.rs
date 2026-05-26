pub mod schema;
pub mod seed;

use rusqlite::{Connection, Result as RusqliteResult};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Represents a corporate template stored in the local SQLite database.
/// `required_variables` holds dynamic tokens like `{{nombre_cliente}}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Option<i64>,
    pub name: String,
    pub category: String,
    pub required_variables: Vec<String>,
    pub system_prompt: String,
    pub base_text: String,
}

impl Template {
    fn from_row(
        id: i64,
        name: String,
        category: String,
        variables_json: String,
        system_prompt: String,
        base_text: String,
    ) -> RusqliteResult<Self> {
        let required_variables: Vec<String> =
            serde_json::from_str(&variables_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                )
            })?;

        Ok(Template {
            id: Some(id),
            name,
            category,
            required_variables,
            system_prompt,
            base_text,
        })
    }
}

/// Global state wrapping the SQLite connection for Tauri commands.
pub struct TemplateDb(pub Mutex<Connection>);

// ── Core logic (testable, takes &Connection directly) ──────────────────

/// Initialize the database schema and insert seed data.
pub fn init_and_seed(conn: &Connection) -> RusqliteResult<()> {
    schema::init_db(conn)?;
    seed::seed_test_template(conn)?;
    Ok(())
}

/// List all templates in the database.
pub fn list_templates(conn: &Connection) -> RusqliteResult<Vec<Template>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category, variables, system_prompt, base_text FROM templates",
    )?;

    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let category: String = row.get(2)?;
        let variables_json: String = row.get(3)?;
        let system_prompt: String = row.get(4)?;
        let base_text: String = row.get(5)?;

        Template::from_row(id, name, category, variables_json, system_prompt, base_text)
    })?;

    let mut templates = Vec::new();
    for t in rows {
        templates.push(t?);
    }

    Ok(templates)
}

/// Get a single template by its ID.
pub fn get_template_by_id(conn: &Connection, id: i64) -> RusqliteResult<Template> {
    let mut stmt = conn.prepare(
        "SELECT id, name, category, variables, system_prompt, base_text FROM templates WHERE id = ?1",
    )?;

    stmt.query_row([id], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let category: String = row.get(2)?;
        let variables_json: String = row.get(3)?;
        let system_prompt: String = row.get(4)?;
        let base_text: String = row.get(5)?;

        Template::from_row(id, name, category, variables_json, system_prompt, base_text)
    })
}

// ── Tauri commands (thin wrappers over core logic) ─────────────────────

/// Initialize the database: create tables and insert seed data.
#[tauri::command]
pub fn template_init(state: State<'_, TemplateDb>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    init_and_seed(&conn).map_err(|e| e.to_string())
}

/// List all templates in the database.
#[tauri::command]
pub fn template_list(state: State<'_, TemplateDb>) -> Result<Vec<Template>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    list_templates(&conn).map_err(|e| e.to_string())
}

/// Get a single template by its ID.
#[tauri::command]
pub fn template_get_by_id(
    state: State<'_, TemplateDb>,
    id: i64,
) -> Result<Template, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_template_by_id(&conn, id).map_err(|e| e.to_string())
}

// ── Unit tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn db_init_creates_table() {
        let conn = Connection::open_in_memory().unwrap();
        schema::init_db(&conn).unwrap();

        let table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='templates')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(table_exists);
    }

    #[test]
    fn seed_inserts_test_template() {
        let conn = test_conn();
        let id = seed::seed_test_template(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM templates", [], |row| row.get(0))
            .unwrap();

        assert_eq!(count, 1);
        assert!(id > 0);
    }

    #[test]
    fn read_template_by_id_returns_correct_data() {
        let conn = test_conn();
        init_and_seed(&conn).unwrap();

        let template = get_template_by_id(&conn, 1).unwrap();

        assert_eq!(template.name, "Carta de Presentación Corporativa");
        assert_eq!(template.category, "comunicacion");
        assert!(template.base_text.contains("{{nombre_cliente}}"));
        assert!(template.base_text.contains("{{empresa}}"));
        assert!(template.base_text.contains("{{monto_propuesta}}"));
        assert!(template.base_text.contains("{{fecha}}"));
    }

    #[test]
    fn template_variables_parsed_as_json_array() {
        let conn = test_conn();
        init_and_seed(&conn).unwrap();

        let template = get_template_by_id(&conn, 1).unwrap();

        assert_eq!(template.required_variables.len(), 4);
        assert!(template.required_variables.contains(&"nombre_cliente".to_string()));
        assert!(template.required_variables.contains(&"empresa".to_string()));
        assert!(template.required_variables.contains(&"fecha".to_string()));
        assert!(template.required_variables.contains(&"monto_propuesta".to_string()));
    }

    #[test]
    fn template_list_returns_all_templates() {
        let conn = test_conn();
        init_and_seed(&conn).unwrap();

        let templates = list_templates(&conn).unwrap();

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Carta de Presentación Corporativa");
    }

    #[test]
    fn get_template_by_id_returns_error_for_nonexistent() {
        let conn = test_conn();
        init_and_seed(&conn).unwrap();

        let result = get_template_by_id(&conn, 999);
        assert!(result.is_err());
    }
}
