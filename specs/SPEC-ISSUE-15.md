# Spec: Issue 15 - Historial local con diff visual de IA

## Objective
Crear una vista de auditoria interna para el editor de documentos. Cuando una accion de IA modifica el documento, la app guarda una instantanea local con el contenido anterior y posterior. El usuario puede revisar versiones, ver un diff visual de texto eliminado en rojo y texto agregado en verde, y revertir el documento a la version anterior con un solo boton.

## Tech Stack
- Frontend: Next.js client components, React 19, TypeScript, TipTap editor.
- Backend: Tauri v2 commands in Rust.
- Storage: JSON local bajo el directorio de datos de la app usando APIs de path de Tauri.

## Commands
- Build frontend: `npm run build`
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Tauri compile check: `cargo check --manifest-path src-tauri/Cargo.toml`
- Dev server: `npm run dev`

## Project Structure
- `src/app/editor/page.tsx` contains the document editor shell and state.
- `src/components/Editor/` contains TipTap editor UI and AI text actions.
- `src/components/VersionHistory/` will contain the audit panel and diff rendering.
- `src/lib/` contains frontend service wrappers and shared types.
- `src-tauri/src/` contains Tauri commands and local storage modules.
- `specs/` contains feature specifications.

## Code Style
Use explicit typed boundaries between frontend and Tauri commands.

```ts
export async function saveDocumentVersion(input: SaveVersionInput): Promise<DocumentVersion> {
  return invoke<DocumentVersion>("document_version_save", { input });
}
```

Rust commands return `Result<T, String>` and delegate file handling to testable helper functions.

## Testing Strategy
- Unit-test the Rust storage module with temp directories and JSON read/write behavior.
- Unit-test the frontend diff algorithm as a pure function.
- Run `npm run build` to verify TypeScript and Next production compilation.
- Run `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo check --manifest-path src-tauri/Cargo.toml`.

## Boundaries
- Always: save snapshots only for successful AI-driven changes, preserve manual editor edits, keep revert explicit.
- Always: cap history reads to a manageable list sorted newest first.
- Ask first: adding a new runtime dependency, changing the editor document format, replacing TipTap.
- Never: send version history off-device, silently revert content, delete unrelated user work.

## Success Criteria
- A successful template merge or AI text transform creates a local version record with previous and new document content.
- The editor shows a history/audit panel listing previous AI changes.
- Selecting a history entry displays removed text in red and added text in green.
- The user can switch between history entries without changing the current document.
- The user can revert a selected AI change with one button, restoring the prior document content and recording that revert locally.
- The feature works when the Tauri backend is available and degrades gracefully in frontend-only/mock mode.

## Open Questions
- The app currently has no persisted document identifier, so Issue 15 will use a stable local editor document id per loaded template or freeform editor session.
