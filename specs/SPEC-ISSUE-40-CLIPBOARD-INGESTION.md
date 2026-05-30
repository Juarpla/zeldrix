# Spec: Issue 40 Clipboard Ingestion

## Objective
Add a quick spotlight command that reads the current operating system clipboard text and sends it to the local inference pipeline. A user can copy text from an external app, open the floating spotlight with Alt+Space, type "Traduce esto", and receive a direct translation of the copied text.

## Tech Stack
- Frontend: Next.js 16, React 19, TypeScript, Tauri JavaScript API.
- Backend: Tauri v2, Rust 2021, llama.cpp sidecar accessed through existing `ai_transform_text`.
- Native clipboard: Rust clipboard crate called from a Tauri command.

## Commands
- Build frontend: `npm run build`
- Build/check Rust backend: `cargo test` from `src-tauri`
- Run app locally: `npm run dev`

## Project Structure
- `src/app/spotlight/page.tsx` contains the floating quick command UI.
- `src-tauri/src/lib.rs` contains Tauri commands and command registration.
- `src-tauri/Cargo.toml` declares native backend dependencies.
- `specs/` stores feature specs.

## Code Style
Use explicit command names and small helpers that describe intent:

```rust
#[tauri::command]
fn read_clipboard_text() -> Result<String, String> {
    let text = read_native_clipboard_text()?;
    Ok(text.trim().to_string())
}
```

TypeScript UI code should keep Tauri IPC calls at event boundaries, handle loading/error states, and avoid speculative abstractions.

## Testing Strategy
- Rust: `cargo test` verifies the backend command compiles and existing Rust tests pass.
- Frontend: `npm run build` verifies TypeScript and Next.js build output.
- Manual: copy text, open spotlight, submit "Traduce esto", and confirm the response is produced from clipboard content.

## Boundaries
- Always: Trim clipboard input, return a clear error for empty or non-text clipboard content, register new Tauri commands in `generate_handler!`.
- Ask first: Broad command language parsing beyond the requested clipboard translation path, replacing the existing AI sidecar API, adding cloud inference.
- Never: Persist clipboard contents, log clipboard contents, silently fall back to unrelated text when clipboard is empty.

## Success Criteria
- The Rust backend exposes a Tauri command that returns the current clipboard text or a safe user-facing error.
- Spotlight recognizes "Traduce esto" and equivalent translate wording as a clipboard-backed translation request.
- Spotlight sends clipboard text to local inference with the existing `translate` action and renders the result.
- Spotlight shows actionable errors when the clipboard is empty, unavailable, or the sidecar is not ready.
- `cargo test` from `src-tauri` and `npm run build` complete successfully, or any failures are documented.

## Open Questions
- None for the initial implementation; advanced natural-language routing can be added in a later issue.
