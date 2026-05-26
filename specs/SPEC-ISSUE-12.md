# SPEC: Motor de Combinación de Correspondencia (Merge Engine)

## Objective

Desarrollar un motor en Rust que tome texto base de plantillas corporativas y reemplace de forma segura los marcadores de posición `{{variable}}` con los datos finales validados por el formulario de la IA, asegurando que la estructura tipográfica del editor no se rompa.

**Problema a resolver:** Cuando la IA genera contenido para una plantilla corporativa, los valores pueden contener caracteres especiales o estructuras que rompen el formato del documento. El motor debe ser resilient a estos casos y preservar la integridad tipográfica.

## ASSUMPTIONS I'M MAKING

1. Las plantillas usan el formato `{{variable_name}}` para marcadores de posición
2. Los nombres de variables son alfanuméricos con guiones bajos (`[a-zA-Z0-9_]+`)
3. Los valores de reemplazo pueden contener cualquier carácter UTF-8
4. El motor debe ser usado desde código Rust, no directamente como binario
5. Se requiere validación de que todos los marcadores fueron reemplazados

## Tech Stack

- **Lenguaje:** Rust (edición 2021)
- **Dependencias:** Ninguna adicional (regex ya está en Cargo.toml)
- **Ubicación:** `src-tauri/src/merge_engine/`

## Commands

```bash
# Build
cd src-tauri && cargo build

# Run tests
cd src-tauri && cargo test merge_engine

# Run clippy
cd src-tauri && cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Project Structure

```
src-tauri/src/merge_engine/
├── mod.rs           # Módulo principal, exports públicos
├── error.rs         # Tipos de errores con thiserror
├── parser.rs        # Parsing de marcadores {{variable}}
├── merger.rs        # Lógica de reemplazo seguro
└── tests.rs         # Tests unitarios

src-tauri/src/lib.rs # Se agregará: mod merge_engine;
```

## Code Style

### Ejemplo de API pública

```rust
use merge_engine::{merge, MergeError, Variables};

let template = "Estimado {{client_name}}:\n\nSu pedido #{{order_id}} está listo.";
let mut vars = Variables::new();
vars.insert("client_name", "Juan Pérez");
vars.insert("order_id", "12345");

let result = merge(&template, &vars).unwrap();
assert!(!result.contains("{{"));
assert!(!result.contains("}}"));
```

### Convenciones

- `Variables` es un `HashMap<String, String>` con helper methods
- `merge()` retorna `Result<String, MergeError>` con `?` propagation
- Errores usan `thiserror` para mensajes descriptivos
- Tests con nombres descriptivos: `merge_should_replace_all_variables()`
- Regex compilado una vez como `static RE` para performance

## API Design

### Tipos públicos

```rust
// src-tauri/src/merge_engine/mod.rs

pub struct Variables(HashMap<String, String>);

impl Variables {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>);
    pub fn get(&self, key: &str) -> Option<&str>;
}

/// Función principal de merge
pub fn merge(template: &str, vars: &Variables) -> Result<String, MergeError>;

/// Verifica si un template tiene marcadores sin resolver
pub fn has_unresolved_placeholders(template: &str) -> bool;
```

### Error types

```rust
// src-tauri/src/merge_engine/error.rs

#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    #[error("Variable '{0}' no proporcionada")]
    MissingVariable(String),

    #[error("Template contiene marcadores sin resolver: {0}")]
    UnresolvedPlaceholders(String),
}
```

### Parser

```rust
// src-tauri/src/merge_engine/parser.rs

/// Extrae todos los nombres de variables de un template
pub fn extract_variables(template: &str) -> Vec<&str>;

/// Verifica si el template tiene marcadores crudos
pub fn has_raw_tokens(template: &str) -> bool {
    static RE: Regex = regex!(r"\{\{[a-zA-Z0-9_]+\}\}");
    !RE.is_match(template)
}
```

### Merger

```rust
// src-tauri/src/merge_engine/merger.rs

/// Realiza el reemplazo seguro, preservando whitespace
pub fn perform_merge(template: &str, vars: &Variables) -> Result<String, MergeError>;
```

## Testing Strategy

### Framework
- Tests unitarios en `src-tauri/src/merge_engine/tests.rs`
- Tests de integración en `src-tauri/tests/merge_engine_integration.rs`

### Coverage requirements
- Todos los públicos API functions deben tener tests
- Casos edge: valores vacíos, valores con `{{}}`, valores con newlines

### Test cases obligatorios

1. **Merge básico:** Template con 5 variables diferentes reemplazadas correctamente
2. **Valores con caracteres especiales:** `<script>`, newlines, tabs preservados
3. **Variables faltantes:** Error descriptivo con nombre de variable
4. **Tokens residuales:** Verificación de que no quedan `{{variable}}` en output
5. **Preservación tipográfica:** Indentación y newlines intactos
6. **Unicode:** Caracteres internacionales preservados

## Boundaries

### Always do
- Verificar que todas las variables requeridas estén presentes antes del merge
- Escapar caracteres especiales en valores si es necesario para seguridad
- Preservar completamente el whitespace original del template
- Retornar error si quedan marcadores sin reemplazar

### Ask first
- Cambiar el formato de marcadores (ej: de `{{}}` a `{{ }}`)
- Agregar features como conditional blocks o loops

### Never do
- Usar `unwrap()` o `expect()` en código de producción
- Modificar el template original (el merge debe ser inmutable)
- Hacer debug print en logs de producción

## Success Criteria

### Criterio de aceptación: 5 variables complejas

Dada una plantilla con 5 variables complejas:

```
Estimado {{client_name}},

Su solicitud de crédito por ${{amount}} ha sido {{status}}.

Detalles:
- Número de referencia: {{reference_id}}
- Fecha de aprobación: {{approval_date}}
- Monto total: {{total_amount}}

Atentamente,
{{agent_name}}
```

Cuando se mergea con variables que contienen:
- Caracteres especiales (`$`, `%`, `&`)
- Newlines múltiples
- Texto muy largo
- Caracteres Unicode

**Entonces:** El documento final:
1. No contiene tokens crudos `{{...}}`
2. Preserva todos los saltos de línea originales
3. Mantiene la indentación del template original
4. Muestra todas las variables reemplazadas correctamente

### Tests de verificación

```rust
#[test]
fn merge_with_5_complex_variables_contains_no_raw_tokens() {
    let template = get_complex_template();
    let vars = get_complex_variables();

    let result = merge(&template, &vars).unwrap();

    assert!(!result.contains("{{"));
    assert!(!result.contains("}}"));
    assert!(result.contains("Juan Pérez"));
    assert!(result.contains("12345"));
}

#[test]
fn merge_preserves_line_breaks_and_indentation() {
    let template = "  {{indent}}\n    {{nested}}";
    let mut vars = Variables::new();
    vars.insert("indent", "level1");
    vars.insert("nested", "level2");

    let result = merge(&template, &vars).unwrap();

    assert!(result.starts_with("  level1"));
    assert!(result.contains("\n    level2"));
}
```

## Open Questions

1. ¿Se requiere sanitización de HTML en los valores de reemplazo para prevenir XSS?
2. ¿El formato de marcadores puede ser configurable (`{{ }}` vs `{{}}`)?
3. ¿Se necesita soporte para valores por defecto: `{{variable:default}}`?