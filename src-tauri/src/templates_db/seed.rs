use rusqlite::{Connection, Result};

/// Insert the corporate test template (seed data).
/// Returns the ID of the inserted template.
pub fn seed_test_template(conn: &Connection) -> Result<i64> {
    let variables = serde_json::json!([
        "nombre_cliente",
        "empresa",
        "fecha",
        "monto_propuesta"
    ])
    .to_string();

    conn.execute(
        "INSERT INTO templates (name, category, variables, system_prompt, base_text)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            "Carta de Presentación Corporativa",
            "comunicacion",
            &variables,
            "Eres un asistente de redacción corporativa. Genera cartas formales manteniendo el tono profesional de la empresa.",
            "Estimado/a {{nombre_cliente}}:\n\nPor medio de la presente, {{empresa}} se complace en presentar la propuesta por un monto de {{monto_propuesta}}, con fecha {{fecha}}.\n\nQuedamos a su disposición para cualquier consulta.\n\nAtentamente,\nEquipo Corporativo",
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates_db::schema::init_db;

    #[test]
    fn seed_inserts_exactly_one_row() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let id = seed_test_template(&conn).unwrap();
        assert!(id > 0, "Inserted row should have positive ID");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM templates", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "Exactly one template should be inserted");
    }

    #[test]
    fn seed_template_has_correct_name() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        seed_test_template(&conn).unwrap();

        let name: String = conn
            .query_row(
                "SELECT name FROM templates WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(name, "Carta de Presentación Corporativa");
    }
}
