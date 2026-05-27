// src-tauri/src/retrieval_engine/mod.rs

use tauri::State;
use crate::sidecar::SidecarState;
use crate::vector_db::VectorDbState;
use crate::document_ingestion::embeddings::generate_embeddings;
use crate::document_ingestion::TokenEstimator;

pub mod prompt_packer;

pub use prompt_packer::{
    pack_context_prompt, estimate_string_tokens, PromptPackerConfig, PackedPrompt,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResult {
    pub id: String,
    pub text: String,
    pub file_path: String,
    pub similarity: f32,
}

#[tauri::command]
pub async fn retrieve_relevant_context(
    sidecar_state: State<'_, SidecarState>,
    db_state: State<'_, VectorDbState>,
    query: String,
    limit: usize,
    min_score: Option<f32>,
) -> Result<Vec<RetrievalResult>, String> {
    let active_port = {
        let state_guard = sidecar_state.0.lock().map_err(|error| error.to_string())?;
        state_guard
            .as_ref()
            .map(|running_sidecar| running_sidecar.port)
    };
    
    let port = active_port.ok_or_else(|| "Sidecar is not currently running. Please start the local llama.cpp server.".to_string())?;
    
    let query_vector = generate_embeddings(&query, port).await?;
    
    let db_guard = db_state.0.read().map_err(|error| error.to_string())?;
    let search_results = db_guard.search(&query_vector, limit)?;
    
    let threshold = min_score.unwrap_or(0.0);
    
    let filtered_results: Vec<RetrievalResult> = search_results
        .into_iter()
        .filter(|result| result.similarity >= threshold)
        .map(|result| RetrievalResult {
            id: result.record.id,
            text: result.record.text,
            file_path: result.record.file_path,
            similarity: result.similarity,
        })
        .collect();
        
    Ok(filtered_results)
}

#[tauri::command]
pub fn format_inference_prompt(
    query: String,
    documents: Vec<RetrievalResult>,
    max_tokens: usize,
    estimator: Option<TokenEstimator>,
    system_instruction: Option<String>,
) -> Result<PackedPrompt, String> {
    let estimator = estimator.unwrap_or(TokenEstimator::Heuristic);
    let config = PromptPackerConfig {
        max_tokens,
        estimator,
        system_instruction,
    };
    pack_context_prompt(&query, &documents, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_db::{VectorDatabase, SearchResult};
    use tempfile::NamedTempFile;

    #[test]
    fn test_retrieval_engine_vacation_policy_relevance() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();
        let mut database = VectorDatabase::new(Some(db_path));

        // Insert Human Resources documents discussing vacation policies, free days, and resting
        database
            .insert(
                "hr_vacation_1".to_string(),
                vec![0.95, 0.05, 0.0],
                "Las políticas de Recursos Humanos otorgan 15 días hábiles de vacaciones pagadas tras cumplir el primer año de servicio.".to_string(),
                "/documents/recursos_humanos/politicas_vacaciones.pdf".to_string(),
            )
            .unwrap();

        database
            .insert(
                "hr_vacation_2".to_string(),
                vec![0.90, 0.10, 0.0],
                "Los empleados de Zeldrix tienen derecho a solicitar días libres adicionales para asuntos personales con aprobación previa.".to_string(),
                "/documents/recursos_humanos/politicas_vacaciones.pdf".to_string(),
            )
            .unwrap();

        // Insert technical manuals discussing database configuration and server installation
        database
            .insert(
                "tech_manual_1".to_string(),
                vec![0.05, 0.95, 0.0],
                "Para instalar el servidor de bases de datos, ejecute apt install postgresql-15 y configure la autenticación.".to_string(),
                "/documents/tecnico/manual_servidor.pdf".to_string(),
            )
            .unwrap();

        database
            .insert(
                "tech_manual_2".to_string(),
                vec![0.0, 0.98, 0.02],
                "La sincronización de réplicas en caliente requiere configurar el parámetro hot_standby en el archivo postgresql.conf.".to_string(),
                "/documents/tecnico/manual_servidor.pdf".to_string(),
            )
            .unwrap();

        // Simulate a query vector representing "Políticas de vacaciones" (highly semantic on HR/vacation dimension)
        let query_vector = vec![1.0, 0.0, 0.0];

        // Search the vector database with a high limit to get all records
        let search_results = database.search(&query_vector, 10).unwrap();

        // Map and filter manually to mimic the command logic
        let threshold = 0.5;
        let filtered_results: Vec<SearchResult> = search_results
            .into_iter()
            .filter(|result| result.similarity >= threshold)
            .collect();

        // Verify that only the Human Resources documents are retrieved
        assert_eq!(filtered_results.len(), 2);
        
        // Verify that the top result is "hr_vacation_1" and has a high similarity score
        assert_eq!(filtered_results[0].record.id, "hr_vacation_1");
        assert!(filtered_results[0].similarity > 0.9);

        // Verify that the second result is "hr_vacation_2"
        assert_eq!(filtered_results[1].record.id, "hr_vacation_2");
        assert!(filtered_results[1].similarity > 0.8);

        // Verify that no technical manual documents are included in the filtered results
        for result in &filtered_results {
            assert!(result.record.id.starts_with("hr_vacation"));
            assert!(!result.record.id.starts_with("tech_manual"));
            assert!(result.record.file_path.contains("recursos_humanos"));
            assert!(!result.record.file_path.contains("tecnico"));
        }
    }
}
