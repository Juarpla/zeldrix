use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

const HISTORY_FILE_NAME: &str = "document_versions.json";
const MAX_VERSIONS_PER_DOCUMENT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentVersion {
    pub id: String,
    pub document_id: String,
    pub created_at: String,
    pub action_label: String,
    pub previous_content: String,
    pub new_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDocumentVersionInput {
    pub document_id: String,
    pub action_label: String,
    pub previous_content: String,
    pub new_content: String,
}

fn history_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(HISTORY_FILE_NAME)
}

fn now_millis() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| error.to_string())
}

fn now_nanos() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .map_err(|error| error.to_string())
}

fn read_versions_from_path(path: &Path) -> Result<Vec<DocumentVersion>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

fn write_versions_to_path(path: &Path, versions: &[DocumentVersion]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let raw = serde_json::to_string_pretty(versions).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

pub fn save_version_at(
    app_data_dir: &Path,
    input: SaveDocumentVersionInput,
) -> Result<DocumentVersion, String> {
    let timestamp = now_millis()?;
    let unique_id = now_nanos()?;
    let version = DocumentVersion {
        id: format!("{}-{}", sanitize_document_id(&input.document_id), unique_id),
        document_id: input.document_id,
        created_at: timestamp.to_string(),
        action_label: input.action_label,
        previous_content: input.previous_content,
        new_content: input.new_content,
    };

    let path = history_path(app_data_dir);
    let mut versions = read_versions_from_path(&path)?;
    versions.push(version.clone());
    versions = prune_versions(versions);
    write_versions_to_path(&path, &versions)?;

    Ok(version)
}

pub fn list_versions_at(
    app_data_dir: &Path,
    document_id: &str,
) -> Result<Vec<DocumentVersion>, String> {
    let path = history_path(app_data_dir);
    let mut versions: Vec<DocumentVersion> = read_versions_from_path(&path)?
        .into_iter()
        .filter(|version| version.document_id == document_id)
        .collect();

    versions.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.id.cmp(&left.id))
    });
    Ok(versions)
}

fn prune_versions(mut versions: Vec<DocumentVersion>) -> Vec<DocumentVersion> {
    versions.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.id.cmp(&left.id))
    });

    let mut kept = Vec::with_capacity(versions.len());
    let mut document_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for version in versions {
        let count = document_counts
            .entry(version.document_id.clone())
            .or_insert(0);
        if *count < MAX_VERSIONS_PER_DOCUMENT {
            kept.push(version);
            *count += 1;
        }
    }

    kept
}

fn sanitize_document_id(document_id: &str) -> String {
    document_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

#[tauri::command]
pub fn document_version_save(
    app: tauri::AppHandle,
    input: SaveDocumentVersionInput,
) -> Result<DocumentVersion, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    save_version_at(&app_data_dir, input)
}

#[tauri::command]
pub fn document_version_list(
    app: tauri::AppHandle,
    document_id: String,
) -> Result<Vec<DocumentVersion>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    list_versions_at(&app_data_dir, &document_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zeldrix-{}-{}", name, now_millis().unwrap()))
    }

    #[test]
    fn save_version_persists_and_lists_newest_first() {
        let dir = unique_test_dir("history-order");

        let first = save_version_at(
            &dir,
            SaveDocumentVersionInput {
                document_id: "doc-1".to_string(),
                action_label: "Primera IA".to_string(),
                previous_content: "Antes".to_string(),
                new_content: "Despues".to_string(),
            },
        )
        .unwrap();

        let second = save_version_at(
            &dir,
            SaveDocumentVersionInput {
                document_id: "doc-1".to_string(),
                action_label: "Segunda IA".to_string(),
                previous_content: "Despues".to_string(),
                new_content: "Final".to_string(),
            },
        )
        .unwrap();

        let versions = list_versions_at(&dir, "doc-1").unwrap();

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].id, second.id);
        assert_eq!(versions[1].id, first.id);
    }

    #[test]
    fn list_versions_filters_by_document_id() {
        let dir = unique_test_dir("history-filter");

        save_version_at(
            &dir,
            SaveDocumentVersionInput {
                document_id: "doc-1".to_string(),
                action_label: "Doc 1".to_string(),
                previous_content: "A".to_string(),
                new_content: "B".to_string(),
            },
        )
        .unwrap();
        save_version_at(
            &dir,
            SaveDocumentVersionInput {
                document_id: "doc-2".to_string(),
                action_label: "Doc 2".to_string(),
                previous_content: "C".to_string(),
                new_content: "D".to_string(),
            },
        )
        .unwrap();

        let versions = list_versions_at(&dir, "doc-1").unwrap();

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].document_id, "doc-1");
    }
}
