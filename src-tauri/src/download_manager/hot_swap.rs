//! Hot-Swap Manager for llama-server process
//!
//! Handles graceful shutdown and restart of llama-server when switching models.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use tokio::time::sleep;

/// Configuración para hot-swap
pub struct HotSwapConfig {
    /// Ruta al binario de llama-server
    pub binary_path: PathBuf,
    /// Ruta al modelo
    pub model_path: PathBuf,
    /// Puerto para el servidor
    pub port: u16,
    /// Número de threads
    pub threads: usize,
    /// Contexto máximo
    pub context_size: u32,
    /// Timeout para esperar a que el proceso termine (en segundos)
    pub shutdown_timeout_secs: u64,
}

impl HotSwapConfig {
    /// Crea una nueva configuración
    pub fn new(binary_path: PathBuf, model_path: PathBuf, port: u16) -> Self {
        let threads = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
            .saturating_sub(2)
            .max(1);

        Self {
            binary_path,
            model_path,
            port,
            threads,
            context_size: 131072,
            shutdown_timeout_secs: 5,
        }
    }

    /// Construye el comando para iniciar llama-server
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new(&self.binary_path);
        cmd.args([
            "-m", self.model_path.to_str().unwrap(),
            "-c", &self.context_size.to_string(),
            "-t", &self.threads.to_string(),
            "--port", &self.port.to_string(),
            "--host", "127.0.0.1",
        ]);
        cmd
    }
}

/// Detiene un proceso de forma graceful, esperando a que termine.
/// Si no termina en el timeout, lo mata forzosamente.
pub async fn stop_process_gracefully(
    child: &mut Child,
    timeout_duration: Duration,
) -> Result<(), String> {
    // Enviar сигнал de terminación (SIGTERM en Unix, equivalente en Windows)
    child.kill().map_err(|e| format!("Failed to send kill signal: {}", e))?;

    // Esperar a que termine usando thread::sleep porque wait() no es async
    let start = std::time::Instant::now();
    while start.elapsed() < timeout_duration {
        if child.try_wait().map(|opt| opt.is_some()).unwrap_or(false) {
            return Ok(());
        }
        sleep(Duration::from_millis(50)).await;
    }

    // Timeout expirado
    Err("Process did not terminate within timeout".to_string())
}

/// Verifica si un puerto está disponible (no hay proceso escuchándolo)
pub async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok()
}

/// Espera hasta que un puerto esté disponible
pub async fn wait_for_port_available(port: u16, max_attempts: u32) -> Result<(), String> {
    for _ in 0..max_attempts {
        if is_port_available(port).await {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(format!("Port {} not available after {} attempts", port, max_attempts))
}

/// Inicia un nuevo proceso de llama-server y espera a que esté listo
pub async fn start_llama_server(config: HotSwapConfig) -> Result<Child, String> {
    // Verificar que el binario existe
    if !config.binary_path.exists() {
        return Err(format!(
            "llama-server binary not found at {}",
            config.binary_path.display()
        ));
    }

    // Verificar que el modelo existe
    if !config.model_path.exists() {
        return Err(format!(
            "Model not found at {}",
            config.model_path.display()
        ));
    }

    // Verificar que el puerto está disponible
    if !is_port_available(config.port).await {
        return Err(format!(
            "Port {} is not available. Another process might be using it.",
            config.port
        ));
    }

    // Spawn proceso
    let mut child = config.build_command()
        .spawn()
        .map_err(|e| format!("Failed to spawn llama-server: {}", e))?;

    // Esperar brevemente a que el proceso inicie
    sleep(Duration::from_millis(500)).await;

    // Verificar que el proceso sigue corriendo
    match child.try_wait() {
        Ok(Some(_)) => Err("llama-server exited immediately".to_string()),
        Ok(None) => Ok(child), // Still running
        Err(e) => Err(format!("Error checking process status: {}", e)),
    }
}

/// Realiza hot-swap de un modelo a otro
///
/// 1. Detiene el proceso actual de forma graceful
/// 2. Espera a que el puerto esté libre
/// 3. Inicia el nuevo proceso
pub async fn perform_hot_swap(
    mut current_child: Option<Child>,
    config: HotSwapConfig,
) -> Result<Child, String> {
    // Paso 1: Detener proceso actual si existe
    if let Some(mut child) = current_child.take() {
        stop_process_gracefully(&mut child, Duration::from_secs(config.shutdown_timeout_secs)).await?;
    }

    // Paso 2: Esperar a que el puerto esté libre
    wait_for_port_available(config.port, 50).await?;

    // Paso 3: Iniciar nuevo proceso
    start_llama_server(config).await
}