pub(crate) mod health;

use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;
use std::net::TcpListener;

use directories::ProjectDirs;

/// Estado del sidecar - opcional porque puede no estar iniciado
pub struct SidecarState(pub Mutex<Option<RunningSidecar>>);

/// Información del sidecar en ejecución
pub struct RunningSidecar {
    pub process: Child,
    pub port: u16,
    pub model_path: PathBuf,
}

/// Resultado con mensaje de error
type Result<T> = std::result::Result<T, String>;

/// Obtiene el directorio de datos de la app
fn get_app_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "zenit", "zeldrix")
        .map(|dirs| dirs.data_dir().to_path_buf())
}

/// Obtiene el directorio de modelos
pub(crate) fn get_models_dir() -> Option<PathBuf> {
    get_app_dir().map(|dir| dir.join("models"))
}

/// Obtiene el path del binario de llama-server
pub(crate) fn resolve_binary_path() -> Result<PathBuf> {
    // 1. Check env var
    if let Ok(path) = std::env::var("ZELDRIX_LLAMA_SERVER") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. Default location in Application Support
    let app_dir = get_app_dir().ok_or("Cannot determine app data directory")?;
    let binary_path = app_dir.join("bin").join("llama-server");

    if binary_path.exists() {
        Ok(binary_path)
    } else {
        Err(format!(
            "llama-server not found at {}. Set ZELDRIX_LLAMA_SERVER env var or place binary in app support directory.",
            binary_path.display()
        ))
    }
}

/// Obtiene el path del modelo
pub(crate) fn resolve_model_path() -> Result<PathBuf> {
    // 1. Check env var
    if let Ok(path) = std::env::var("ZELDRIX_MODEL_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. Default location in Application Support
    let app_dir = get_app_dir().ok_or("Cannot determine app data directory")?;
    let model_path = app_dir.join("models").join("gemma-4-E2B-it-IQ4_XS.gguf");

    if model_path.exists() {
        Ok(model_path)
    } else {
        Err(format!(
            "Model not found at {}. Set ZELDRIX_MODEL_PATH env var or download the model.",
            model_path.display()
        ))
    }
}

/// Extrae el model_id del path del modelo (ej: "gemma-4-E2B-it-IQ4_XS" de "/path/to/gemma-4-E2B-it-IQ4_XS.gguf")
pub fn extract_model_id(model_path: &PathBuf) -> Option<String> {
    model_path
        .file_stem() // gemma-4-E2B-it-IQ4_XS.gguf -> gemma-4-E2B-it-IQ4_XS
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

/// Obtiene el path del mmproj para un modelo multimodal
/// Retorna Some(path) si existe, None si no existe o no es multimodal
pub fn resolve_mmproj_path(model_path: &PathBuf) -> Option<PathBuf> {
    let model_id = extract_model_id(model_path)?;
    let models_dir = get_models_dir()?;

    // Modelos multimodales conocidos
    let multimodal_models = [
        "gemma-4-E2B-it-IQ4_XS",
        "gemma-4-E4B-it-IQ4_XS",
    ];

    if !multimodal_models.iter().any(|id| id == &model_id) {
        return None;
    }

    let mmproj_path = models_dir.join(format!("mmproj-{}.gguf", model_id));
    if mmproj_path.exists() {
        Some(mmproj_path)
    } else {
        None
    }
}

/// Encuentra un puerto disponible en localhost
pub(crate) fn find_available_port() -> Result<u16> {
    TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to bind to localhost: {}", e))
        .map(|listener| {
            listener.local_addr()
                .map(|addr| addr.port())
                .unwrap_or_else(|_| 8080)
        })
}

/// Obtiene el número de threads óptimo para el sistema
pub(crate) fn get_optimal_threads() -> usize {
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
        .saturating_sub(2) // Leave 2 cores for system
        .max(1)
}

/// Estado del sidecar para retornar al frontend
#[derive(serde::Serialize)]
pub struct SidecarStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub model: Option<String>,
    pub multimodal: bool,
}