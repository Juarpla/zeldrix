use crate::document_ingestion::TokenEstimator;
use crate::retrieval_engine::RetrievalResult;
use serde::{Deserialize, Serialize};

const DEFAULT_SYSTEM_INSTRUCTION: &str = 
    "Eres un asistente corporativo inteligente de Zeldrix. Tu tarea es responder a la pregunta del usuario utilizando únicamente la información provista en los documentos de contexto a continuación.\n\
    Si la información provista no contiene la respuesta, debes responder indicando claramente que no posees suficiente información para responder. No inventes ni asumas información fuera del contexto.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPackerConfig {
    pub max_tokens: usize,
    pub estimator: TokenEstimator,
    pub system_instruction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedPrompt {
    pub formatted_prompt: String,
    pub estimated_tokens: usize,
    pub included_documents_count: usize,
    pub omitted_documents_count: usize,
}

pub fn estimate_string_tokens(text: &str, estimator: TokenEstimator) -> usize {
    match estimator {
        TokenEstimator::Words => text.split_whitespace().count(),
        TokenEstimator::Characters { chars_per_token } => {
            let character_count = text.chars().count();
            character_count / chars_per_token.max(1)
        }
        TokenEstimator::Heuristic => {
            (text.chars().count() as f64 / 4.0).round() as usize
        }
    }
}

fn format_document(index: usize, document: &RetrievalResult) -> String {
    format!(
        "---\nDocumento [{}]: {}\nID: {}\nContenido: {}\n",
        index, document.file_path, document.id, document.text
    )
}

pub fn pack_context_prompt(
    query: &str,
    documents: &[RetrievalResult],
    config: &PromptPackerConfig,
) -> Result<PackedPrompt, String> {
    let system_instruction = config
        .system_instruction
        .as_deref()
        .unwrap_or(DEFAULT_SYSTEM_INSTRUCTION);

    let base_prompt_start = format!(
        "System: {}\n\nDocumentos de contexto provistos:\n\n",
        system_instruction
    );
    let base_prompt_end = format!("\nPregunta del usuario: {}\n", query);

    let start_tokens = estimate_string_tokens(&base_prompt_start, config.estimator);
    let end_tokens = estimate_string_tokens(&base_prompt_end, config.estimator);
    let base_tokens = start_tokens + end_tokens;

    if base_tokens > config.max_tokens {
        return Err(format!(
            "Base prompt size ({} tokens) exceeds the maximum limit of {} tokens.",
            base_tokens, config.max_tokens
        ));
    }

    let mut remaining_budget = config.max_tokens - base_tokens;
    let mut included_documents = Vec::new();
    let mut included_count = 0;
    let mut omitted_count = 0;

    for (index, doc) in documents.iter().enumerate() {
        let formatted_doc = format_document(index + 1, doc);
        let doc_tokens = estimate_string_tokens(&formatted_doc, config.estimator);

        if doc_tokens <= remaining_budget {
            remaining_budget -= doc_tokens;
            included_documents.push(formatted_doc);
            included_count += 1;
        } else {
            omitted_count = documents.len() - index;
            break;
        }
    }

    let formatted_documents = included_documents.join("\n");
    let formatted_prompt = format!(
        "{}{}{}",
        base_prompt_start, formatted_documents, base_prompt_end
    );

    let total_estimated_tokens = estimate_string_tokens(&formatted_prompt, config.estimator);

    Ok(PackedPrompt {
        formatted_prompt,
        estimated_tokens: total_estimated_tokens,
        included_documents_count: included_count,
        omitted_documents_count: omitted_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_documents() -> Vec<RetrievalResult> {
        vec![
            RetrievalResult {
                id: "doc_1".to_string(),
                text: "First document with high relevance and semantic weight.".to_string(),
                file_path: "/docs/relevance_one.pdf".to_string(),
                similarity: 0.95,
                page_number: None,
            },
            RetrievalResult {
                id: "doc_2".to_string(),
                text: "Second document content is somewhat relevant to the query.".to_string(),
                file_path: "/docs/relevance_two.pdf".to_string(),
                similarity: 0.85,
                page_number: None,
            },
            RetrievalResult {
                id: "doc_3".to_string(),
                text: "Third document has low correlation with the user intent.".to_string(),
                file_path: "/docs/relevance_three.pdf".to_string(),
                similarity: 0.70,
                page_number: None,
            },
        ]
    }

    #[test]
    fn test_prompt_packer_within_budget() {
        let documents = create_mock_documents();
        let config = PromptPackerConfig {
            max_tokens: 1000,
            estimator: TokenEstimator::Heuristic,
            system_instruction: None,
        };

        let result = pack_context_prompt("How is relevance measured?", &documents, &config).unwrap();

        assert_eq!(result.included_documents_count, 3);
        assert_eq!(result.omitted_documents_count, 0);
        assert!(result.formatted_prompt.contains("doc_1"));
        assert!(result.formatted_prompt.contains("doc_2"));
        assert!(result.formatted_prompt.contains("doc_3"));
        assert!(result.formatted_prompt.contains("relevance_one.pdf"));
        assert!(result.formatted_prompt.contains("How is relevance measured?"));
        assert!(result.estimated_tokens <= 1000);
    }

    #[test]
    fn test_prompt_packer_strict_budget_omits_lowest_similarity() {
        let documents = create_mock_documents();
        let config = PromptPackerConfig {
            max_tokens: 160,
            estimator: TokenEstimator::Heuristic,
            system_instruction: None,
        };

        let result = pack_context_prompt("How is relevance measured?", &documents, &config).unwrap();

        assert_eq!(result.included_documents_count, 1);
        assert_eq!(result.omitted_documents_count, 2);
        assert!(result.formatted_prompt.contains("doc_1"));
        assert!(!result.formatted_prompt.contains("doc_2"));
        assert!(!result.formatted_prompt.contains("doc_3"));
        assert!(result.estimated_tokens <= 160);
    }

    #[test]
    fn test_prompt_packer_exceeded_base_budget_returns_error() {
        let documents = create_mock_documents();
        let config = PromptPackerConfig {
            max_tokens: 20,
            estimator: TokenEstimator::Heuristic,
            system_instruction: None,
        };

        let result = pack_context_prompt("How is relevance measured?", &documents, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds the maximum limit"));
    }

    #[test]
    fn test_prompt_packer_custom_system_instruction() {
        let documents = create_mock_documents();
        let custom_instruction = "This is a custom corporate instruction statement.".to_string();
        let config = PromptPackerConfig {
            max_tokens: 1000,
            estimator: TokenEstimator::Heuristic,
            system_instruction: Some(custom_instruction.clone()),
        };

        let result = pack_context_prompt("How is relevance measured?", &documents, &config).unwrap();

        assert!(result.formatted_prompt.contains(&custom_instruction));
        assert!(!result.formatted_prompt.contains("Eres un asistente corporativo inteligente"));
    }
}
