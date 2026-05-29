use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use crate::sidecar::{health, SidecarState};

pub struct CustomAutomationPresetDb(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CustomAutomationOutputType {
    Text,
    Table,
}

impl CustomAutomationOutputType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Table => "table",
        }
    }

    fn from_str(value: &str) -> PresetResult<Self> {
        match value {
            "text" => Ok(Self::Text),
            "table" => Ok(Self::Table),
            _ => Err(PresetError::Validation(
                "Output type must be text or table.".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAutomationPreset {
    pub id: i64,
    pub title: String,
    pub icon_name: String,
    pub base_prompt: String,
    pub output_type: CustomAutomationOutputType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAutomationPresetInput {
    pub title: String,
    pub icon_name: String,
    pub base_prompt: String,
    pub output_type: CustomAutomationOutputType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAutomationRunRequest {
    pub preset_id: i64,
    pub input_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAutomationRunResult {
    pub output_type: CustomAutomationOutputType,
    pub content: String,
}

#[derive(Debug)]
pub enum PresetError {
    Database(rusqlite::Error),
    Validation(String),
}

type PresetResult<T> = Result<T, PresetError>;

impl std::fmt::Display for PresetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "Database error: {}", error),
            Self::Validation(message) => write!(formatter, "{}", message),
        }
    }
}

impl From<rusqlite::Error> for PresetError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub fn open_database(path: &Path) -> PresetResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            PresetError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
        })?;
    }

    let conn = Connection::open(path)?;
    init_db(&conn)?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> PresetResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS custom_automation_presets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            icon_name TEXT NOT NULL,
            base_prompt TEXT NOT NULL,
            output_type TEXT NOT NULL CHECK(output_type IN ('text', 'table')),
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    Ok(())
}

pub fn create_custom_preset(
    conn: &Connection,
    input: CustomAutomationPresetInput,
) -> PresetResult<CustomAutomationPreset> {
    validate_preset_input(&input)?;

    conn.execute(
        "INSERT INTO custom_automation_presets (title, icon_name, base_prompt, output_type)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            input.title.trim(),
            input.icon_name.trim(),
            input.base_prompt.trim(),
            input.output_type.as_str(),
        ],
    )?;

    get_custom_preset(conn, conn.last_insert_rowid())
}

pub fn list_custom_presets(conn: &Connection) -> PresetResult<Vec<CustomAutomationPreset>> {
    let mut statement = conn.prepare(
        "SELECT id, title, icon_name, base_prompt, output_type
         FROM custom_automation_presets
         ORDER BY id ASC",
    )?;

    let rows = statement.query_map([], preset_from_row)?;
    let mut presets = Vec::new();

    for row in rows {
        presets.push(row?);
    }

    Ok(presets)
}

pub fn get_custom_preset(conn: &Connection, id: i64) -> PresetResult<CustomAutomationPreset> {
    conn.query_row(
        "SELECT id, title, icon_name, base_prompt, output_type
         FROM custom_automation_presets
         WHERE id = ?1",
        [id],
        preset_from_row,
    )
    .map_err(PresetError::from)
}

fn preset_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CustomAutomationPreset> {
    let output_type: String = row.get(4)?;
    let parsed_output_type =
        CustomAutomationOutputType::from_str(&output_type).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    error.to_string(),
                )),
            )
        })?;

    Ok(CustomAutomationPreset {
        id: row.get(0)?,
        title: row.get(1)?,
        icon_name: row.get(2)?,
        base_prompt: row.get(3)?,
        output_type: parsed_output_type,
    })
}

