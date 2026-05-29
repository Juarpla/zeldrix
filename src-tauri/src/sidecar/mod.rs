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
pub(crate) fn get_app_dir() -> Option<PathBuf> {
    // 1. Check if we have a local bin or models directory in current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let is_root_local = cwd.join("bin").exists() || cwd.join("models").exists();
        if is_root_local {
            return Some(cwd);
        }
        
        if let Some(parent) = cwd.parent() {
            let is_parent_local = parent.join("bin").exists() || parent.join("models").exists();
            if is_parent_local {
                return Some(parent.to_path_buf());
            }
        }
    }

    // 2. Default to standard Application Support directory
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

    // 2. Resolve target app dir location
    let app_dir = get_app_dir().ok_or("Cannot determine app data directory")?;
    let bin_dir = app_dir.join("bin");
    let local_binary = bin_dir.join("llama-server");

    // 3. If local binary exists, use it
    if local_binary.exists() {
        return Ok(local_binary);
    }

    // 4. Otherwise, check common global system paths on macOS and auto-copy to local bin if local data folders exist
    #[cfg(target_os = "macos")]
    {
        let system_paths = [
            "/opt/homebrew/bin/llama-server",
            "/usr/local/bin/llama-server",
        ];

        for path in &system_paths {
            let system_bin = PathBuf::from(path);
            if system_bin.exists() {
                // We found a global llama-server installation!
                // Copy it to our local data/project bin directory to make it self-contained
                if let Err(e) = std::fs::create_dir_all(&bin_dir) {
                    eprintln!("Warning: Failed to create bin directory: {}", e);
                } else if let Err(e) = std::fs::copy(&system_bin, &local_binary) {
                    eprintln!("Warning: Failed to auto-copy llama-server binary: {}", e);
                } else {
                    println!("Successfully auto-installed llama-server to local path: {}", local_binary.display());
                    return Ok(local_binary);
                }
                
                // Fallback: return the system path directly if copying fails
                return Ok(system_bin);
            }
        }
    }

    // 5. Default check in Application Support / local bin
    if local_binary.exists() {
        Ok(local_binary)
    } else {
        Err(format!(
            "llama-server not found at {}. Set ZELDRIX_LLAMA_SERVER env var or place binary in app support or project bin directory.",
            local_binary.display()
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
    let model_path = app_dir.join("models").join("gemma-4-E4B-it-IQ4_XS.gguf");

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

    let projector_name = if model_id.starts_with("gemma-4-E2B") {
        "mmproj-gemma-4-E2B-it-IQ4_XS.gguf"
    } else if model_id.starts_with("gemma-4-E4B") {
        "mmproj-gemma-4-E4B-it-IQ4_XS.gguf"
    } else {
        return None;
    };

    let mmproj_path = models_dir.join(projector_name);
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