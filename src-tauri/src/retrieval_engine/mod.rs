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
    #[serde(default)]
    pub page_number: Option<u32>,
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
            page_number: result.record.page_number,
        })
        .collect();
        
    Ok(filtered_results)
}

#[tauri::command]
pub fn get_citation_fragment(
    db_state: State<'_, VectorDbState>,
    chunk_id: String,
) -> Result<RetrievalResult, String> {
    let db_guard = db_state.0.read().map_err(|error| error.to_string())?;
    let record = db_guard
        .find_by_id(&chunk_id)
        .ok_or_else(|| format!("Citation fragment not found for chunk_id: {}", chunk_id))?;

    Ok(RetrievalResult {
        id: record.id.clone(),
        text: record.text.clone(),
        file_path: record.file_path.clone(),
        similarity: 0.0,
        page_number: record.page_number,
    })
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

        database
            .insert(
                "hr_vacation_1".to_string(),
                vec![0.95, 0.05, 0.0],
                "Las políticas de Recursos Humanos otorgan 15 días hábiles de vacaciones pagadas tras cumplir el primer año de servicio.".to_string(),
                "/documents/recursos_humanos/politicas_vacaciones.pdf".to_string(),
                Some(3),
            )
            .unwrap();

        database
            .insert(
                "hr_vacation_2".to_string(),
                vec![0.90, 0.10, 0.0],
                "Los empleados de Zeldrix tienen derecho a solicitar días libres adicionales para asuntos personales con aprobación previa.".to_string(),
                "/documents/recursos_humanos/politicas_vacaciones.pdf".to_string(),
                Some(4),
            )
            .unwrap();

        database
            .insert(
                "tech_manual_1".to_string(),
                vec![0.05, 0.95, 0.0],
                "Para instalar el servidor de bases de datos, ejecute apt install postgresql-15 y configure la autenticación.".to_string(),
                "/documents/tecnico/manual_servidor.pdf".to_string(),
                None,
            )
            .unwrap();

        database
            .insert(
                "tech_manual_2".to_string(),
                vec![0.0, 0.98, 0.02],
                "La sincronización de réplicas en caliente requiere configurar el parámetro hot_standby en el archivo postgresql.conf.".to_string(),
                "/documents/tecnico/manual_servidor.pdf".to_string(),
                None,
            )
            .unwrap();

        let query_vector = vec![1.0, 0.0, 0.0];
        let search_results = database.search(&query_vector, 10).unwrap();

        let threshold = 0.5;
        let filtered_results: Vec<SearchResult> = search_results
            .into_iter()
            .filter(|result| result.similarity >= threshold)
            .collect();

        assert_eq!(filtered_results.len(), 2);
        assert_eq!(filtered_results[0].record.id, "hr_vacation_1");
        assert!(filtered_results[0].similarity > 0.9);
        assert_eq!(filtered_results[1].record.id, "hr_vacation_2");
        assert!(filtered_results[1].similarity > 0.8);

        for result in &filtered_results {
            assert!(result.record.id.starts_with("hr_vacation"));
            assert!(!result.record.id.starts_with("tech_manual"));
            assert!(result.record.file_path.contains("recursos_humanos"));
            assert!(!result.record.file_path.contains("tecnico"));
        }
    }

    #[test]
    fn test_get_citation_fragment_returns_known_chunk() {
        let mut database = VectorDatabase::new(None);
        database
            .insert(
                "chunk_policy_01".to_string(),
                vec![0.8, 0.2],
                "El proceso de auditoría interna se realiza trimestralmente por el departamento de control.".to_string(),
                "/documents/auditoria/politicas_internas.pdf".to_string(),
                Some(7),
            )
            .unwrap();

        let found = database.find_by_id("chunk_policy_01").expect("chunk must exist");

        assert_eq!(found.id, "chunk_policy_01");
        assert_eq!(found.page_number, Some(7));
        assert!(found.text.contains("auditoría interna"));
        assert!(found.file_path.contains("politicas_internas.pdf"));
    }

    #[test]
    fn test_get_citation_fragment_missing_chunk_returns_none() {
        let database = VectorDatabase::new(None);
        let result = database.find_by_id("nonexistent_chunk_id_xyz");
        assert!(result.is_none());
    }
}

