# Spec: Issue 32 - Background Batch Processing Queue

## Objective
Build a backend queue for batch invoice extraction so a user can submit many documents at once without overwhelming local CPU/GPU resources. The queue must process jobs in submission order, isolate failures per file, emit progress events for the UI, and continue processing remaining invoices when one item fails.

Assumptions:
- The first implementation targets structured extraction for already-extracted document text, not filesystem PDF parsing from a drag-and-drop payload.
- The backend exposes Tauri commands and emits events that the existing UI can subscribe to for visual error reporting.
- The resource-safe default is one active extraction worker because llama.cpp/Gemma inference is the constrained resource.

## Tech Stack
- Backend: Rust + Tauri v2
- Async queue: `tokio::sync::mpsc`
- Shared state: `std::sync::RwLock`, atomics
- Serialization: `serde`
- Existing extraction engine: `structured_extraction`

## Commands
- Build: `npm run build`
- Rust tests: `cd src-tauri && cargo test`
- Dev: `npm run dev`

## Project Structure
- `src-tauri/src/batch_processing/mod.rs` owns batch queue state, job models, enqueue/status commands, worker startup, and event emission.
- `src-tauri/src/structured_extraction/mod.rs` keeps model request and JSON normalization logic reusable by queue workers.
- `src-tauri/src/lib.rs` manages queue state, starts workers, and registers Tauri commands.
- `specs/SPEC-ISSUE-32-BATCH-PROCESSING-QUEUE.md` stores this contract.

## Code Style
Use small Rust helpers with descriptive English names. Tauri command boundary types are serializable, and pure queue state transitions are tested without requiring a running model.

```rust
pub fn build_batch_items(batch_id: &str, documents: Vec<BatchDocumentInput>) -> Vec<BatchQueueItem> {
    documents
        .into_iter()
        .enumerate()
        .map(|(index, document)| BatchQueueItem::pending(batch_id, index, document))
        .collect()
}
```

## Testing Strategy
- Unit-test FIFO item creation and order indices.
- Unit-test status transitions for pending, processing, completed, and failed items.
- Unit-test that failures are stored on the failed item without removing later pending items.
- Run `cd src-tauri && cargo test`.

## Boundaries
- Always: keep extraction local, process invoice extraction jobs sequentially by default, store per-item status and error messages, emit queue update events after state changes.
- Ask first: adding dependencies, persisting queue history to disk, changing the structured table schema, increasing default parallelism above one worker.
- Never: stop the whole batch because one invoice fails, panic on malformed job input, or hold synchronous locks across async model calls.

## Success Criteria
- A Tauri command can enqueue multiple invoice extraction inputs in one call.
- Enqueued items preserve the original batch order through an `order_index`.
- A background worker processes one item at a time using the existing structured extraction engine.
- A failed invoice item is marked `failed` with an error message and does not block subsequent queued items.
- Completed invoice items store normalized structured table JSON.
- Queue updates are available through a Tauri command and emitted through an event suitable for UI rendering.
- Rust tests cover ordered enqueueing and failure-resilient state transitions.

## Open Questions
- Should future UI drag-and-drop pass file paths for backend text extraction, or should frontend keep extracting/passing text as the first version assumes?
