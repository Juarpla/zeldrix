# Spec: Issue 21 - Relevant Context Retrieval Engine

## Objective
Implement the retrieval engine logic in the Rust backend of the Zeldrix desktop application. The retrieval engine will:
1. Accept a free-text search query from the corporate user.
2. Convert this query into a high-dimensional query vector using the local embedding model via the running `llama.cpp` sidecar.
3. Search the vector database for the top $N$ closest text chunks.
4. Calculate and sort by their relevance (similarity) score.
5. Filter or return the most relevant context chunks.

### Acceptance Criteria
- Given a query like "Políticas de vacaciones" (Vacation policies), the function must return the exact fragments from the Human Resources PDF that discuss free days and vacations, while successfully discarding unrelated content like technical manuals.
- Implement an automated test representing this exact scenario with mock records to verify ranking and accuracy.

## Tech Stack
- Backend: Rust (Tauri command ecosystem)
- HTTP Client: `reqwest` (already used for sidecar calls)
- State Management: `tauri::State` (to access the global `VectorDbState` and `SidecarState`)
- JSON Parsing: `serde` and `serde_json`

## Commands
- Run Rust Tests: `cargo test`
- Build Tauri App: `npm run build`

## Project Structure
We will create and modify the following files:
- `src-tauri/src/retrieval_engine/mod.rs` [NEW]: Implements the core retrieval service, logic, structs, and Tauri commands.
- `src-tauri/src/lib.rs` [MODIFY]: Register the new Tauri commands and hook up the retrieval engine module.

## Code Style
According to the `AGENTS.md` and project coding standards:
- Strictly English for code, variable identifiers, function names, and comments.
- Avoid redundant or cluttering comments; prefer self-documenting, descriptive identifiers.
- Keep functions small and modular.
- Match existing Tauri/Rust state management patterns.

### Example Code Snippet:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResult {
    pub id: String,
    pub text: String,
    pub file_path: String,
    pub similarity: f32,
}
```

## Testing Strategy
- **Unit and Integration Tests:**
  - Implement tests in `src-tauri/src/retrieval_engine/mod.rs` to verify that when documents are added to a temporary `VectorDatabase` representing both HR vacation policies and technical manuals, querying for "Políticas de vacaciones" returns the HR chunks at the top with high relevance, and the technical manual chunks are either ranked lowest or filtered out.
  - Test edge cases: empty database, sidecar offline (returning clear error), large limit values, and exact vector distance alignment.

## Boundaries
- **Always:** Return clear error messages in English if the sidecar is not running or vector db is corrupted.
- **Never:** Include raw llama.cpp port or host config as hardcoded values across the code; retrieve them dynamically from `SidecarState`.

## Success Criteria
1. The Rust command `retrieve_relevant_context` is successfully registered in Tauri.
2. Querying "Políticas de vacaciones" on a test database with mixed HR and technical manuals returns the HR fragments discussing free days and vacations at the top with higher similarity scores, while discarding or ranking manual technical fragments below.
3. Tests run and pass cleanly without any Rust compiler warnings or build failures.

## Open Questions
- Is there a specific threshold score for relevance under which chunks should be discarded entirely, or should we just rely on `limit`?
  *We will allow the frontend to specify a `limit`, and also we can optionally support a minimal similarity threshold (e.g. `min_score`) in the Tauri command for maximum flexibility, defaulting to `0.0` if not provided.*