fn validate_preset_input(input: &CustomAutomationPresetInput) -> PresetResult<()> {
    let title = input.title.trim();
    let prompt = input.base_prompt.trim();
    let icon = input.icon_name.trim();

    if title.len() < 3 || title.len() > 80 {
        return Err(PresetError::Validation(
            "Title must be between 3 and 80 characters.".to_string(),
        ));
    }

    if prompt.len() < 12 || prompt.len() > 4_000 {
        return Err(PresetError::Validation(
            "Base prompt must be between 12 and 4000 characters.".to_string(),
        ));
    }

    let allowed_icons = ["translate", "table", "briefcase", "email", "reply"];
    if !allowed_icons.contains(&icon) {
        return Err(PresetError::Validation(
            "Icon is not part of the corporate icon set.".to_string(),
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn custom_automation_preset_create(
    state: State<'_, CustomAutomationPresetDb>,
    input: CustomAutomationPresetInput,
) -> Result<CustomAutomationPreset, String> {
    let conn = state.0.lock().map_err(|error| error.to_string())?;
    create_custom_preset(&conn, input).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn custom_automation_preset_list(
    state: State<'_, CustomAutomationPresetDb>,
) -> Result<Vec<CustomAutomationPreset>, String> {
    let conn = state.0.lock().map_err(|error| error.to_string())?;
    list_custom_presets(&conn).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn custom_automation_preset_run(
    preset_state: State<'_, CustomAutomationPresetDb>,
    sidecar_state: State<'_, SidecarState>,
    request: CustomAutomationRunRequest,
) -> Result<CustomAutomationRunResult, String> {
    if request.input_text.trim().is_empty() {
        return Err("Input text is required to run this custom preset.".to_string());
    }

    let preset = {
        let conn = preset_state.0.lock().map_err(|error| error.to_string())?;
        get_custom_preset(&conn, request.preset_id).map_err(|error| error.to_string())?
    };

    let port = {
        let guard = sidecar_state.0.lock().map_err(|error| error.to_string())?;
        (*guard).as_ref().map(|sidecar| sidecar.port)
    }
    .ok_or_else(|| "Sidecar not running. Start it first.".to_string())?;

    if !health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {}", port));
    }

    let content = run_prompt_against_sidecar(port, &preset, &request.input_text).await?;

    Ok(CustomAutomationRunResult {
        output_type: preset.output_type,
        content,
    })
}

async fn run_prompt_against_sidecar(
    port: u16,
    preset: &CustomAutomationPreset,
    input_text: &str,
) -> Result<String, String> {
    #[derive(Serialize)]
    struct ChatRequest<'a> {
        model: &'a str,
        messages: Vec<serde_json::Value>,
        stream: bool,
    }

    let format_instruction = match preset.output_type {
        CustomAutomationOutputType::Text => "Return only the final text result. Do not explain your process.",
        CustomAutomationOutputType::Table => {
            "Return only an HTML table with clear headers and rows. Do not include explanations before or after the table."
        }
    };

    let messages = serde_json::json!([
        {
            "role": "system",
            "content": format!("{}\n\n{}", preset.base_prompt, format_instruction)
        },
        {
            "role": "user",
            "content": input_text
        }
    ]);

    let request_body = ChatRequest {
        model: "gemma-4-E2B-it",
        messages: messages.as_array().cloned().unwrap_or_default(),
        stream: false,
    };

    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);
    let response = reqwest::Client::new()
        .post(url)
        .json(&request_body)
        .send()
        .await
        .map_err(|error| format!("Request to llama-server failed: {}", error))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!(
            "llama-server returned error {}: {}",
            status, error_text
        ));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse response: {}", error))?;

    response_json
        .pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| "No content in response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    fn valid_input() -> CustomAutomationPresetInput {
        CustomAutomationPresetInput {
            title: "Traducir a Aleman".to_string(),
            icon_name: "translate".to_string(),
            base_prompt: "Translate this into German with a commercial tone.".to_string(),
            output_type: CustomAutomationOutputType::Text,
        }
    }

    #[test]
    fn init_db_creates_custom_presets_table() {
        let conn = test_conn();

        let table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='custom_automation_presets')",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(table_exists);
    }

    #[test]
    fn create_custom_preset_returns_saved_row() {
        let conn = test_conn();

        let preset = create_custom_preset(&conn, valid_input()).unwrap();

        assert_eq!(preset.id, 1);
        assert_eq!(preset.title, "Traducir a Aleman");
        assert_eq!(preset.icon_name, "translate");
        assert_eq!(preset.output_type, CustomAutomationOutputType::Text);
    }

    #[test]
    fn list_custom_presets_returns_inserted_rows() {
        let conn = test_conn();
        create_custom_preset(&conn, valid_input()).unwrap();

        let presets = list_custom_presets(&conn).unwrap();

        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].title, "Traducir a Aleman");
    }

    #[test]
    fn create_custom_preset_rejects_unknown_icon() {
        let conn = test_conn();
        let mut input = valid_input();
        input.icon_name = "unknown".to_string();

        let result = create_custom_preset(&conn, input);

        assert!(matches!(result, Err(PresetError::Validation(_))));
    }

    #[test]
    fn create_custom_preset_rejects_short_prompt() {
        let conn = test_conn();
        let mut input = valid_input();
        input.base_prompt = "Too short".to_string();

        let result = create_custom_preset(&conn, input);

        assert!(matches!(result, Err(PresetError::Validation(_))));
    }
}
