use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::sidecar::{health, SidecarState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableDataType {
    String,
    Number,
    Integer,
    Boolean,
    Date,
    Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractedTable {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableColumn {
    pub name: String,
    pub data_type: TableDataType,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableCell {
    pub column: String,
    pub value: TableCellValue,
    pub raw_value: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TableCellValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Null,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<serde_json::Value>,
    stream: bool,
    temperature: f32,
    response_format: serde_json::Value,
}

#[tauri::command]
pub async fn extract_structured_table_json(
    sidecar_state: State<'_, SidecarState>,
    document_text: String,
) -> Result<String, String> {
    let port = {
        let guard = sidecar_state.0.lock().map_err(|error| error.to_string())?;
        guard.as_ref().map(|sidecar| sidecar.port)
    }
    .ok_or_else(|| "Sidecar not running. Start it first.".to_string())?;

    if !health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {port}"));
    }

    let request_body = build_structured_extraction_request(&document_text);
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/v1/chat/completions");

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|error| format!("Request to llama-server failed: {error}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("llama-server returned error {status}: {text}"));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("Failed to parse llama-server response: {error}"))?;

    let content = response_json
        .pointer("/choices/0/message/content")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "No structured content in response".to_string())?;

    normalize_table_json(content)
}

fn build_structured_extraction_request(document_text: &str) -> ChatRequest {
    let system_prompt = "You are a local data extraction engine. Extract expense records from unstructured text into the required table JSON schema. Return only JSON that matches the schema. Do not include markdown, explanations, greetings, or conversational text.";
    let user_prompt = format!(
        "Extract all expense-like records from this document. Infer concise column names in English snake_case, choose the closest data type for each column, and preserve original evidence in raw_value.\n\nDOCUMENT:\n{document_text}"
    );

    ChatRequest {
        model: "gemma-4-E2B-it".to_string(),
        messages: serde_json::json!([
            {
                "role": "system",
                "content": system_prompt
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ])
        .as_array()
        .cloned()
        .unwrap_or_default(),
        stream: false,
        temperature: 0.0,
        response_format: table_response_format_schema(),
    }
}

fn table_response_format_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "json_schema",
        "json_schema": {
            "name": "strict_expense_table",
            "strict": true,
            "schema": {
                "type": "object",
                "additionalProperties": false,
                "required": ["columns", "rows"],
                "properties": {
                    "columns": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "required": ["name", "data_type", "nullable"],
                            "properties": {
                                "name": {
                                    "type": "string",
                                    "description": "Stable English snake_case column identifier."
                                },
                                "data_type": {
                                    "type": "string",
                                    "enum": ["string", "number", "integer", "boolean", "date", "currency"]
                                },
                                "nullable": {
                                    "type": "boolean"
                                }
                            }
                        }
                    },
                    "rows": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "additionalProperties": false,
                            "required": ["cells"],
                            "properties": {
                                "cells": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "additionalProperties": false,
                                        "required": ["column", "value", "raw_value", "confidence"],
                                        "properties": {
                                            "column": {
                                                "type": "string"
                                            },
                                            "value": {
                                                "description": "Typed value matching the referenced column data_type, or null when nullable and absent.",
                                                "oneOf": [
                                                    { "type": "string" },
                                                    { "type": "number" },
                                                    { "type": "integer" },
                                                    { "type": "boolean" },
                                                    { "type": "null" }
                                                ]
                                            },
                                            "raw_value": {
                                                "type": "string",
                                                "description": "Original text span used for this value, empty only when value is null."
                                            },
                                            "confidence": {
                                                "type": "number",
                                                "minimum": 0,
                                                "maximum": 1
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn normalize_table_json(raw_content: &str) -> Result<String, String> {
    let table: ExtractedTable = serde_json::from_str(raw_content)
        .map_err(|error| format!("Model did not return valid table JSON: {error}"))?;
    validate_table(&table)?;
    serde_json::to_string(&table).map_err(|error| error.to_string())
}

fn validate_table(table: &ExtractedTable) -> Result<(), String> {
    if table.columns.is_empty() {
        return Err("Structured table must contain at least one column".to_string());
    }

    let mut column_names = HashSet::new();
    let mut column_types = HashMap::new();
    let mut nullable_columns = HashSet::new();

    for column in &table.columns {
        let name = column.name.trim();
        if name.is_empty() {
            return Err("Column names cannot be empty".to_string());
        }
        if !column_names.insert(name.to_string()) {
            return Err(format!("Duplicate column name: {name}"));
        }
        if column.nullable {
            nullable_columns.insert(name.to_string());
        }
        column_types.insert(name.to_string(), column.data_type.clone());
    }

    for (row_index, row) in table.rows.iter().enumerate() {
        validate_row(
            row_index,
            row,
            &column_names,
            &column_types,
            &nullable_columns,
        )?;
    }

    Ok(())
}

fn validate_row(
    row_index: usize,
    row: &TableRow,
    column_names: &HashSet<String>,
    column_types: &HashMap<String, TableDataType>,
    nullable_columns: &HashSet<String>,
) -> Result<(), String> {
    let mut row_columns = HashSet::new();

    for cell in &row.cells {
        if !column_names.contains(&cell.column) {
            return Err(format!(
                "Row {row_index} references unknown column {}",
                cell.column
            ));
        }
        if !row_columns.insert(cell.column.clone()) {
            return Err(format!(
                "Row {row_index} contains duplicate cell for column {}",
                cell.column
            ));
        }
        if !(0.0..=1.0).contains(&cell.confidence) {
            return Err(format!(
                "Row {row_index} column {} has confidence outside 0..1",
                cell.column
            ));
        }
        if matches!(cell.value, TableCellValue::Null) && !nullable_columns.contains(&cell.column) {
            return Err(format!(
                "Row {row_index} column {} is null but column is not nullable",
                cell.column
            ));
        }

        let data_type = column_types
            .get(&cell.column)
            .ok_or_else(|| format!("Missing type for column {}", cell.column))?;
        validate_cell_type(row_index, cell, data_type)?;
    }

    if row_columns.len() != column_names.len() {
        return Err(format!(
            "Row {row_index} must contain exactly one cell per declared column"
        ));
    }

    Ok(())
}

fn validate_cell_type(
    row_index: usize,
    cell: &TableCell,
    data_type: &TableDataType,
) -> Result<(), String> {
    let valid = match data_type {
        TableDataType::String | TableDataType::Date => {
            matches!(cell.value, TableCellValue::String(_) | TableCellValue::Null)
        }
        TableDataType::Number | TableDataType::Currency => {
            matches!(
                cell.value,
                TableCellValue::Number(_) | TableCellValue::Integer(_) | TableCellValue::Null
            )
        }
        TableDataType::Integer => {
            matches!(
                cell.value,
                TableCellValue::Integer(_) | TableCellValue::Null
            )
        }
        TableDataType::Boolean => {
            matches!(
                cell.value,
                TableCellValue::Boolean(_) | TableCellValue::Null
            )
        }
    };

    if valid {
        Ok(())
    } else {
        Err(format!(
            "Row {row_index} column {} value does not match declared type",
            cell.column
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_format_uses_strict_json_schema() {
        let schema = table_response_format_schema();

        assert_eq!(
            schema.pointer("/type").and_then(serde_json::Value::as_str),
            Some("json_schema")
        );
        assert_eq!(
            schema
                .pointer("/json_schema/strict")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert!(schema
            .pointer("/json_schema/schema/properties/columns")
            .is_some());
        assert!(schema
            .pointer("/json_schema/schema/properties/rows")
            .is_some());
    }

    #[test]
    fn normalize_table_json_returns_parseable_minified_json() {
        let raw = r#"{
            "columns": [
                { "name": "date", "data_type": "date", "nullable": false },
                { "name": "vendor", "data_type": "string", "nullable": false },
                { "name": "amount", "data_type": "currency", "nullable": false }
            ],
            "rows": [
                {
                    "cells": [
                        { "column": "date", "value": "2026-05-12", "raw_value": "12/05/2026", "confidence": 0.91 },
                        { "column": "vendor", "value": "Office Depot", "raw_value": "Office Depot", "confidence": 0.87 },
                        { "column": "amount", "value": 48.5, "raw_value": "S/ 48.50", "confidence": 0.94 }
                    ]
                }
            ]
        }"#;

        let normalized = normalize_table_json(raw).expect("valid table");
        let parsed: serde_json::Value = serde_json::from_str(&normalized).expect("parseable json");

        assert!(parsed.get("columns").is_some());
        assert!(parsed.get("rows").is_some());
    }

    #[test]
    fn normalize_table_json_rejects_conversational_residue() {
        let raw = r#"Here is the JSON:
        {
            "columns": [],
            "rows": []
        }"#;

        let error = normalize_table_json(raw).expect_err("conversational text must fail");

        assert!(error.contains("valid table JSON"));
    }

    #[test]
    fn normalize_table_json_rejects_type_mismatch() {
        let raw = r#"{
            "columns": [
                { "name": "amount", "data_type": "currency", "nullable": false }
            ],
            "rows": [
                {
                    "cells": [
                        { "column": "amount", "value": "forty", "raw_value": "forty", "confidence": 0.2 }
                    ]
                }
            ]
        }"#;

        let error = normalize_table_json(raw).expect_err("type mismatch must fail");

        assert!(error.contains("does not match declared type"));
    }
}
