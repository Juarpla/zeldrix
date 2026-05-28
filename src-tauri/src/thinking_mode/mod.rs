// src-tauri/src/thinking_mode/mod.rs

use crate::sidecar::SidecarState;
use regex::Regex;
use tauri::State;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThinkingModeResponse {
    pub thinking: String,
    pub acuerdos: Vec<String>,
    pub conflictos: Vec<String>,
    pub matriz: Vec<ResponsibilityItem>,
    pub full_response: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponsibilityItem {
    pub tarea: String,
    pub responsable: String,
    pub fecha_limite: String,
}

#[derive(serde::Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<serde_json::Value>,
    stream: bool,
}

/// Invoca a Gemma 4 con capacidades de Thinking estructurado y parsea la respuesta.
#[tauri::command]
pub async fn ai_analyze_thinking_mode(
    sidecar_state: State<'_, SidecarState>,
    text: String,
) -> Result<ThinkingModeResponse, String> {
    let port = {
        let guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        (*guard).as_ref().map(|sidecar| sidecar.port)
    };

    let port = match port {
        Some(p) => p,
        None => return Err("Sidecar is not currently running. Please start it first.".to_string()),
    };

    let system_prompt = "You are a professional assistant specialized in unstructured information analysis and minute structuring. \
    You MUST first execute structured thinking inside <thinking>...</thinking> XML tags to chronologically break down the information, \
    evaluate implicit responsibilities, and analyze the conversation step-by-step. \
    Then, output the final result outside the tags strictly in three lists formatted with markdown/HTML: \
    1. ACUERDOS: bulleted list of agreements \
    2. CONFLICTOS: bulleted list of conflicts/risks \
    3. MATRIZ DE RESPONSABILIDADES: HTML table with 3 columns: Tarea, Responsable, Fecha Límite. \
    Keep all text and final results in Spanish. Be professional and detailed.";

    let messages = serde_json::json!([
        {
            "role": "system",
            "content": system_prompt
        },
        {
            "role": "user",
            "content": text
        }
    ]);

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

    let request_body = ChatRequest {
        model: "gemma-4-E2B-it",
        messages: messages.as_array().unwrap().clone(),
        stream: false,
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request to llama-server failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text_err = response.text().await.unwrap_or_default();
        return Err(format!(
            "llama-server returned error {}: {}",
            status, text_err
        ));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let full_response = response_json
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "No content in response".to_string())?
        .to_string();

    // Parse the thinking tag if present, otherwise default to empty/custom
    let (thinking, rest) = parse_thinking_tag(&full_response);

    // Extract lists from the remaining text
    let acuerdos = extract_acuerdos(&rest);
    let conflictos = extract_conflictos(&rest);
    let matriz = extract_matriz(&rest);

    Ok(ThinkingModeResponse {
        thinking,
        acuerdos,
        conflictos,
        matriz,
        full_response: rest,
    })
}

fn parse_thinking_tag(full_text: &str) -> (String, String) {
    let re = Regex::new(r"(?s)<thinking>(.*?)</thinking>").unwrap();
    if let Some(caps) = re.captures(full_text) {
        let thinking = caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
        let rest = re.replace(full_text, "").trim().to_string();
        (thinking, rest)
    } else {
        // Fallback for different or missing tag format
        let re_thought = Regex::new(r"(?s)<thought>(.*?)</thought>").unwrap();
        if let Some(caps) = re_thought.captures(full_text) {
            let thinking = caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
            let rest = re_thought.replace(full_text, "").trim().to_string();
            (thinking, rest)
        } else {
            (
                "No raw thinking block detected. Standard processing applied.".to_string(),
                full_text.to_string(),
            )
        }
    }
}

