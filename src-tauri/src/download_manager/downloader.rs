//! Downloader - Async streaming download with progress tracking

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Progreso de descarga
#[derive(Clone, Default)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub is_finished: bool,
    pub is_cancelled: bool,
}

/// Handle para controlar la descarga desde外部
pub struct DownloaderHandle {
    progress: Arc<AtomicU64>,
    total_bytes: Arc<AtomicU64>,
    is_finished: Arc<AtomicBool>,
    pub is_cancelled: Arc<AtomicBool>,
}

impl DownloaderHandle {
    /// Crea un nuevo handle con el tamaño esperado
    pub fn new(expected_size: u64) -> Self {
        Self {
            progress: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(expected_size)),
            is_finished: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Obtiene el progreso actual
    pub fn get_progress(&self) -> DownloadProgress {
        DownloadProgress {
            bytes_downloaded: self.progress.load(Ordering::Relaxed),
            total_bytes: if self.total_bytes.load(Ordering::Relaxed) == 0 {
                None
            } else {
                Some(self.total_bytes.load(Ordering::Relaxed))
            },
            is_finished: self.is_finished.load(Ordering::Relaxed),
            is_cancelled: self.is_cancelled.load(Ordering::Relaxed),
        }
    }

    /// Cancela la descarga
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::Relaxed);
    }
}

/// Downloader que realiza streaming de archivos con seguimiento de progreso
pub struct Downloader {
    url: String,
    destination: PathBuf,
    expected_size: u64,
    client: Client,
    handle: Arc<DownloaderHandle>,
}

impl Downloader {
    pub fn new(url: String, destination: PathBuf, expected_size: u64, handle: Arc<DownloaderHandle>) -> Self {
        Self {
            url,
            destination,
            expected_size,
            client: Client::new(),
            handle,
        }
    }

    /// Realiza la descarga con streaming
    pub async fn download(self) -> Result<(), DownloadError> {
        let response = self.client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| DownloadError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DownloadError::Http(response.status().as_u16()));
        }

        // Crear archivo de destino
        let mut file = File::create(&self.destination)
            .await
            .map_err(|e| DownloadError::FileSystem(e.to_string()))?;

        let mut stream = response.bytes_stream();

        loop {
            // Verificar si fue cancelado
            if self.handle.is_cancelled.load(Ordering::Relaxed) {
                let _ = tokio::fs::remove_file(&self.destination).await;
                return Err(DownloadError::Cancelled);
            }

            match stream.next().await {
                Some(Ok(bytes)) => {
                    file.write_all(&bytes)
                        .await
                        .map_err(|e| DownloadError::FileSystem(e.to_string()))?;

                    self.handle.progress.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                }
                Some(Err(e)) => {
                    let _ = tokio::fs::remove_file(&self.destination).await;
                    return Err(DownloadError::Network(e.to_string()));
                }
                None => break,
            }
        }

        file.flush()
            .await
            .map_err(|e| DownloadError::FileSystem(e.to_string()))?;

        self.handle.is_finished.store(true, Ordering::Relaxed);

        Ok(())
    }
}

#[derive(Debug)]
pub enum DownloadError {
    Network(String),
    Http(u16),
    FileSystem(String),
    Cancelled,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Network(msg) => write!(f, "Network error: {}", msg),
            DownloadError::Http(code) => write!(f, "HTTP error: {}", code),
            DownloadError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            DownloadError::Cancelled => write!(f, "Download cancelled"),
        }
    }
}

impl std::error::Error for DownloadError {}