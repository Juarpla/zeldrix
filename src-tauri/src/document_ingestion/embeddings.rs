use tauri::State;
use crate::sidecar::SidecarState;

#[derive(serde::Serialize)]
struct EmbeddingRequest<'a> {
    input: &'a str,
    model: &'a str,
}

#[derive(serde::Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(serde::Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

pub async fn generate_embeddings(text: &str, port: u16) -> Result<Vec<f32>, String> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/v1/embeddings", port);
    
    let request_payload = EmbeddingRequest {
        input: text,
        model: "local",
    };
    
    let http_response = client
        .post(&url)
        .json(&request_payload)
        .send()
        .await
        .map_err(|error| format!("Failed to send embedding request: {error}"))?;
        
    if !http_response.status().is_success() {
        let status_code = http_response.status();
        let error_body = http_response.text().await.unwrap_or_default();
        return Err(format!("Server returned error {status_code}: {error_body}"));
    }
    
    let parsed_response: EmbeddingResponse = http_response
        .json()
        .await
        .map_err(|error| format!("Failed to parse embedding response JSON: {error}"))?;
        
    let embedding_vector = parsed_response
        .data
        .into_iter()
        .next()
        .map(|item| item.embedding)
        .ok_or_else(|| "Embedding response data array was empty".to_string())?;
        
    Ok(embedding_vector)
}

#[tauri::command]
pub async fn get_embeddings(
    sidecar_state: State<'_, SidecarState>,
    text: String,
) -> Result<Vec<f32>, String> {
    let active_port = {
        let state_guard = sidecar_state.0.lock().map_err(|error| error.to_string())?;
        state_guard
            .as_ref()
            .map(|running_sidecar| running_sidecar.port)
    };
    
    let port = active_port.ok_or_else(|| "Sidecar is not currently running".to_string())?;
    
    generate_embeddings(&text, port).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization_of_embedding_response() {
        let raw_json_response = r#"{
            "object": "list",
            "data": [
                {
                    "object": "embedding",
                    "index": 0,
                    "embedding": [0.1, 0.2, 0.3]
                }
            ],
            "model": "local"
        }"#;

        let parsed: EmbeddingResponse = serde_json::from_str(raw_json_response).unwrap();
        assert_eq!(parsed.data.len(), 1);
        assert_eq!(parsed.data[0].embedding, vec![0.1, 0.2, 0.3]);
    }
}
