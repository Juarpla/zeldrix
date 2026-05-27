// src-tauri/src/lib.rs

mod sidecar;
mod hardware;
mod download_manager;
mod exporter;
mod multimodal;
mod function_calling;
mod templates_db;
pub mod merge_engine;
mod document_history;
mod document_ingestion;

use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

use sidecar::{RunningSidecar, SidecarState, SidecarStatus};
use hardware::{HardwareInfo, ModelRecommendation};
use download_manager::DownloadState;
use exporter::{ExportRequest, ExportResult};
use multimodal::MultimodalChatState;
use templates_db::TemplateDb;
use templates_db::{template_init, template_list, template_get_by_id};
use merge_engine::{merge, Variables};
use document_history::{document_version_list, document_version_save};
use document_ingestion::extract_document_text;

/// Start the llama.cpp sidecar server
#[tauri::command]
async fn sidecar_start(
    app: tauri::AppHandle,
    state: State<'_, SidecarState>,
) -> Result<u16, String> {
    // Check if already running
    {
        let guard = state.0.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Err("Sidecar already running".to_string());
        }
    }

    // Get binary and model paths first to validate before spawning
    let binary_path = sidecar::resolve_binary_path()?;
    let model_path = sidecar::resolve_model_path()?;
    let port = sidecar::find_available_port()?;
    let threads = sidecar::get_optimal_threads();

    // Check for multimodal projector
    let mmproj_path = sidecar::resolve_mmproj_path(&model_path);

    // Emit starting event
    let _ = app.emit("sidecar:starting", ());

    // Build command with optional mmproj
    let mut cmd = std::process::Command::new(&binary_path);
    let model_str = model_path.to_str().unwrap();
    let threads_str = threads.to_string();
    let port_str = port.to_string();

    let mut args = vec![
        "-m", model_str,
        "-c", "131072",
        "-t", &threads_str,
        "--port", &port_str,
        "--host", "127.0.0.1",
    ];

    // Add --mmproj if available
    if let Some(ref mmproj) = mmproj_path {
        args.push("--mmproj");
        args.push(mmproj.to_str().unwrap());
    }

    cmd.args(&args);

    // Spawn the process
    let child = cmd.spawn().map_err(|e| format!("Failed to start llama-server: {}", e))?;

    // Store in state
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(RunningSidecar {
            process: child,
            port,
            model_path,
        });
    }

    // Emit ready event
    let _ = app.emit("sidecar:ready", port);

    Ok(port)
}

/// Stop the llama.cpp sidecar server
#[tauri::command]
async fn sidecar_stop(
    state: State<'_, SidecarState>,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(mut sidecar) = guard.take() {
        sidecar.process.kill().map_err(|e| format!("Failed to stop sidecar: {}", e))?;
    }

    Ok(())
}

/// Check if the sidecar is healthy (responding to /v1/models)
#[tauri::command]
async fn sidecar_health(
    state: State<'_, SidecarState>,
) -> Result<bool, String> {
    // Extract port before await to release the lock
    let port = {
        let guard = state.0.lock().map_err(|e| e.to_string())?;
        (*guard).as_ref().map(|sidecar| sidecar.port)
    };

    match port {
        Some(p) => Ok(sidecar::health::check_health(p).await),
        None => Ok(false),
    }
}

/// Get current sidecar status
#[tauri::command]
fn sidecar_status(
    state: State<'_, SidecarState>,
) -> Result<SidecarStatus, String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;

    match &*guard {
        Some(s) => Ok(SidecarStatus {
            running: true,
            port: Some(s.port),
            model: Some(s.model_path.to_string_lossy().to_string()),
        }),
        None => Ok(SidecarStatus {
            running: false,
            port: None,
            model: None,
        }),
    }
}

/// Get hardware information of the system
#[tauri::command]
fn hardware_info() -> HardwareInfo {
    HardwareInfo::detect()
}

/// Get model recommendation based on hardware capabilities
#[tauri::command]
fn model_recommendation() -> ModelRecommendation {
    let info = HardwareInfo::detect();
    hardware::recommend_model(&info)
}

