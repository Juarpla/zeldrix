# SPEC.md — Download Orchestrator con Verificación SHA-256 y Hot-Swap

## 1. Concepto y Visión

Un gestor de descargas seguro y resistente que permite a los usuarios descargar cualquiera de los 8 modelos opcionales de Unsloth directamente desde Hugging Face. El sistema proporciona transparencia total mediante actualizaciones de progreso en tiempo real, validación criptográfica del archivo descargado, y la capacidad de cambiar modelos "en caliente" sin interrumpir la aplicación. La experiencia debe sentirse profesional y confiable — como un gestor de actualizaciones de un software de producción.

## 2. Diseño Técnico

### 2.1 Modelos Disponibles (Unsloth)

| ID | Nombre | Tipo | VRAM | Params |
|----|--------|------|------|--------|
| gemma-4-E2B-it-IQ4_XS | Gemma 3 4B IT Q4_XS | Chat | ~4GB | 4B |
| llama-4-E2B-it-IQ4_M | Llama 4 Scout Q4_M | Chat | ~8GB | 17B |
| mistral-4-E2B-it-IQ4_M | Mistral Small 3.1 Q4_M | Chat | ~8GB | 22B |
| qwen-4-E2B-it-IQ4_M | Qwen 3.5 Q4_M | Chat | ~8GB | 32B |
| deepseek-4-E2B-it-IQ4_M | Deepseek Chat Q4_M | Chat | ~8GB | 32B |
| phi-4-E2B-it-IQ4_M | Phi-4 Q4_M | Chat | ~8GB | 14B |
| nomic-4-E2B-it-IQ4_M | Nomic Embed Q4_M | Embedding | ~4GB | 7B |
| excel-4-E2B-it-IQ4_M | Excel LLM Q4_M | Specialized | ~8GB | 14B |

### 2.2 Estructura de Descarga

```
~/.local/share/zeldrix/
├── models/
│   ├── gemma-4-E2B-it-IQ4_XS.gguf
│   ├── llama-4-E2B-it-IQ4_M.gguf
│   └── ...
├── downloads/
│   └── gemma-4-E2B-it-IQ4_XS.gguf.part (archivo en progreso)
└── bin/
    └── llama-server
```

### 2.3 Endpoints de Descarga

URL Base: `https://huggingface.co/unsloth/unsloth.2025.41/resolve/main/`

- gemma-4-E2B-it-IQ4_XS.gguf
- llama-4-E2B-it-IQ4_M.gguf
- mistral-4-E2B-it-IQ4_M.gguf
- qwen-4-E2B-it-IQ4_M.gguf
- deepseek-4-E2B-it-IQ4_M.gguf
- phi-4-E2B-it-IQ4_M.gguf
- nomic-4-E2B-it-IQ4_M.gguf
- excel-4-E2B-it-IQ4_M.gguf

## 3. Arquitectura del Backend (Rust)

### 3.1 Módulo `download_manager`

Nuevo módulo en `src-tauri/src/download_manager/`:

```
download_manager/
├── mod.rs          # Estado global y comandos Tauri
├── downloader.rs   # Lógica de descarga con streams
├── hasher.rs       # Verificación SHA-256
└── hot_swap.rs     # Gestión de procesos llama-server
```

### 3.2 Estado Global

```rust
pub struct DownloadState(pub Mutex<Option<RunningDownload>>);

pub struct RunningDownload {
    pub model_id: String,
    pub progress: DownloadProgress,
    pub cancel_token: CancellationToken,
}

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
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Completed,
    Failed { error: String },
    Cancelled,
}
```

### 3.3 Comandos Tauri

| Comando | Descripción |
|---------|-------------|
| `download_model(model_id)` | Inicia descarga de un modelo |
| `cancel_download()` | Cancela la descarga en curso |
| `get_download_progress()` | Retorna estado actual |
| `hot_swap_model(model_id)` | Detiene llama-server y carga nuevo modelo |
| `list_models()` | Lista modelos disponibles con estado |

### 3.4 Flujo de Descarga

