// src-tauri/src/vector_db/mod.rs

use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub text: String,
    pub file_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub record: VectorRecord,
    pub similarity: f32,
}

pub struct VectorDatabase {
    storage_path: Option<PathBuf>,
    records: Vec<VectorRecord>,
}

impl VectorDatabase {
    pub fn new(storage_path: Option<PathBuf>) -> Self {
        VectorDatabase {
            storage_path,
            records: Vec::new(),
        }
    }

    pub fn insert(
        &mut self,
        id: String,
        vector: Vec<f32>,
        text: String,
        file_path: String,
    ) -> Result<(), String> {
        let normalized_vector = normalize_vector(&vector);
        let record = VectorRecord {
            id: id.clone(),
            vector: normalized_vector,
            text,
            file_path,
        };

        if let Some(position) = self.records.iter().position(|item| item.id == id) {
            self.records[position] = record;
        } else {
            self.records.push(record);
        }

        self.save()?;
        Ok(())
    }

    pub fn search(&self, query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>, String> {
        if self.records.is_empty() {
            return Ok(Vec::new());
        }

        let normalized_query = normalize_vector(query_vector);
        let query_dimensions = normalized_query.len();
        let mut results = Vec::new();

        for record in &self.records {
            if record.vector.len() != query_dimensions {
                return Err(format!(
                    "Dimension mismatch: query vector has {} dimensions, but database record '{}' has {} dimensions",
                    query_dimensions, record.id, record.vector.len()
                ));
            }

            let similarity = calculate_dot_product(&normalized_query, &record.vector);
            results.push(SearchResult {
                record: record.clone(),
                similarity,
            });
        }

        results.sort_by(|first, second| {
            second
                .similarity
                .partial_cmp(&first.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if results.len() > limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    pub fn clear(&mut self) -> Result<(), String> {
        self.records.clear();
        self.save()?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|error| format!("Failed to read database file: {error}"))?;
            self.records = serde_json::from_str(&content)
                .map_err(|error| format!("Failed to deserialize database content: {error}"))?;
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let Some(ref path) = self.storage_path else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create database parent directories: {error}"))?;
        }

        let content = serde_json::to_string_pretty(&self.records)
            .map_err(|error| format!("Failed to serialize database: {error}"))?;

        let temporary_path = path.with_extension("tmp");
        std::fs::write(&temporary_path, &content)
            .map_err(|error| format!("Failed to write temporary database file: {error}"))?;

        std::fs::rename(temporary_path, path)
            .map_err(|error| format!("Failed to rename database file: {error}"))?;

        Ok(())
    }
}

fn calculate_dot_product(first: &[f32], second: &[f32]) -> f32 {
    first
        .iter()
        .zip(second)
        .map(|(left, right)| left * right)
        .sum()
}

fn normalize_vector(vector: &[f32]) -> Vec<f32> {
    let sum_of_squares: f32 = vector.iter().map(|value| value * value).sum();
    let magnitude = sum_of_squares.sqrt();

    if magnitude > 0.0 {
        vector.iter().map(|value| value / magnitude).collect()
    } else {
        vector.to_vec()
    }
}

pub struct VectorDbState(pub std::sync::RwLock<VectorDatabase>);

#[tauri::command]
pub fn vector_db_insert(
    state: State<'_, VectorDbState>,
    id: String,
    vector: Vec<f32>,
    text: String,
    file_path: String,
) -> Result<(), String> {
    let mut database = state.0.write().map_err(|error| error.to_string())?;
    database.insert(id, vector, text, file_path)
}

#[tauri::command]
pub fn vector_db_search(
    state: State<'_, VectorDbState>,
    query_vector: Vec<f32>,
    limit: usize,
) -> Result<Vec<SearchResult>, String> {
    let database = state.0.read().map_err(|error| error.to_string())?;
    database.search(&query_vector, limit)
}

#[tauri::command]
pub fn vector_db_clear(state: State<'_, VectorDbState>) -> Result<(), String> {
    let mut database = state.0.write().map_err(|error| error.to_string())?;
    database.clear()
}

#[tauri::command]
pub fn vector_db_load(state: State<'_, VectorDbState>) -> Result<(), String> {
    let mut database = state.0.write().map_err(|error| error.to_string())?;
    database.load()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_normalization_and_dot_product() {
        let first = vec![3.0, 4.0];
        let second = vec![0.0, 5.0];

        let normalized_first = normalize_vector(&first);
        let normalized_second = normalize_vector(&second);

        assert!((normalized_first[0] - 0.6).abs() < 1e-5);
        assert!((normalized_first[1] - 0.8).abs() < 1e-5);
        assert!((normalized_second[0] - 0.0).abs() < 1e-5);
        assert!((normalized_second[1] - 1.0).abs() < 1e-5);

        let similarity = calculate_dot_product(&normalized_first, &normalized_second);
        assert!((similarity - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_insert_search_and_clear() {
        let temp_file = NamedTempFile::new().unwrap();
        let database_path = temp_file.path().to_path_buf();

        let mut database = VectorDatabase::new(Some(database_path.clone()));

        database
            .insert(
                "doc_1".to_string(),
                vec![1.0, 0.0, 0.0],
                "Hello world".to_string(),
                "/path/hello.txt".to_string(),
            )
            .unwrap();

        database
            .insert(
                "doc_2".to_string(),
                vec![0.0, 1.0, 0.0],
                "Rust coding".to_string(),
                "/path/rust.txt".to_string(),
            )
            .unwrap();

        let results = database.search(&[0.9, 0.1, 0.0], 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record.id, "doc_1");
        assert!(results[0].similarity > 0.9);

        database.clear().unwrap();
        let empty_results = database.search(&[1.0, 0.0, 0.0], 1).unwrap();
        assert_eq!(empty_results.len(), 0);
    }

    #[test]
    fn test_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let database_path = temp_file.path().to_path_buf();

        {
            let mut database = VectorDatabase::new(Some(database_path.clone()));
            database
                .insert(
                    "persistent_doc".to_string(),
                    vec![0.5, 0.5, 0.5],
                    "Persistent text".to_string(),
                    "/path/persistent.txt".to_string(),
                )
                .unwrap();
        }

        {
            let mut loaded_database = VectorDatabase::new(Some(database_path));
            loaded_database.load().unwrap();

            let results = loaded_database.search(&[0.5, 0.5, 0.5], 1).unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].record.id, "persistent_doc");
            assert_eq!(results[0].record.text, "Persistent text");
        }
    }

    #[test]
    fn test_mass_search_performance_10k_vectors() {
        let mut database = VectorDatabase::new(None);
        let dimensions = 1024;
        let count = 10000;

        for i in 0..count {
            let mut vector = vec![0.0; dimensions];
            vector[i % dimensions] = 1.0;
            vector[(i + 1) % dimensions] = 0.5;

            database
                .insert(
                    format!("doc_{i}"),
                    vector,
                    format!("Raw text chunk content {i}"),
                    format!("/path/to/file_{i}.txt"),
                )
                .unwrap();
        }

        let mut query = vec![0.0; dimensions];
        query[0] = 1.0;
        query[5] = 0.5;

        let start_time = std::time::Instant::now();
        let results = database.search(&query, 10).unwrap();
        let duration = start_time.elapsed();

        println!("Searched and sorted {} vectors in {:?}", count, duration);
        assert_eq!(results.len(), 10);
        assert!(
            duration.as_millis() < 50,
            "Mass search took too long: {:?}",
            duration
        );
    }
}
