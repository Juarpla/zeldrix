//! Multimodal module - Handles image and audio encoding for multimodal models
//!
//! Provides utilities for encoding media files to base64 and constructing
//! multimodal prompts for llama.cpp's OpenAI-compatible API.

pub mod types;
pub use types::*; // Re-export all types for convenience

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, State};

use crate::function_calling::{
    execute_tool_call, extract_content, extract_tool_calls, get_all_tools,
    get_system_message_with_tools, has_tool_calls, MAX_TOOL_CALL_ITERATIONS,
};
use crate::sidecar::{health, SidecarState};

/// Estado del chat multimodal
pub struct MultimodalChatState(pub Mutex<Option<MultimodalSession>>);

/// Sesión de chat multimodal activa
pub struct MultimodalSession {
    pub messages: Vec<types::MultimodalMessage>,
}

const LOCAL_MODEL_ID: &str = "gemma-4-E2B-it";
const TRANSCRIPTION_SYSTEM_PROMPT: &str = "You are a local automatic speech recognition engine specialized in Spanish executive dictations and meeting recordings. Transcribe the provided audio faithfully in Spanish. Return only the transcript text. Do not summarize, translate, add timestamps, or add explanations.";
const TRANSCRIPTION_USER_PROMPT: &str = "Transcribe this audio recording faithfully in Spanish.";

// ============================================================================
// Comandos Tauri
// ============================================================================

/// Recibe un mensaje multimodal y retorna la respuesta del modelo
/// Ahora soporta function calling para web search fallback
#[tauri::command]
pub async fn chat_complete_multimodal(
    _app: AppHandle,
    sidecar_state: State<'_, SidecarState>,
    messages: Vec<types::MultimodalMessage>,
    enable_web_search: Option<bool>,
) -> Result<String, String> {
    // Obtener el puerto del sidecar
    let port = {
        let guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        match guard.as_ref() {
            Some(sidecar) => sidecar.port,
            None => return Err("Sidecar not running. Start it first.".to_string()),
        }
    };

    // Verificar que el sidecar está saludable
    if !health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {}", port));
    }

    // Convertir mensajes al formato OpenAI
    let mut oai_messages: Vec<types::OpenAIMessage> =
        messages.into_iter().map(|m| m.to_openai()).collect();

    // Añadir system message con instrucciones de tools si web_search está habilitado
    let web_search_enabled = enable_web_search.unwrap_or(false);
    if web_search_enabled {
        let system_msg = types::OpenAIMessage {
            role: "system".to_string(),
            content: Some(vec![types::OpenAIContentPart::Text {
                text: get_system_message_with_tools(),
            }]),
            tool_calls: None,
            tool_call_id: None,
        };
        oai_messages.insert(0, system_msg);
    }

    // Preparar tools para el request
    let tools = if web_search_enabled {
        get_all_tools()
    } else {
        Vec::new()
    };

    // Client HTTP
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

    // Iteración del tool call loop
    let mut iterations = 0;

    loop {
        iterations += 1;

        // Verificar límite de iteraciones
        if iterations > MAX_TOOL_CALL_ITERATIONS {
            return Err("Max tool call iterations exceeded (infinite loop prevented)".to_string());
        }

        // Construir request con tools si están disponibles
        #[derive(serde::Serialize)]
        struct ChatRequest<'a> {
            model: &'a str,
            messages: &'a [types::OpenAIMessage],
            stream: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            tools: Option<&'a Vec<serde_json::Value>>,
        }

        let request_body = ChatRequest {
            model: "gemma-4-E2B-it",
            messages: &oai_messages,
            stream: false,
            tools: if tools.is_empty() { None } else { Some(&tools) },
        };

        // Enviar request al llama-server
        let response = client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Request to llama-server failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("llama-server returned error {}: {}", status, text));
        }

        // Parsear respuesta
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Verificar si hay tool calls
        if has_tool_calls(&response_json) {
            let tool_calls = extract_tool_calls(&response_json);

            // Extraer content del mensaje del asistente
            let assistant_content = response_json
                .pointer("/choices/0/message/content")
                .and_then(|v| v.as_str())
                .map(|s| {
                    vec![types::OpenAIContentPart::Text {
                        text: s.to_string(),
                    }]
                });

            // Añadir el mensaje del asistente a la conversación
            oai_messages.push(types::OpenAIMessage {
                role: "assistant".to_string(),
                content: assistant_content,
                tool_calls: None, // No necesitamos guardar los tool_calls después de extraerlos
                tool_call_id: None,
            });

            // Ejecutar cada tool call y añadir resultados
            for tool_call in tool_calls {
                match execute_tool_call(&tool_call).await {
                    Ok(tool_result) => {
                        // Añadir mensaje de tool result
                        let tool_msg = types::OpenAIMessage {
                            role: "tool".to_string(),
                            content: tool_result
                                .get("content")
                                .and_then(|v| v.as_str())
                                .map(|s| {
                                    vec![types::OpenAIContentPart::Text {
                                        text: s.to_string(),
                                    }]
                                }),
                            tool_calls: None,
                            tool_call_id: tool_result
                                .get("tool_call_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        };
                        oai_messages.push(tool_msg);
                    }
                    Err(e) => {
                        // Si falla, añadir mensaje de error
                        let error_msg = types::OpenAIMessage {
                            role: "tool".to_string(),
                            content: Some(vec![types::OpenAIContentPart::Text {
                                text: format!("Error executing tool: {}", e),
                            }]),
                            tool_calls: None,
                            tool_call_id: Some(tool_call.id.clone()),
                        };
                        oai_messages.push(error_msg);
                    }
                }
            }

            // Continuar el loop con los nuevos mensajes
            continue;
        }

        // No hay tool calls - extraer contenido y retornar
        if let Some(content) = extract_content(&response_json) {
            return Ok(content);
        }

        // Intentar extraer contenido del campo message.content
        if let Some(msg) = response_json.pointer("/choices/0/message") {
            if let Some(content) = msg.get("content").and_then(|v| v.as_str()) {
                return Ok(content.to_string());
            }
        }

        return Err("No content in response".to_string());
    }
}