1. **Inicio**: Validar que no haya descarga activa
2. **Stream HTTP**: Usar `reqwest` con streaming para descarga chunked
3. **Escritura chunks**: Escribir a `.part` file mientras se descarga
4. **Progreso**: Calcular bytes/tiempo para velocidad, emit events cada segundo
5. **Verificación**: Calcular SHA-256 del archivo final, comparar con hash conocido
6. **Rename**: Si hash OK, renombrar `.part` → `.gguf`
7. **Error**: Si hash falla, borrar archivo y notificar

### 3.5 Eventos Tauri Emitidos

| Evento | Payload | Frecuencia |
|--------|---------|------------|
| `download:progress` | `DownloadProgress` | Cada segundo |
| `download:complete` | `{ model_id, path }` | Una vez |
| `download:error` | `{ model_id, error }` | Una vez |
| `download:cancelled` | `{ model_id }` | Una vez |

### 3.6 Hot-Swap Logic

```
hot_swap_model(model_id):
  1. Verificar que modelo existe en ~/.local/share/zeldrix/models/
  2. Si sidecar corriendo:
     a. Enviar evento "sidecar:stopping"
     b. Matar proceso llama-server (graceful SIGTERM, luego SIGKILL)
     c. Esperar a que puerto esté libre
  3. Obtener nuevo binary_path y model_path
  4. Iniciar nuevo llama-server con nuevo modelo
  5. Enviar evento "sidecar:ready" con nuevo puerto
```

## 4. Frontend (React)

### 4.1 Componentes

```
src/components/
├── DownloadManager.tsx    # Contenedor principal
├── ModelCard.tsx          # Tarjeta individual de modelo
├── ProgressBar.tsx        # Barra de progreso animada
└── DownloadEvents.tsx     # Handler de eventos
```

### 4.2 Modelo de Datos (TypeScript)

```typescript
interface ModelInfo {
  id: string;
  name: string;
  type: 'chat' | 'embedding' | 'specialized';
  vram: string;
  params: string;
  size_bytes?: number;
  status: 'available' | 'downloading' | 'not_downloaded' | 'error';
  download_progress?: number;
}

interface DownloadProgress {
  model_id: string;
  bytes_downloaded: number;
  total_bytes: number | null;
  percentage: number;
  speed_bps: number;
  status: 'pending' | 'downloading' | 'verifying' | 'completed' | 'failed' | 'cancelled';
  error?: string;
}
```

### 4.3 Interfaz Visual

**Layout**: Grid de 2x4 con tarjetas de modelo
**Cada tarjeta muestra**:
- Nombre del modelo
- Tipo y VRAM estimada
- Estado (disponible/descargando/no descargado)
- Botón de acción (Descargar / Cancelar / Usar)
- Barra de progreso (solo durante descarga)

**Estados de la tarjeta**:
- `not_downloaded`: Botón "Descargar" activo
- `downloading`: Barra de progreso animada + botón "Cancelar"
- `available`: Badge "Descargado" + botón "Usar"
- `error`: Mensaje de error + botón "Reintentar"

## 5. Criterios de Aceptación

1. ✅ Al iniciar descarga, el frontend recibe eventos `download:progress` cada segundo con porcentaje exacto
2. ✅ El porcentaje se calcula como `(bytes_descargados / total_bytes) * 100`
3. ✅ Si el hash SHA-256 no coincide, el archivo `.part` se borra automáticamente
4. ✅ Se emite `download:error` con mensaje de hash inválido
5. ✅ El hot-swap permite cambiar de modelo sin reiniciar la aplicación Tauri
6. ✅ Solo puede haber una descarga activa a la vez
7. ✅ La velocidad de descarga se calcula y muestra en MB/s

## 6. Dependencias a Agregar (Cargo.toml)

```toml
# Download manager
sha2 = "0.10"           # SHA-256 hashing
tokio-util = { version = "0.7", features = ["io"] }  # Stream utilities
futures = "0.3"         # Async utilities

# Para cancel token
tokio = { version = "1", features = ["sync"] }
```

## 7. Notas de Implementación

- Usar `reqwest::get(url).await` con `.bytes_stream()` para streaming
- Escribir chunks directamente a archivo con `tokio::fs::File`
- Calcular hash con `sha2::{Sha256, Digest}` actualizando incrementally
- Guardar estado de descarga en `State<DownloadState>` accesible por comandos
- Los eventos se emiten via `app.emit()` hacia el frontend
- Para hot-swap, el proceso de llama-server se maneja vía el `SidecarState` existente