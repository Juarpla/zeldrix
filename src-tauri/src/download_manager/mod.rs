//! Download Manager - Async download orchestrator with SHA-256 verification and hot-swap
//!
//! Manages model downloads from Hugging Face with:
//! - Streaming downloads with progress tracking
//! - SHA-256 hash verification
//! - Hot-swap capability for llama-server process

pub(crate) mod downloader;
pub(crate) mod hasher;
pub(crate) mod hot_swap;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::{AppHandle, Emitter, State};

use downloader::{Downloader, DownloaderHandle};
use hasher::verify_sha256;

use sidecar::{RunningSidecar, SidecarState};

use crate::sidecar;

/// Estado global del gestor de descargas
pub struct DownloadState(pub Mutex<Option<RunningDownload>>);

/// Información de la descarga en ejecución
pub struct RunningDownload {
    pub model_id: String,
    pub handle: Arc<DownloaderHandle>,
    pub started_at: Instant,
}

/// Progreso de descarga enviado al frontend
#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: f64,
    pub speed_bps: u64,
    pub status: DownloadStatus,
}

#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Completed,
    Failed { error: String },
    Cancelled,
}

/// Información de un modelo disponible para descarga
#[derive(Clone, serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub vram: String,
    pub params: String,
    pub url: String,
    pub expected_hash: String,
    pub size_bytes: u64,
    /// URL del proyector multimodal (mmproj) - solo para modelos multimodales
    pub mmproj_url: Option<String>,
    /// Hash SHA-256 del archivo mmproj
    pub mmproj_hash: Option<String>,
    /// Tamaño del archivo mmproj en bytes
    pub mmproj_size: Option<u64>,
}