/// Codifica un archivo de imagen a base64 para uso en prompts multimodales
#[tauri::command]
pub fn encode_image_base64(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Detectar mime type desde la extensión
    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png", // default
    };

    let base64_data = BASE64_STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    Ok(data_url)
}

/// Transcribes a local audio file through the local llama.cpp multimodal server.
#[tauri::command]
pub async fn transcribe_audio_local(
    sidecar_state: State<'_, SidecarState>,
    audio_path: String,
) -> Result<String, String> {
    let port = {
        let guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        match guard.as_ref() {
            Some(sidecar) => sidecar.port,
            None => return Err("Sidecar not running. Start it first.".to_string()),
        }
    };

    if !health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {}", port));
    }

    transcribe_audio_file(port, PathBuf::from(audio_path)).await
}

async fn transcribe_audio_file(port: u16, audio_path: PathBuf) -> Result<String, String> {
    let audio_data_url = encode_audio_file_as_data_url(&audio_path)?;
    let messages = build_transcription_messages(audio_data_url);

    #[derive(serde::Serialize)]
    struct ChatRequest<'a> {
        model: &'a str,
        messages: &'a [types::OpenAIMessage],
        stream: bool,
    }

    let request_body = ChatRequest {
        model: LOCAL_MODEL_ID,
        messages: &messages,
        stream: false,
    };

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request to local llama-server failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("llama-server returned error {}: {}", status, text));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse transcription response: {}", e))?;

    extract_chat_completion_content(&response_json)
        .map(str::trim)
        .filter(|content| !content.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| "No transcript returned from local ASR process".to_string())
}

fn build_transcription_messages(audio_data_url: String) -> Vec<types::OpenAIMessage> {
    vec![
        types::OpenAIMessage {
            role: "system".to_string(),
            content: Some(vec![types::OpenAIContentPart::Text {
                text: TRANSCRIPTION_SYSTEM_PROMPT.to_string(),
            }]),
            tool_calls: None,
            tool_call_id: None,
        },
        types::OpenAIMessage {
            role: "user".to_string(),
            content: Some(vec![
                types::OpenAIContentPart::Text {
                    text: TRANSCRIPTION_USER_PROMPT.to_string(),
                },
                types::OpenAIContentPart::AudioUrl {
                    audio_url: types::AudioUrlContent {
                        url: audio_data_url,
                    },
                },
            ]),
            tool_calls: None,
            tool_call_id: None,
        },
    ]
}

fn encode_audio_file_as_data_url(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Audio path is not a file: {}", path.display()));
    }

    let mime_type = supported_audio_mime_type(path)?;
    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read audio file: {}", e))?;
    let base64_data = BASE64_STANDARD.encode(&bytes);

    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

