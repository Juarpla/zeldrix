# SPEC.md — Issue #05: Web Search Fallback System

## 1. Concepto & Visión

Sistema de verificación basado en Tool Calling nativo de Gemma 4. Cuando el modelo exprese baja confianza o detecte consultas sobre eventos en tiempo real, dispara automáticamente una llamada a `web_search()`, obtiene los snippets relevantes, los inyecta en el contexto de 128K y genera la respuesta final sin alucinaciones.

El flujo es: **User Query → Model Low Confidence → Tool Call → Web Search → Inject Results → Final Response**

## 2. Arquitectura Técnica

### 2.1 Flujo de Function Calling

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  User Msg   │────▶│  Llama.cpp   │────▶│  Tool Call  │────▶│  Web Search  │────▶│  Re-inject   │
│ (real-time) │     │  (Gemma 4)   │     │  detected   │     │  + snippets  │     │  + Final Rx │
└─────────────┘     └──────────────┘     └─────────────┘     └──────────────┘     └─────────────┘
```

### 2.2 Componentes

| Componente | Archivo | Responsabilidad |
|------------|---------|-----------------|
| Tool Schema | `function_calling/schema.rs` | Define JSON schema para `web_search` |
| Web Search Executor | `function_calling/web_search.rs` | Ejecuta búsqueda ligera vía API local |
| Tool Handler | `function_calling/handler.rs` | Parsea tool_calls, ejecuta, retorna resultados |
| Chat Completion | `multimodal/mod.rs` | Loop de function calling, reinyección de contexto |

### 2.3 Tool Definition (JSON Schema)

```json
{
  "type": "function",
  "function": {
    "name": "web_search",
    "description": "Search the web for current information. Use when the user's query requires real-time data, stock prices, weather, news, or any information that may have changed after your training date. The query should be concise and focused.",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "The search query to look up on the web. Be specific and concise."
        }
      },
      "required": ["query"]
    }
  }
}
```

## 3. Especificación de APIs

### 3.1 Web Search API (Rust → External)

```rust
// src-tauri/src/function_calling/web_search.rs
pub async fn web_search(query: &str) -> Result<WebSearchResult, String>
```

**WebSearchResult structure:**
```rust
struct WebSearchResult {
    query: String,
    snippets: Vec<SearchSnippet>,  // Max 5 snippets
    timestamp: String,
}

struct SearchSnippet {
    title: String,
    url: String,
    content: String,  // Max 200 chars
}
```

### 3.2 Tool Call Loop

La función `chat_complete_multimodal` ahora:

1. Envía request inicial con `tools: [web_search_schema]`
2. Si respuesta tiene `tool_calls`:
   - Extrae el tool y argumentos
   - Ejecuta `web_search()`
   - Añade mensaje del sistema con resultados: `"[WEB SEARCH RESULTS]\n{snippets}\n[/WEB SEARCH RESULTS]"`
   - Añade mensaje `tool` con el resultado
   - Reenvía al modelo para respuesta final
3. Si respuesta tiene `content` directo: retorna ese content

### 3.3 System Prompt (Inyectado)

Para activar detección de baja confianza, el prompt del sistema incluye:

```
When answering questions about current events, stock prices, weather,
sports scores, or any information that may have changed after your
training data, you MUST use the web_search function.
If you are unsure about the current value of something, use web_search.
```

## 4. Dependencias

### Cargo.toml additions:
```toml
# Web scraping
scraper = "0.21"  # HTML parsing
regex = "1"       # Text extraction

# Async HTTP (already have reqwest)
```

## 5. Casos de Prueba

### Test Case 1: Query de acciones (Aceptación)
**Input:** "¿Cuál es el precio actual de las acciones de Apple el día de hoy?"
**Expected:**
1. Modelo detecta necesidad de datos reales
2. Activa tool_call con `web_search`
3. Backend ejecuta búsqueda y retorna snippets
4. Modelo genera respuesta con datos reales (no alucina)
5. Output incluye precio real y fecha de la búsqueda

### Test Case 2: Conocimiento interno
**Input:** "¿Quién escribió Don Quijote?"
**Expected:**
1. Modelo no activa tool_call (tiene la info)
2. Responde directamente con "Miguel de Cervantes"

### Test Case 3: Múltiples tool calls
**Input:** "¿Qué tiempo hace en Madrid y cuál es el precio de Bitcoin?"
**Expected:**
1. Dos tool_calls sequentially ejecutados
2. Ambos resultados inyectados
3. Respuesta final con ambos datos

## 6. Límites y Restricciones

- Máximo 5 snippets por búsqueda (límite de tokens)
- Máximo 200 caracteres por snippet
- Timeout de 10 segundos para web search
- Máximo 3 iteraciones de tool call loop (prevenir loops infinitos)
- Fallback: si web search falla, retornar error claro al usuario

## 7. Archivos a Crear/Modificar

| Acción | Archivo |
|--------|---------|
| CREATE | `src-tauri/src/function_calling/mod.rs` |
| CREATE | `src-tauri/src/function_calling/schema.rs` |
| CREATE | `src-tauri/src/function_calling/web_search.rs` |
| CREATE | `src-tauri/src/function_calling/handler.rs` |
| MODIFY | `src-tauri/src/multimodal/mod.rs` |
| MODIFY | `src-tauri/src/multimodal/types.rs` |
| MODIFY | `src-tauri/src/lib.rs` |
| MODIFY | `src-tauri/Cargo.toml` |