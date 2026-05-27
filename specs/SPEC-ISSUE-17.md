# Spec: Issue 17 - Módulo de Chunking (Segmentación de Texto) con Solapamiento Dinámico

## Objective
Desarrollar un algoritmo en Rust para fragmentar texto extraído de documentos en fragmentos ("chunks") manejables de tamaño configurable (por ejemplo, 512 tokens) con un solapamiento dinámico y configurable (por defecto 10%) para preservar el contexto semántico entre fragmentos adyacentes. El módulo debe garantizar que no se pierdan palabras en los límites de los fragmentos y que se respeten los límites de las palabras (evitando cortar palabras a la mitad).

## Tech Stack
- Lenguaje: Rust (dentro del backend de Tauri v2).
- Dependencias: Ninguna dependencia externa de red. Uso de la biblioteca estándar de Rust y la biblioteca de expresiones regulares (`regex`) si es necesario.

## Commands
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Tauri compile check: `cargo check --manifest-path src-tauri/Cargo.toml`

## Project Structure
- `src-tauri/src/document_ingestion/chunking.rs`: Contiene la lógica del algoritmo de chunking, tipos de configuración, estimación de tokens y tests asociados.
- `src-tauri/src/document_ingestion/mod.rs`: Registra y expone el módulo de chunking.

## Code Style
El módulo de chunking debe ser puro, altamente testable, eficiente y documentado. Utilizaremos el sistema de tipos de Rust para parametrizar la estrategia de conteo de tokens.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenEstimator {
    /// Cada palabra delimitada por espacios es un token.
    Words,
    /// Estimación basada en caracteres (ej. 1 token ≈ 4 caracteres).
    Characters { chars_per_token: usize },
    /// Estimación heurística común para LLMs (ej. 1 palabra ≈ 1.3 tokens).
    Heuristic,
}

#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub chunk_size: usize,
    pub overlap_percentage: f64,
    pub estimator: TokenEstimator,
}
```

## Testing Strategy
- **Pruebas Unitarias de Límites:** Verificar que un texto largo fragmentado no pierda ninguna palabra en la reconstrucción o en las uniones.
- **Pruebas de Solapamiento:** Comprobar que los fragmentos adyacentes contienen exactamente el solapamiento esperado (10%).
- **Pruebas de Integridad de Palabras:** Asegurar que ninguna palabra sea cortada por la mitad en los extremos de un fragmento.
- **Pruebas de Robustez:** Probar con textos vacíos, textos más cortos que el tamaño de un chunk, y textos con caracteres Unicode complejos.

## Boundaries
- **Siempre:** Garantizar la integridad total del texto original (todas las palabras significativas deben estar presentes en al menos un chunk).
- **Siempre:** Dividir los chunks respetando los límites de palabras (espacios, saltos de línea).
- **Nunca:** Perder caracteres o palabras en los límites de los fragmentos.
- **Nunca:** Modificar el texto original destructivamente (debe ser posible reconstruir el texto original de forma coherente o mapear los chunks a índices del texto original).

## Success Criteria
1. El algoritmo recibe un texto de entrada y un `ChunkConfig`, y produce un `Vec<Chunk>` donde cada `Chunk` contiene el fragmento de texto, su rango en caracteres (`start_char`, `end_char`) y el número de tokens estimados.
2. Un texto largo procesado pasa un test automatizado que valida que el 100% de las palabras y caracteres originales se conservan, y que los fragmentos adyacentes tienen el solapamiento configurado sin mutilar palabras.
3. Se implementa en Rust limpio y pasa `cargo test` y `cargo check` sin errores ni warnings.

## Open Questions
- ¿Deberíamos exponer esta funcionalidad como un comando de Tauri para el frontend? *Sí, añadiremos una función expuesta en Tauri si es necesario para depuración o procesamiento del lado del cliente, pero la lógica principal residirá en Rust para el flujo RAG.*