fn supported_audio_mime_type(path: &Path) -> Result<&'static str, String> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("wav") => Ok("audio/wav"),
        Some("mp3") => Ok("audio/mpeg"),
        Some("ogg") => Ok("audio/ogg"),
        Some("flac") => Ok("audio/flac"),
        Some("m4a") => Ok("audio/mp4"),
        Some("aac") => Ok("audio/aac"),
        Some(extension) => Err(format!("Unsupported audio format: .{}", extension)),
        None => Err("Audio file must have a supported extension".to_string()),
    }
}

fn extract_chat_completion_content(response_json: &serde_json::Value) -> Option<&str> {
    response_json
        .pointer("/choices/0/message/content")
        .and_then(|value| value.as_str())
}

/// Codifica un archivo de audio a base64 para uso en prompts multimodales
#[tauri::command]
pub fn encode_audio_base64(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    let bytes = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Detectar mime type desde la extensión
    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("wav") => "audio/wav",
        Some("mp3") => "audio/mpeg",
        Some("ogg") => "audio/ogg",
        Some("flac") => "audio/flac",
        _ => "audio/wav", // default
    };

    let base64_data = BASE64_STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    Ok(data_url)
}

/// Procesa una imagen local a través del canal multimodal para extraer su texto (OCR local)
#[tauri::command]
pub async fn process_ocr_local(
    sidecar_state: State<'_, SidecarState>,
    image_base64: String,
) -> Result<String, String> {
    // Obtener el puerto del sidecar
    let port = {
        let guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        match guard.as_ref() {
            Some(sidecar) => sidecar.port,
            None => return Err("Sidecar not running. Start it first.".to_string()),
        }
    };

    // Verificar que el sidecar está saludable
    if !health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {}", port));
    }

    // Usar la imagen base64 directamente
    let data_url = image_base64;

    // Construir mensajes en formato OpenAI
    let system_msg = types::OpenAIMessage {
        role: "system".to_string(),
        content: Some(vec![types::OpenAIContentPart::Text {
            text: "Eres un motor de OCR local de alto rendimiento. Extrae todo el texto visible en la imagen de documento, factura o recibo proporcionada con la mayor precisión posible. Devuelve ÚNICAMENTE el texto extraído estructurado de forma limpia y legible. No agregues explicaciones, introducciones ni conclusiones.".to_string(),
        }]),
        tool_calls: None,
        tool_call_id: None,
    };

    let user_msg = types::OpenAIMessage {
        role: "user".to_string(),
        content: Some(vec![
            types::OpenAIContentPart::Text {
                text: "Extrae todo el texto legible de este documento.".to_string(),
            },
            types::OpenAIContentPart::ImageUrl {
                image_url: types::ImageUrlContent {
                    url: data_url,
                    detail: Some("auto".to_string()),
                },
            },
        ]),
        tool_calls: None,
        tool_call_id: None,
    };

    let oai_messages = vec![system_msg, user_msg];

    // Client HTTP
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

    #[derive(serde::Serialize)]
    struct ChatRequest<'a> {
        model: &'a str,
        messages: &'a [types::OpenAIMessage],
        stream: bool,
    }

    let request_body = ChatRequest {
        model: "gemma-4-E2B-it",
        messages: &oai_messages,
        stream: false,
    };

    // Enviar request al llama-server
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request to llama-server failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("llama-server returned error {}: {}", status, text));
    }

    // Parsear respuesta
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Extraer contenido del asistente
    if let Some(content) = response_json
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
    {
        return Ok(content.to_string());
    }

    Err("No content returned from OCR process".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_audio_mime_type_should_accept_wav() {
        let mime_type = supported_audio_mime_type(Path::new("meeting.WAV"));

        assert_eq!(mime_type, Ok("audio/wav"));
    }

    #[test]
    fn supported_audio_mime_type_should_reject_unsupported_extension() {
        let error = supported_audio_mime_type(Path::new("meeting.txt")).unwrap_err();

        assert!(error.contains("Unsupported audio format"));
    }

    #[test]
    fn extract_chat_completion_content_should_return_assistant_content() {
        let response = serde_json::json!({
            "choices": [
                {
                    "message": {
                        "content": "Esta es la transcripcion."
                    }
                }
            ]
        });

        assert_eq!(
            extract_chat_completion_content(&response),
            Some("Esta es la transcripcion.")
        );
    }
}
