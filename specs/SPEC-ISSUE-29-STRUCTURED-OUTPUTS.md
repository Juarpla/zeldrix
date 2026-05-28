# Spec: Issue 29 - Strict JSON Table Structured Outputs

## Objective
Implement a backend extraction path that converts unstructured expense documents into a strict JSON table string. The model request must use llama.cpp's OpenAI-compatible structured output support so Gemma 4 is constrained to the table schema, and the Rust backend must validate and normalize the returned content before exposing it.

Assumptions:
- The app already communicates with local `llama-server` through `/v1/chat/completions`.
- The first version is backend-only and exposes a Tauri command for future UI workflows.
- The table architecture is `columns` plus `rows`, where each row stores typed cells aligned to column names.

## Tech Stack
- Backend: Rust + Tauri v2
- Model Runtime: local Gemma 4 through llama.cpp
- Serialization: `serde` and `serde_json`
- HTTP: `reqwest`

## Commands
- Build: `npm run build`
- Rust tests: `cd src-tauri && cargo test`
- Dev: `npm run dev`

## Project Structure
- `src-tauri/src/structured_extraction/mod.rs` contains schema construction, prompt construction, output validation, and the Tauri command.
- `src-tauri/src/lib.rs` registers the module and command.
- `specs/SPEC-ISSUE-29-STRUCTURED-OUTPUTS.md` stores this contract.

## Code Style
Use small Rust helpers with descriptive English names. Tauri commands return `Result<T, String>` at the IPC boundary while pure helpers return typed or string errors that are straightforward to test.

```rust
pub fn normalize_table_json(raw_content: &str) -> Result<String, String> {
    let table: ExtractedTable = serde_json::from_str(raw_content)
        .map_err(|error| format!("Model did not return valid table JSON: {error}"))?;
    validate_table(&table)?;
    serde_json::to_string(&table).map_err(|error| error.to_string())
}
```

## Testing Strategy
- Unit-test schema payload shape without requiring a running model.
- Unit-test valid table JSON normalization.
- Unit-test rejection of conversational text or malformed table JSON.
- Run `cd src-tauri && cargo test`.

## Boundaries
- Always: keep inference local, validate model output before returning it, return only normalized JSON strings from the command.
- Ask first: adding new dependencies, changing the frontend workflow, changing persisted data schemas.
- Never: return conversational model text from the strict extraction command, silently coerce invalid JSON into partial data, or require network access at runtime.

## Success Criteria
- A Tauri command `extract_structured_table_json(document_text: String) -> Result<String, String>` exists.
- The command sends llama.cpp a strict JSON schema via `response_format`.
- The schema represents table columns and rows with typed cell values.
- The command returns a string that parses as valid JSON and contains only the table object.
- Invalid or conversational model output is rejected instead of returned.
- Unit tests cover valid normalization and invalid conversational residue.

## Open Questions
- Should future UI flows map this schema directly to an editable grid, or transform it into an existing app table model first?