/// Transform text using AI based on the specified action
#[tauri::command]
async fn ai_transform_text(
    sidecar_state: State<'_, SidecarState>,
    action: String,
    text: String,
) -> Result<String, String> {
    // Obtener el puerto del sidecar - extract before await to release lock
    let port = {
        let guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        (*guard).as_ref().map(|sidecar| sidecar.port)
    };

    let port = match port {
        Some(p) => p,
        None => return Err("Sidecar not running. Start it first.".to_string()),
    };

    // Verificar que el sidecar está saludable
    if !sidecar::health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {}", port));
    }

    // Mapear acción a prompt de sistema
    let system_prompt = match action.as_str() {
        "formal" => {
            "Eres un editor profesional que transforma texto informal a un estilo formal y profesional. \
            Mantén el idioma original del texto. Solo devuelve el texto transformado, sin explicaciones."
        }
        "style" => {
            "Eres un editor experto que corrige errores de estilo, gramática y puntuación. \
            Mejora la claridad y fluidez manteniendo el tono original. Solo devuelve el texto corregido, sin explicaciones."
        }
        "translate" => {
            "Eres un traductor experto. Traduce el siguiente texto al inglés estadounidense, \
            manteniendo el tono y estilo del original. Solo devuelve la traducción, sin explicaciones."
        }
        "summarize" => {
            "Eres un editor experto que resume textos manteniendo las ideas principales. \
            El resumen debe ser conciso pero completo. Solo devuelve el resumen, sin explicaciones."
        }
        _ => {
            return Err(format!("Unknown action: {}. Valid actions: formal, style, translate, summarize", action));
        }
    };

    // Construir messages para el request
    #[derive(serde::Serialize)]
    struct ChatRequest<'a> {
        model: &'a str,
        messages: Vec<serde_json::Value>,
        stream: bool,
    }

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

    // Client HTTP
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

    let request_body = ChatRequest {
        model: "gemma-4-E2B-it",
        messages: messages.as_array().unwrap().clone(),
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
        let text_err = response.text().await.unwrap_or_default();
        return Err(format!("llama-server returned error {}: {}", status, text_err));
    }

    // Parsear respuesta
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Extraer contenido
    if let Some(content) = response_json
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
    {
        Ok(content.to_string())
} else {
        Err("No content in response".to_string())
    }
}

/// Merge a template with provided variables.
///
/// Takes a template string with `{{variable}}` placeholders and a map of variable
/// values, returns the merged document with all placeholders replaced.
#[tauri::command]
fn merge_template(
    template: String,
    variables: std::collections::HashMap<String, String>,
) -> Result<String, String> {
    let vars: Variables = variables.into();
    merge(&template, &vars).map_err(|e| e.to_string())
}

/// Export editor HTML to a local corporate document format.
#[tauri::command]
async fn export_document(request: ExportRequest) -> Result<ExportResult, String> {
    exporter::export_document(request)
        .await
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .manage(SidecarState(Mutex::new(None::<RunningSidecar>)))
        .manage(DownloadState(Mutex::new(None::<download_manager::RunningDownload>)))
        .manage(MultimodalChatState(Mutex::new(None::<multimodal::MultimodalSession>)))
        .manage(TemplateDb(Mutex::new(
            rusqlite::Connection::open_in_memory().expect("failed to open templates db"),
        )))
        .invoke_handler(tauri::generate_handler![
            sidecar_start,
            sidecar_stop,
            sidecar_health,
            sidecar_status,
            hardware_info,
            model_recommendation,
            download_manager::list_models,
            download_manager::download_model,
            download_manager::cancel_download,
            download_manager::get_download_progress,
            download_manager::hot_swap_model,
            download_manager::get_model_path,
            multimodal::chat_complete_multimodal,
            multimodal::encode_image_base64,
            multimodal::encode_audio_base64,
            ai_transform_text,
            template_init,
            template_list,
            template_get_by_id,
            merge_template,
            export_document,
            document_version_save,
            document_version_list,
            extract_document_text,
        ])
        .setup(|app| {
            // Auto-start sidecar in background thread (non-blocking)
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                // Get state access
                let state = app_handle.state::<SidecarState>();

                // Emit starting event
                let _ = app_handle.emit("sidecar:starting", ());

                // Resolve paths
                let binary_path = match sidecar::resolve_binary_path() {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = app_handle.emit("sidecar:error", e);
                        return;
                    }
                };

                let model_path = match sidecar::resolve_model_path() {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = app_handle.emit("sidecar:error", e);
                        return;
                    }
                };

                let port = match sidecar::find_available_port() {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = app_handle.emit("sidecar:error", e);
                        return;
                    }
                };

                let threads = sidecar::get_optimal_threads();

                // Check for mmproj
                let mmproj_path = sidecar::resolve_mmproj_path(&model_path);

                // Build command with optional mmproj
                let mut cmd = std::process::Command::new(&binary_path);
                let threads_str = threads.to_string();
                let port_str = port.to_string();
                let mut args = vec![
                    "-m", model_path.to_str().unwrap(),
                    "-c", "131072",
                    "-t", &threads_str,
                    "--port", &port_str,
                    "--host", "127.0.0.1",
                ];

                if let Some(ref mmproj) = mmproj_path {
                    args.push("--mmproj");
                    args.push(mmproj.to_str().unwrap());
                }

                cmd.args(&args);

                // Spawn process
                match cmd.spawn() {
                    Ok(child) => {
                        // Store in state
                        let mut guard = state.0.lock().unwrap();
                        *guard = Some(RunningSidecar {
                            process: child,
                            port,
                            model_path,
                        });
                        drop(guard);

                        // Emit ready event
                        let _ = app_handle.emit("sidecar:ready", port);
                    }
                    Err(e) => {
                        let _ = app_handle.emit("sidecar:error", format!("Failed to start: {}", e));
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
