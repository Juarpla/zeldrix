# Spec: Issue 18 - Integración del Modelo de Embeddings Local mediante llama.cpp

## Objective
Configurar el backend de Rust para ejecutar inferencias en modo embedding utilizando el servidor local de llama.cpp (llama-server). El sistema permitirá transformar cualquier fragmento de texto plano en un vector numérico (arreglo de flotantes de alta dimensión) para representar fielmente su semántica.

## Tech Stack
- Lenguaje: Rust (dentro del backend de Tauri v2).
- Dependencias: `reqwest` (cliente HTTP para la API de llama-server), `serde` y `serde_json` (para serialización y deserialización de payloads).

## Commands
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Tauri compile check: `cargo check --manifest-path src-tauri/Cargo.toml`
- Dev server: `npm run dev`

## Project Structure
- `src-tauri/src/document_ingestion/embeddings.rs` [NEW]: Contiene la lógica del cliente HTTP para comunicarse con el endpoint de embeddings de llama-server, la función interna y el comando Tauri.
- `src-tauri/src/document_ingestion/mod.rs` [MODIFY]: Registra y expone el módulo de embeddings.
- `src-tauri/src/lib.rs` [MODIFY]: Configura los argumentos de inicio del sidecar de llama.cpp con el flag `--embedding` para habilitar el endpoint de embeddings, y registra el nuevo comando Tauri.
- `src-tauri/src/download_manager/mod.rs` [MODIFY]: Configura los argumentos de inicio de `hot_swap_model` con el flag `--embedding`.

## Code Style
El módulo de embeddings debe ser modular, libre de comentarios innecesarios, escrito estrictamente en inglés y seguir las mejores prácticas de Rust (manejo de errores robusto con `Result` en lugar de panics, sin `unwrap()`, evitando clones redundantes).

Ejemplo de firma de la función interna:
```rust
pub async fn generate_embeddings(text: &str, port: u16) -> Result<Vec<f32>, String>;
```

Y la firma del comando Tauri expuesto:
```rust
#[tauri::command]
pub async fn get_embeddings(
    sidecar_state: State<'_, SidecarState>,
    text: String,
) -> Result<Vec<f32>, String>;
```

## Testing Strategy
- **Pruebas de Integración con Mock HTTP:** Utilizar tests unitarios en Rust que validen la deserialización de la respuesta de embeddings de llama-server.
- **Prueba con Servidor Activo (Opcional/Manual):** Si el servidor local está iniciado en el puerto activo, comprobar que la llamada a `generate_embeddings` retorna un vector con flotantes no vacíos y dimensiones adecuadas (por ejemplo, 768 o 1024 dimensiones dependiendo del modelo de embeddings cargado).

## Boundaries
- **Siempre:** Agregar el flag `--embedding` al iniciar llama-server en todos los flujos de inicio (`sidecar_start`, `setup` automático y `hot_swap_model`).
- **Siempre:** Propagar los errores de red, de parseo JSON y de estado de forma limpia mediante `Result<Vec<f32>, String>`.
- **Nunca:** Hacer `unwrap()` o panickear en la lógica de embeddings o del sidecar.

## Success Criteria
1. El backend de Rust incluye una función `generate_embeddings` que acepta un fragmento de texto y un puerto, y retorna un vector de flotantes (`Vec<f32>`).
2. Se expone un comando de Tauri llamado `get_embeddings` para poder ser invocado desde el frontend o mediante tests de integración.
3. El servidor local de `llama-server` se inicia correctamente con el flag `--embedding`.
4. El proyecto compila y los tests pasan sin errores ni advertencias.

## Open Questions
- ¿Deberíamos soportar tanto el endpoint compatible con OpenAI `/v1/embeddings` como el endpoint directo `/embedding` de llama.cpp?
  *Implementaremos soporte robusto intentando usar preferentemente `/v1/embeddings` (OpenAI-compatible) ya que es la interfaz estándar configurada en llama.cpp moderno y consistente con el resto de la app, y manejaremos los errores de forma clara.*
