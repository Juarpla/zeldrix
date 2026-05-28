# Spec: Issue 25 - Background Local Network Sync Service

## Objective
Develop a completely background file watcher service in Rust that monitors a specified local or network shared folder (such as a company NAS or Windows shared folder). When a new document (specifically `.txt` files for this feature, but expandable to other supported formats) is added, the application automatically detects it, extracts its text content, chunks the text, generates high-dimensional embeddings using the local llama.cpp server, and indexes the vectors in the local vector database—all without user intervention.

### Acceptance Criteria
- Given a configured monitored directory, saving a new `.txt` file in that folder automatically triggers background detection.
- App logs must confirm within **5 seconds** of file creation that the document has successfully entered the indexing queue without user intervention.
- The document is parsed, chunked, embedded, and indexed into the local `VectorDatabase` as soon as the local llama.cpp embedding sidecar is active.
- Emits real-time synchronization status updates to the frontend via Tauri events.

## Tech Stack
- **Backend:** Rust, Tauri command architecture
- **File Watching:** `notify` (v6.1.1 or compatible)
- **Asynchronous Runtime:** Tokio (channels for non-blocking in-memory queue)
- **Local DB:** VectorDatabase (`vector_db.json`)
- **Model Interface:** llama.cpp embeddings API (`/v1/embeddings`)

## Commands
- Run Rust Tests: `cargo test`
- Build Tauri App: `npm run build`
- Dev Server: `npm run dev`

## Project Structure
We will create or modify the following files:
- `src-tauri/Cargo.toml` [MODIFY]: Add `notify` dependency.
- `src-tauri/src/document_ingestion/sync_service.rs` [NEW]: Implement the background file watcher thread, synchronization queue, and background processing worker.
- `src-tauri/src/document_ingestion/mod.rs` [MODIFY]: Export and integrate the sync service module.
- `src-tauri/src/lib.rs` [MODIFY]: Initialize and manage the background sync service state and register new Tauri commands (`get_monitored_folder`, `set_monitored_folder`, `get_sync_status`, `get_sync_queue`).
- `src-tauri/capabilities/default.json` [MODIFY]: Grant required permissions for files and folders if needed.

## Code Style
- Follow the rules in `AGENTS.md`: English only, self-documenting code, short modular functions, minimal comments.
- Thread-safe and async-safe patterns using Tokio channels (`mpsc::channel`), `Arc`, and `Mutex` / `RwLock`.

## Testing Strategy
- **Unit & Integration Tests:** Write tests for queue insertion, file discovery, and indexing operations.
- **Manual Verification:** Configure a temporary directory as the monitored folder, write a `.txt` file to it, and assert in the app logs that the file enters the queue in less than 5 seconds and gets indexed.

## Success Criteria
1. Adding the `notify` dependency does not break the project build.
2. The background sync worker correctly monitors a configured path.
3. Adding a new `.txt` file triggers automatic queue insertion within 5 seconds (typically instant).
4. The background queue is processed automatically, generating embeddings and inserting into the database when the sidecar is active.

## Open Questions
- Should we monitor subfolders recursively? Yes, `notify` supports recursive monitoring, which is ideal for corporate shared folders (NAS).