fn extract_acuerdos(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let re_section = Regex::new(r"(?is)(?:ACUERDOS|1\.\s*ACUERDOS)(.*?)(?:CONFLICTOS|2\.\s*CONFLICTOS|MATRIZ|3\.\s*MATRIZ|$)").unwrap();
    if let Some(caps) = re_section.captures(text) {
        let section = caps.get(1).map_or("", |m| m.as_str());
        let re_item = Regex::new(r"(?m)^\s*[\-\*\+•]\s*(.*)$").unwrap();
        for item_cap in re_item.captures_iter(section) {
            let cleaned = item_cap
                .get(1)
                .map_or("", |m| m.as_str())
                .trim()
                .to_string();
            if !cleaned.is_empty() {
                results.push(cleaned);
            }
        }
    }
    if results.is_empty() {
        results.push("No se extrajeron acuerdos explícitos de la minuta.".to_string());
    }
    results
}

fn extract_conflictos(text: &str) -> Vec<String> {
    let mut results = Vec::new();
    let re_section = Regex::new(r"(?is)(?:CONFLICTOS|2\.\s*CONFLICTOS)(.*?)(?:MATRIZ|3\.\s*MATRIZ|ACUERDOS|1\.\s*ACUERDOS|$)").unwrap();
    if let Some(caps) = re_section.captures(text) {
        let section = caps.get(1).map_or("", |m| m.as_str());
        let re_item = Regex::new(r"(?m)^\s*[\-\*\+•]\s*(.*)$").unwrap();
        for item_cap in re_item.captures_iter(section) {
            let cleaned = item_cap
                .get(1)
                .map_or("", |m| m.as_str())
                .trim()
                .to_string();
            if !cleaned.is_empty() {
                results.push(cleaned);
            }
        }
    }
    if results.is_empty() {
        results.push("No se identificaron conflictos o riesgos de manera explícita.".to_string());
    }
    results
}

fn extract_matriz(text: &str) -> Vec<ResponsibilityItem> {
    let mut results = Vec::new();
    // Look for table rows: <tr> <td>...</td> <td>...</td> <td>...</td> </tr>
    let re_row = Regex::new(
        r"(?is)<tr>\s*<td>\s*(.*?)\s*</td>\s*<td>\s*(.*?)\s*</td>\s*<td>\s*(.*?)\s*</td>\s*</tr>",
    )
    .unwrap();
    for cap in re_row.captures_iter(text) {
        let tarea = cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
        let responsable = cap.get(2).map_or("", |m| m.as_str()).trim().to_string();
        let fecha_limite = cap.get(3).map_or("", |m| m.as_str()).trim().to_string();

        // Skip header rows if present
        if tarea.to_lowercase().contains("tarea")
            || responsable.to_lowercase().contains("responsable")
        {
            continue;
        }

        if !tarea.is_empty() {
            results.push(ResponsibilityItem {
                tarea,
                responsable,
                fecha_limite,
            });
        }
    }

    // Fallback if no HTML table is present (e.g. parsed from markdown lists in matrix section)
    if results.is_empty() {
        let re_section = Regex::new(
            r"(?is)(?:MATRIZ DE RESPONSABILIDADES|3\.\s*MATRIZ)(.*?)(?:CONFLICTOS|ACUERDOS|$)",
        )
        .unwrap();
        if let Some(caps) = re_section.captures(text) {
            let section = caps.get(1).map_or("", |m| m.as_str());
            let re_item = Regex::new(r"(?m)^\s*[\-\*\+•]\s*(.*)$").unwrap();
            for item_cap in re_item.captures_iter(section) {
                let text_line = item_cap.get(1).map_or("", |m| m.as_str());
                // Try parsing "Tarea - Responsable - Fecha" or similar
                let parts: Vec<&str> = text_line.split(" - ").collect();
                if parts.len() >= 2 {
                    results.push(ResponsibilityItem {
                        tarea: parts[0].trim().to_string(),
                        responsable: parts[1].trim().to_string(),
                        fecha_limite: parts.get(2).unwrap_or(&"Sin definir").trim().to_string(),
                    });
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_thinking_tag() {
        let raw = "<thinking>Analizando cronología del proyecto retrasado.</thinking>Esta es la minuta final.";
        let (thinking, rest) = parse_thinking_tag(raw);
        assert_eq!(thinking, "Analizando cronología del proyecto retrasado.");
        assert_eq!(rest, "Esta es la minuta final.");
    }
}
