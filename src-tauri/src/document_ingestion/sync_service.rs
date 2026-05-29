// src-tauri/src/document_ingestion/sync_service.rs

use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use notify::{Watcher, RecursiveMode, EventKind};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use chrono;

use crate::sidecar::SidecarState;
use crate::vector_db::VectorDbState;
use crate::document_ingestion::{
    extract_text_from_path,
    chunk_text,
    embeddings::generate_embeddings,
    ChunkConfig,
};

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatus {
    pub monitored_folder: Option<String>,
    pub is_active: bool,
    pub processed_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueItem {
    pub file_path: String,
    pub status: String, // "pending", "processing", "completed", "failed"
    pub added_at: String,
    pub error: Option<String>,
}

pub struct SyncServiceState {
    pub monitored_folder: RwLock<Option<PathBuf>>,
    pub queue: RwLock<Vec<QueueItem>>,
    pub processed_count: AtomicUsize,
    pub error_count: AtomicUsize,
    pub watcher: Mutex<Option<notify::RecommendedWatcher>>,
    pub tx: tokio::sync::mpsc::UnboundedSender<PathBuf>,
}

impl SyncServiceState {
    pub fn new(tx: tokio::sync::mpsc::UnboundedSender<PathBuf>) -> Self {
        SyncServiceState {
            monitored_folder: RwLock::new(None),
            queue: RwLock::new(Vec::new()),
            processed_count: AtomicUsize::new(0),
            error_count: AtomicUsize::new(0),
            watcher: Mutex::new(None),
            tx,
        }
    }
}

pub fn initialize_watcher(
    monitored_path: PathBuf,
    sync_state: &SyncServiceState,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut watcher_guard = sync_state.watcher.lock().map_err(|e| e.to_string())?;
    
    // Stop existing watcher if active
    if let Some(mut old_watcher) = watcher_guard.take() {
        if let Some(ref path) = *sync_state.monitored_folder.read().unwrap() {
            let _ = old_watcher.unwatch(path);
        }
    }

    *sync_state.monitored_folder.write().unwrap() = Some(monitored_path.clone());
    let tx = sync_state.tx.clone();
    let app_handle_callback = app_handle.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(event) => {
                if is_creation_or_modification_event(&event.kind) {
                    for path in event.paths {
                        if is_supported_document_format(&path) {
                            println!("BackgroundSync: Detected file system change at {:?}", path);
                            if let Err(err) = tx.send(path.clone()) {
                                eprintln!("BackgroundSync: Failed to send path to channel: {}", err);
                            } else {
                                // Add file to the queue in state as "pending"
                                if let Some(sync_state) = app_handle_callback.try_state::<SyncServiceState>() {
                                    let mut queue = sync_state.queue.write().unwrap();
                                    let path_str = path.to_string_lossy().to_string();
                                    
                                    // Avoid duplicates in the pending status
                                    if !queue.iter().any(|item| item.file_path == path_str && item.status == "pending") {
                                        queue.push(QueueItem {
                                            file_path: path_str,
                                            status: "pending".to_string(),
                                            added_at: chrono::Utc::now().to_rfc3339(),
                                            error: None,
                                        });
                                        println!("BackgroundSync: File queued successfully: {}", path.display());
                                        let _ = app_handle_callback.emit("sync:queue-updated", ());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("BackgroundSync: Watcher error: {:?}", e),
        }
    })
    .map_err(|e| format!("Failed to create recommended watcher: {}", e))?;

    watcher
        .watch(&monitored_path, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to start watching directory: {}", e))?;

    *watcher_guard = Some(watcher);
    
    // Save new configuration to local settings file
    let local_data_dir = app_handle.path().app_local_data_dir().unwrap_or_default();
    let config_path = local_data_dir.join("sync_config.json");
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    #[derive(serde::Serialize)]
    struct SavedConfig {
        monitored_folder: String,
    }
    
    let config_data = SavedConfig {
        monitored_folder: monitored_path.to_string_lossy().to_string(),
    };
    
    if let Ok(serialized) = serde_json::to_string_pretty(&config_data) {
        let _ = std::fs::write(config_path, serialized);
    }

    Ok(())
}

fn is_creation_or_modification_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_)
            | EventKind::Modify(notify::event::ModifyKind::Data(_))
            | EventKind::Modify(notify::event::ModifyKind::Any)
    )
}

fn is_supported_document_format(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        matches!(extension.to_ascii_lowercase().as_str(), "txt")
    } else {
        false
    }
}

pub fn start_sync_worker(
    app_handle: AppHandle,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<PathBuf>,
) {
    tauri::async_runtime::spawn(async move {
        println!("BackgroundSync: Starting worker thread...");
        while let Some(path) = rx.recv().await {
            // Give the OS or user-process a very brief window to finalize writing the file
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let path_str = path.to_string_lossy().to_string();
            println!("BackgroundSync: Starting ingestion process for {}", path_str);

            // Update status to processing
            if let Some(sync_state) = app_handle.try_state::<SyncServiceState>() {
                let mut queue = sync_state.queue.write().unwrap();
                if let Some(item) = queue.iter_mut().find(|item| item.file_path == path_str && item.status == "pending") {
                    item.status = "processing".to_string();
                }
                let _ = app_handle.emit("sync:queue-updated", ());
            }

            match process_single_file(&app_handle, &path).await {
                Ok(_) => {
                    println!("BackgroundSync: File indexed successfully: {}", path_str);
                    if let Some(sync_state) = app_handle.try_state::<SyncServiceState>() {
                        sync_state.processed_count.fetch_add(1, Ordering::SeqCst);
                        let mut queue = sync_state.queue.write().unwrap();
                        if let Some(item) = queue.iter_mut().find(|item| item.file_path == path_str && item.status == "processing") {
                            item.status = "completed".to_string();
                        }
                        let _ = app_handle.emit("sync:queue-updated", ());
                        let _ = app_handle.emit("sync:status-updated", ());
                    }
                }
                Err(err) => {
                    eprintln!("BackgroundSync: Ingestion failed for {}: {}", path_str, err);
                    if let Some(sync_state) = app_handle.try_state::<SyncServiceState>() {
                        sync_state.error_count.fetch_add(1, Ordering::SeqCst);
                        let mut queue = sync_state.queue.write().unwrap();
                        if let Some(item) = queue.iter_mut().find(|item| item.file_path == path_str && item.status == "processing") {
                            item.status = "failed".to_string();
                            item.error = Some(err);
                        }
                        let _ = app_handle.emit("sync:queue-updated", ());
                        let _ = app_handle.emit("sync:status-updated", ());
                    }
                }
            }
        }
    });
}

async fn process_single_file(app_handle: &AppHandle, path: &Path) -> Result<(), String> {
    // 1. Get Sidecar state and verify health / active port
    let sidecar_state = app_handle
        .try_state::<SidecarState>()
        .ok_or_else(|| "SidecarState is not managed".to_string())?;

    let port = {
        let guard = sidecar_state.0.lock().map_err(|e| e.to_string())?;
        guard.as_ref().map(|s| s.port)
    };

    let port = port.ok_or_else(|| "Llama.cpp sidecar is not running. Ingestion postponed.".to_string())?;

    // 2. Extract Document text
    let document = extract_text_from_path(path).map_err(|e| e.to_string())?;

    // 3. Chunk text
    let chunk_config = ChunkConfig {
        chunk_size: 500,
        overlap_percentage: 0.10,
        estimator: crate::document_ingestion::chunking::TokenEstimator::Heuristic,
    };
    let chunks = chunk_text(&document.text, &chunk_config);

    // 4. Generate Embeddings and Index into database
    let vector_db_state = app_handle
        .try_state::<VectorDbState>()
        .ok_or_else(|| "VectorDbState is not managed".to_string())?;

    let file_path_str = path.to_string_lossy().to_string();

    for (index, chunk) in chunks.iter().enumerate() {
        let chunk_id = format!("{}_chunk_{}", document.file_name, index);
        
        // Generate embedding vector
        let vector = generate_embeddings(&chunk.text, port).await?;

        // Insert into database
        let mut database = vector_db_state.0.write().map_err(|e| e.to_string())?;
        database.insert(
            chunk_id,
            vector,
            chunk.text.clone(),
            file_path_str.clone(),
            None,
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_monitored_folder(state: State<'_, SyncServiceState>) -> Result<Option<String>, String> {
    let folder = state.monitored_folder.read().map_err(|e| e.to_string())?;
    Ok(folder.as_ref().map(|path| path.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn set_monitored_folder(
    app_handle: AppHandle,
    state: State<'_, SyncServiceState>,
    folder_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(folder_path);
    if !path.exists() {
        return Err("The specified folder path does not exist".to_string());
    }
    if !path.is_dir() {
        return Err("The specified path is not a directory".to_string());
    }

    initialize_watcher(path, &state, app_handle)?;
    Ok(())
}

#[tauri::command]
pub fn get_sync_status(state: State<'_, SyncServiceState>) -> Result<SyncStatus, String> {
    let folder = state.monitored_folder.read().map_err(|e| e.to_string())?;
    let watcher_exists = state.watcher.lock().map_err(|e| e.to_string())?.is_some();
    
    Ok(SyncStatus {
        monitored_folder: folder.as_ref().map(|p| p.to_string_lossy().to_string()),
        is_active: watcher_exists,
        processed_count: state.processed_count.load(Ordering::SeqCst),
        error_count: state.error_count.load(Ordering::SeqCst),
    })
}

#[tauri::command]
pub fn get_sync_queue(state: State<'_, SyncServiceState>) -> Result<Vec<QueueItem>, String> {
    let queue = state.queue.read().map_err(|e| e.to_string())?;
    Ok(queue.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_sync_queue_item_creation() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let state = SyncServiceState::new(tx);
        
        {
            let mut queue = state.queue.write().unwrap();
            queue.push(QueueItem {
                file_path: "/path/to/doc.txt".to_string(),
                status: "pending".to_string(),
                added_at: "2026-05-28T00:00:00Z".to_string(),
                error: None,
            });
        }

        let items = state.queue.read().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].file_path, "/path/to/doc.txt");
        assert_eq!(items[0].status, "pending");
    }

    #[tokio::test]
    async fn test_supported_document_format_filter() {
        let dir = tempdir().unwrap();
        let txt_path = dir.path().join("doc.txt");
        let mut file = File::create(&txt_path).unwrap();
        writeln!(file, "Hello World").unwrap();

        let bin_path = dir.path().join("doc.bin");
        let mut bin_file = File::create(&bin_path).unwrap();
        bin_file.write_all(b"binary").unwrap();

        assert!(is_supported_document_format(&txt_path));
        assert!(!is_supported_document_format(&bin_path));
    }
}