/// Lista de modelos disponibles desde Unsloth
pub fn get_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gemma-4-E2B-it-IQ4_XS".to_string(),
            name: "Gemma 3 4B IT Q4_XS".to_string(),
            model_type: "chat".to_string(),
            vram: "~4GB".to_string(),
            params: "4B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/gemma-4-E2B-it-IQ4_XS.gguf".to_string(),
            expected_hash: "5cd8fb7e8f5f6e7c8f5f6e7c8f5f6e7c8f5f6e7c".to_string(), // Placeholder - real hash needed
            size_bytes: 2_500_000_000,
            // mmproj para visión y audio - mismo prefijo "mmproj-" en Hugging Face
            mmproj_url: Some("https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/mmproj-gemma-4-E2B-it-IQ4_XS.gguf".to_string()),
            mmproj_hash: Some("placeholder".to_string()),
            mmproj_size: Some(900_000_000), // ~900MB para el projector
        },
        ModelInfo {
            id: "llama-4-E2B-it-IQ4_M".to_string(),
            name: "Llama 4 Scout Q4_M".to_string(),
            model_type: "chat".to_string(),
            vram: "~8GB".to_string(),
            params: "17B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/llama-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 9_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
        ModelInfo {
            id: "mistral-4-E2B-it-IQ4_M".to_string(),
            name: "Mistral Small 3.1 Q4_M".to_string(),
            model_type: "chat".to_string(),
            vram: "~8GB".to_string(),
            params: "22B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/mistral-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 12_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
        ModelInfo {
            id: "qwen-4-E2B-it-IQ4_M".to_string(),
            name: "Qwen 3.5 Q4_M".to_string(),
            model_type: "chat".to_string(),
            vram: "~8GB".to_string(),
            params: "32B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/qwen-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 18_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
        ModelInfo {
            id: "deepseek-4-E2B-it-IQ4_M".to_string(),
            name: "Deepseek Chat Q4_M".to_string(),
            model_type: "chat".to_string(),
            vram: "~8GB".to_string(),
            params: "32B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/deepseek-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 18_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
        ModelInfo {
            id: "phi-4-E2B-it-IQ4_M".to_string(),
            name: "Phi-4 Q4_M".to_string(),
            model_type: "chat".to_string(),
            vram: "~8GB".to_string(),
            params: "14B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/phi-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 8_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
        ModelInfo {
            id: "nomic-4-E2B-it-IQ4_M".to_string(),
            name: "Nomic Embed Q4_M".to_string(),
            model_type: "embedding".to_string(),
            vram: "~4GB".to_string(),
            params: "7B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/nomic-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 4_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
        ModelInfo {
            id: "excel-4-E2B-it-IQ4_M".to_string(),
            name: "Excel LLM Q4_M".to_string(),
            model_type: "specialized".to_string(),
            vram: "~8GB".to_string(),
            params: "14B".to_string(),
            url: "https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/excel-4-E2B-it-IQ4_M.gguf".to_string(),
            expected_hash: "placeholder".to_string(),
            size_bytes: 8_000_000_000,
            mmproj_url: None,
            mmproj_hash: None,
            mmproj_size: None,
        },
    ]
}

/// Obtiene la ruta del directorio de descargas
fn get_downloads_dir() -> Result<PathBuf, String> {
    let app_dir = directories::ProjectDirs::from("com", "zenit", "zeldrix")
        .ok_or("Cannot determine app data directory")?;
    let downloads_dir = app_dir.data_dir().join("downloads");
    Ok(downloads_dir)
}

/// Obtiene la ruta del directorio de modelos
fn get_models_dir() -> Result<PathBuf, String> {
    let app_dir = directories::ProjectDirs::from("com", "zenit", "zeldrix")
        .ok_or("Cannot determine app data directory")?;
    let models_dir = app_dir.data_dir().join("models");
    Ok(models_dir)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Lista todos los modelos disponibles con su estado actual
#[tauri::command]
pub async fn list_models(
    state: State<'_, DownloadState>,
    sidecar_state: State<'_, SidecarState>,
) -> Result<Vec<ModelStatusInfo>, String> {
    let download_guard = state.0.lock().map_err(|e| e.to_string())?;
    let sidecar_guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;

    let models = get_available_models();
    let mut result = Vec::with_capacity(models.len());

    for model in models {
        let status = if download_guard.as_ref().map(|d| d.model_id == model.id).unwrap_or(false) {
            ModelStatus::Downloading
        } else if sidecar_guard.as_ref().map(|s| s.model_path.to_string_lossy().contains(&model.id)).unwrap_or(false) {
            ModelStatus::Loaded
        } else if model_exists(&model.id)? {
            ModelStatus::Downloaded
        } else {
            ModelStatus::NotDownloaded
        };

        result.push(ModelStatusInfo {
            info: model,
            status,
        });
    }

    Ok(result)
}

#[derive(Clone, serde::Serialize)]
pub struct ModelStatusInfo {
    pub info: ModelInfo,
    pub status: ModelStatus,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    NotDownloaded,
    Downloading,
    Downloaded,
    Loaded,
}

fn model_exists(model_id: &str) -> Result<bool, String> {
    let models_dir = get_models_dir()?;
    let model_path = models_dir.join(format!("{}.gguf", model_id));
    Ok(model_path.exists())
}

/// Verifica si existe el archivo mmproj para un modelo multimodal
fn mmproj_exists(model_id: &str) -> Result<bool, String> {
    let models_dir = get_models_dir()?;
    let mmproj_path = models_dir.join(format!("mmproj-{}.gguf", model_id));
    Ok(mmproj_path.exists())
}

/// Obtiene la ruta del archivo mmproj para un modelo
pub fn get_mmproj_path(model_id: &str) -> Result<Option<PathBuf>, String> {
    let models_dir = get_models_dir()?;
    let mmproj_path = models_dir.join(format!("mmproj-{}.gguf", model_id));
    if mmproj_path.exists() {
        Ok(Some(mmproj_path))
    } else {
        Ok(None)
    }
}

/// Inicia la descarga de un modelo
#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    state: State<'_, DownloadState>,
    model_id: String,
) -> Result<(), String> {
    // Verificar si ya hay una descarga activa
    {
        let guard = state.0.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Err("Another download is already in progress".to_string());
        }
    }

    // Encontrar el modelo
    let model_info = get_available_models()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Model '{}' not found", model_id))?;

    // Crear directorio de descargas si no existe
    let downloads_dir = get_downloads_dir()?;
    tokio::fs::create_dir_all(&downloads_dir).await.map_err(|e| e.to_string())?;

    // Crear directorio de modelos si no existe
    let models_dir = get_models_dir()?;
    tokio::fs::create_dir_all(&models_dir).await.map_err(|e| e.to_string())?;

    let part_path = downloads_dir.join(format!("{}.gguf.part", model_id));
    let final_path = models_dir.join(format!("{}.gguf", model_id));

    // Crear el handle y envolverlo en Arc
    let handle = DownloaderHandle::new(model_info.size_bytes);
    let handle_arc = Arc::new(handle);
    let handle_for_downloader = handle_arc.clone();

    // Crear el downloader con el handle
    let downloader = Downloader::new(
        model_info.url.clone(),
        part_path.clone(),
        model_info.size_bytes,
        handle_for_downloader,
    );

    // Guardar estado
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(RunningDownload {
            model_id: model_id.clone(),
            handle: handle_arc,
            started_at: Instant::now(),
        });
    }

    // Iniciar descarga en background
    let app_handle = app.clone();
    let model_id_clone = model_id.clone();
    let expected_hash = model_info.expected_hash.clone();
    let part_path_clone = part_path.clone();
    let final_path_clone = final_path.clone();

    tokio::spawn(async move {
        let result = downloader.download().await;

        match result {
            Ok(()) => {
                // Verificar hash SHA-256
                let _ = app_handle.emit("download:verifying", model_id_clone.clone());

                match verify_sha256(&part_path_clone, &expected_hash).await {
                    Ok(true) => {
                        // Renombrar archivo
                        if let Err(e) = tokio::fs::rename(&part_path_clone, &final_path_clone).await {
                            let _ = app_handle.emit("download:error", serde_json::json!({
                                "model_id": model_id_clone,
                                "error": format!("Failed to finalize download: {}", e)
                            }));
                            return;
                        }

                        let _ = app_handle.emit("download:complete", serde_json::json!({
                            "model_id": model_id_clone,
                            "path": final_path_clone.to_string_lossy()
                        }));
                    }
                    Ok(false) => {
                        // Hash no coincide - borrar archivo
                        let _ = tokio::fs::remove_file(&part_path_clone).await;
                        let _ = app_handle.emit("download:error", serde_json::json!({
                            "model_id": model_id_clone,
                            "error": "SHA-256 hash verification failed. File corrupted."
                        }));
                    }
                    Err(e) => {
                        let _ = app_handle.emit("download:error", serde_json::json!({
                            "model_id": model_id_clone,
                            "error": format!("Failed to verify hash: {}", e)
                        }));
                    }
                }
            }
            Err(e) => {
                let _ = app_handle.emit("download:error", serde_json::json!({
                    "model_id": model_id_clone,
                    "error": e.to_string()
                }));
            }
        }
    });

    Ok(())
}

/// Cancela la descarga en curso
#[tauri::command]
pub async fn cancel_download(
    app: AppHandle,
    state: State<'_, DownloadState>,
) -> Result<(), String> {
    let running = {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        guard.take()
    };

    if let Some(download) = running {
        download.handle.cancel();
        let _ = app.emit("download:cancelled", serde_json::json!({
            "model_id": download.model_id
        }));
    }

    Ok(())
}

/// Obtiene el progreso actual de la descarga
#[tauri::command]
pub async fn get_download_progress(
    state: State<'_, DownloadState>,
) -> Result<Option<DownloadProgress>, String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;

    if let Some(download) = guard.as_ref() {
        let progress = download.handle.get_progress();
        let elapsed = download.started_at.elapsed().as_secs_f64();
        let speed_bps = if elapsed > 0.0 {
            progress.bytes_downloaded as f64 / elapsed
        } else {
            0.0
        } as u64;

        let percentage = progress.total_bytes
            .map(|total| (progress.bytes_downloaded as f64 / total as f64) * 100.0)
            .unwrap_or(0.0);

        Ok(Some(DownloadProgress {
            model_id: download.model_id.clone(),
            bytes_downloaded: progress.bytes_downloaded,
            total_bytes: progress.total_bytes,
            percentage,
            speed_bps,
            status: if progress.is_finished {
                DownloadStatus::Completed
            } else if progress.is_cancelled {
                DownloadStatus::Cancelled
            } else {
                DownloadStatus::Downloading
            },
        }))
    } else {
        Ok(None)
    }
}

/// Realiza hot-swap del modelo en ejecución
#[tauri::command]
pub async fn hot_swap_model(
    app: AppHandle,
    sidecar_state: State<'_, SidecarState>,
    model_id: String,
) -> Result<u16, String> {
    // Verificar que el modelo existe
    let models_dir = get_models_dir()?;
    let model_path = models_dir.join(format!("{}.gguf", model_id));

    if !model_path.exists() {
        return Err(format!(
            "Model '{}' not found at {}. Download it first.",
            model_id,
            model_path.display()
        ));
    }

    // Verificar mmproj si el modelo es multimodal
    let mmproj_path = get_mmproj_path(&model_id)?;

    // Detener sidecar actual si está corriendo
    let current_port = {
        let mut guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        if let Some(mut sidecar) = guard.take() {
            let _ = app.emit("sidecar:stopping", ());
            sidecar.process.kill().map_err(|e| format!("Failed to stop sidecar: {}", e))?;
            // Esperar a que el proceso termine
            let _ = sidecar.process.wait();
        }
        None::<u16>
    };

    // Emitir evento de inicio de nuevo sidecar
    let _ = app.emit("sidecar:starting", ());

    // Resolver binary path
    let binary_path = sidecar::resolve_binary_path()?;

    // Encontrar puerto disponible
    let port = sidecar::find_available_port()?;

    // Obtener threads óptimos
    let threads = sidecar::get_optimal_threads();

    // Construir comando con args base
    let mut cmd = std::process::Command::new(&binary_path);
    let mut args = vec![
        "-m".to_string(),
        model_path.to_str().unwrap().to_string(),
        "-c".to_string(),
        "131072".to_string(),
        "-t".to_string(),
        threads.to_string(),
        "--port".to_string(),
        port.to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--embedding".to_string(),
    ];

    // Agregar --mmproj si existe
    if let Some(mmproj) = mmproj_path {
        args.push("--mmproj".to_string());
        args.push(mmproj.to_str().unwrap().to_string());
    }

    cmd.args(&args);

    // Spawn nuevo proceso
    let child = cmd.spawn().map_err(|e| format!("Failed to start llama-server: {}", e))?;

    // Guardar nuevo estado
    {
        let mut guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        *guard = Some(RunningSidecar {
            process: child,
            port,
            model_path,
        });
    }

    // Emitir ready
    let _ = app.emit("sidecar:ready", port);

    Ok(port)
}

/// Obtiene la ruta del modelo seleccionado para usar en el frontend
#[tauri::command]
pub fn get_model_path(model_id: String) -> Result<String, String> {
    let models_dir = get_models_dir()?;
    let model_path = models_dir.join(format!("{}.gguf", model_id));

    if model_path.exists() {
        Ok(model_path.to_string_lossy().to_string())
    } else {
        Err(format!("Model '{}' not found", model_id))
    }
}