# Spec: ISSUE #09 — Base de Datos Local para Plantillas Corporativas (.db)

## Objective

Crear un sistema de persistencia local usando SQLite (rusqlite) para almacenar las plantillas oficiales de la empresa. Cada plantilla contiene metadatos estructurados: nombre, categoría, variables requeridas, prompt de sistema y texto base con tokens dinámicos tipo `{{nombre_cliente}}`.

**Usuario:** El sistema (backend) — las plantillas son consumidas por el motor de IA para generar documentos corporativos.

**Éxito:** La base de datos se inicializa correctamente, guarda una plantilla de prueba y permite su lectura instantánea mediante tests unitarios.

## Tech Stack

- **Rust** (edition 2021) — ya presente en `src-tauri/`
- **rusqlite** v0.32+ — binding SQLite para Rust
- **serde** + **serde_json** — ya presentes, para serialización de variables
- **directories** v6 — ya presente, para rutas de datos de la app

## Commands

```
Build:    cargo build --manifest-path src-tauri/Cargo.toml
Test:     cargo test --manifest-path src-tauri/Cargo.toml --lib
Lint:     cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
Dev:      npm run dev
```

## Project Structure

```
src-tauri/
├── Cargo.toml                          # Agregar rusqlite
├── src/
│   ├── lib.rs                          # Registrar TemplateState + comandos
│   ├── main.rs                         # Sin cambios
│   └── templates_db/
│       ├── mod.rs                      # Template struct, TemplateState, comandos Tauri
│       ├── schema.rs                   # Inicialización de la base de datos (CREATE TABLE)
│       ├── seed.rs                     # Plantilla de prueba (seed data)
│       └── tests.rs                    # Tests unitarios (in-memory DB)
```

## Code Style

```rust
// Template struct — derives para serde y rusqlite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Option<i64>,
    pub name: String,
    pub category: String,
    pub required_variables: Vec<String>,  // ["nombre_cliente", "fecha", "monto"]
    pub system_prompt: String,
    pub base_text: String,                // Contiene tokens {{variable}}
}

// Error type con thiserror-style (String por simplicidad en comandos Tauri)
// Los comandos Tauri retornan Result<T, String>

// Patrón de estado global (consistente con SidecarState, DownloadState)
pub struct TemplateDb(pub Mutex<Connection>);

// Ejemplo de comando Tauri
#[tauri::command]
fn template_list(state: State<'_, TemplateDb>) -> Result<Vec<Template>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    // ... query ...
}
```

**Convenciones:**
- Nombres de módulos: `snake_case`
- Structs: `PascalCase`
- Comandos Tauri: `snake_case` (se auto-convierte a camelCase en JS)
- Sin `unwrap()` fuera de tests
- `Result<T, String>` en comandos Tauri para propagación de errores

## Testing Strategy

- **Framework:** `#[cfg(test)]` modules con tests unitarios en Rust
- **DB de tests:** SQLite `:memory:` — no toca disco, instantánea
- **Cobertura mínima:**
  1. `db_init_creates_tables` — verificar que la tabla se crea sin error
  2. `seed_inserts_test_template` — verificar que el seed inserta correctamente
  3. `read_template_by_id` — verificar que se puede leer la plantilla insertada
  4. `template_variables_parsed` — verificar que las variables se serializan/deserializan correctamente
- **Ubicación:** `src-tauri/src/templates_db/mod.rs` con `#[cfg(test)] mod tests { ... }`

## Boundaries

- **Always:** Usar `:memory:` para tests, usar `directories` para rutas de producción, validar que la tabla existe antes de queries
- **Ask first:** Cambiar el schema de la base de datos (migraciones), agregar encriptación a la DB
- **Never:** Hardcodear paths absolutos, usar `unwrap()` en código de producción, commit de archivos `.db` al repo

## Schema de la Base de Datos

```sql
CREATE TABLE IF NOT EXISTS templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,
    variables   TEXT NOT NULL DEFAULT '[]',   -- JSON array de strings
    system_prompt TEXT NOT NULL,
    base_text   TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Nota:** `variables` se almacena como JSON (`["nombre_cliente", "fecha"]`) para mantener la tabla normalizada y permitir queries flexibles.

## Plantilla de Prueba (Seed)

```json
{
  "name": "Carta de Presentación Corporativa",
  "category": "comunicacion",
  "required_variables": ["nombre_cliente", "empresa", "fecha", "monto_propuesta"],
  "system_prompt": "Eres un asistente de redacción corporativa. Genera cartas formales manteniendo el tono profesional de la empresa.",
  "base_text": "Estimado/a {{nombre_cliente}}:\n\nPor medio de la presente, {{empresa}} se complace en presentar la propuesta por un monto de {{monto_propuesta}}, con fecha {{fecha}}.\n\nQuedamos a su disposición para cualquier consulta.\n\nAtentamente,\nEquipo Corporativo"
}
```

## Success Criteria

1. ✅ `cargo test` pasa con los 4 tests unitarios mínimo
2. ✅ La base de datos se inicializa con la tabla `templates` sin errores
3. ✅ La plantilla de prueba se inserta correctamente via seed
4. ✅ La plantilla se puede leer por ID y retorna datos correctos
5. ✅ Las variables requeridas se almacenan y recuperan como JSON array
6. ✅ El módulo se registra en `lib.rs` con `TemplateState` y comandos en `generate_handler!`
7. ✅ `cargo clippy` sin warnings

## Open Questions

1. ¿Se requiere encriptación de la base de datos (SQLCipher) o es suficiente con almacenamiento local plano? → **Por ahora, sin encriptación. Se puede agregar después.**
2. ¿Las plantillas son solo de lectura para el usuario final o también se pueden crear/editar? → **Por ahora, solo lectura + seed. CRUD completo puede ser un issue futuro.**
3. ¿Hay un set fijo de categorías o son dinámicas? → **Dinámicas (campo TEXT libre).**
