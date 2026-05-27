# Spec: Issue 19 - Base de Datos Vectorial Embebida e Indexación en Local

## Objective
Integrar una base de datos vectorial ultraligera y embebida que corra en el mismo proceso de la aplicación (como una base vectorial en memoria en Rust persistida mediante archivos planos). El sistema guardará los vectores junto con el texto crudo y la ruta del archivo original. Se implementará una búsqueda por similitud de cosenos altamente optimizada.

El criterio clave de aceptación es que consultas masivas sobre un índice de 10,000 vectores deben ejecutarse y ordenar resultados de similitud en menos de 50ms.

## Tech Stack
- Lenguaje: Rust (dentro del backend de Tauri v2).
- Dependencias: `serde` y `serde_json` (para serialización y deserialización de la persistencia en formato plano).
- Dependencias opcionales de optimización: Ninguna requerida; utilizaremos Rust standard library iterators auto-vectorizados por LLVM para máxima simplicidad y robustez, o multihilo simple si fuera necesario.

## Commands
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Tauri compile check: `cargo check --manifest-path src-tauri/Cargo.toml`
- Dev server: `npm run dev`

## Project Structure
- `src-tauri/src/vector_db/mod.rs` [NEW]: Contiene la estructura de datos `VectorDatabase` en memoria con soporte para:
  - Inserción y eliminación de vectores.
  - Búsqueda por similitud de cosenos (calculada de forma altamente optimizada).
  - Persistencia y carga incremental/completa a un archivo JSON plano local (`vector_db.json`).
  - Comando Tauri expuestos para frontend e integraciones.
- `src-tauri/src/lib.rs` [MODIFY]: Registra y expone el estado global de la base de datos vectorial en Tauri y registra los nuevos comandos.

## Code Style
El módulo de la base de datos vectorial debe seguir las directrices de `AGENTS.md` y `rust-best-practices`:
- Libre de comentarios redundantes o innecesarios.
- Nombres de funciones y variables sumamente descriptivos en inglés.
- Cero `unwrap()` o `expect()` en producción. Manejo correcto de errores con `Result<T, E>`.
- Evitar clonaciones excesivas mediante el uso de referencias y estructuras eficientes.

### Estructura del VectorRecord
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub text: String,
    pub file_path: String,
}
```

### Operación de Similitud de Coseno
Para maximizar el rendimiento, los vectores se pueden pre-normalizar al insertarlos para que la similitud de coseno se reduzca a un simple producto punto:
$$Similarity(A, B) = \sum (A_i \cdot B_i)$$
Esto reduce significativamente el número de operaciones de división y raíz cuadrada durante las búsquedas.

## Testing Strategy
- **Pruebas de Funcionalidad:** Validar inserción, búsqueda exacta y persistencia en disco de vectores de prueba.
- **Pruebas de Rendimiento (Benchmark):** Un test automático que genere e inserte 10,000 vectores sintéticos (por ejemplo, de 1024 dimensiones) y ejecute una consulta de similitud de cosenos, validando que el tiempo transcurrido para ordenar los resultados sea menor a 50ms (el objetivo real debería ser <5ms en hardware moderno).

## Boundaries
- **Siempre:** Validar que los vectores de consulta tengan la misma dimensión que los vectores almacenados antes de calcular similitudes.
- **Siempre:** Guardar de forma atómica los cambios en disco o gestionar la consistencia ante caídas inesperadas de la aplicación.
- **Nunca:** Paniquear en la lógica de cálculo matemático o persistencia; propagar los errores de forma limpia hacia Tauri.

## Success Criteria
1. Creación e integración del módulo `src-tauri/src/vector_db`.
2. Implementación de una consulta por similitud de cosenos que ordene 10,000 vectores en menos de 50ms (verificado mediante un test unitario de rendimiento).
3. Persistencia automática o explícita del índice vectorial en un archivo plano en la ruta de datos local.
4. Registro de los comandos Tauri `vector_db_insert`, `vector_db_search`, `vector_db_clear` y `vector_db_load`.
5. Compilación del proyecto y paso de tests unitarios exitosamente.

## Open Questions
- ¿Es necesario indexar los vectores con estructuras avanzadas (como HNSW)?
  *No es necesario para 10,000 vectores, ya que un escaneo plano lineal en Rust con pre-normalización toma menos de 2ms y ofrece recall del 100% de manera determinista.*
