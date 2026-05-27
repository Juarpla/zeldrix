# Spec: Issue 16 - Pipeline local de ingesta y extraccion multi-formato

## Objective
Implementar un extractor local de documentos corporativos para el futuro buscador inteligente RAG. El backend debe leer archivos del disco y devolver texto normalizado en memoria sin llamadas externas ni dependencias de red en tiempo de ejecucion.

El usuario objetivo es quien arrastra documentos locales a Zeldrix para indexarlos. El exito inicial es que un PDF digital de 50 paginas con texto extraible se procese en menos de 2 segundos en una maquina estandar.

## Tech Stack
- Backend: Rust en Tauri v2.
- PDF digital: extraccion nativa con crate Rust.
- DOCX: lectura del paquete OOXML local.
- XLSX: lectura de hojas de calculo con parser Rust.
- Texto plano: lectura local UTF-8/UTF-8-lossy.
- Frontend boundary: comando Tauri invocable desde TypeScript.

## Commands
- Build frontend: `npm run build`
- Rust tests: `cargo test --manifest-path src-tauri/Cargo.toml`
- Tauri compile check: `cargo check --manifest-path src-tauri/Cargo.toml`
- Full build: `npm run build`

## Project Structure
- `src-tauri/src/document_ingestion/` contains extractors, errors, result types, and tests.
- `src-tauri/src/lib.rs` registers the Tauri command.
- `src/lib/` may contain a thin TypeScript wrapper for invoking extraction.
- `specs/` contains this feature specification.

## Code Style
Rust extraction helpers remain pure and testable. Tauri commands convert typed errors into strings only at the IPC boundary.

```rust
pub fn extract_text_from_path(path: &Path) -> Result<ExtractedDocument, IngestionError> {
    match detect_document_format(path)? {
        DocumentFormat::PlainText => extract_plain_text(path),
        DocumentFormat::Pdf => extract_pdf(path),
        DocumentFormat::Docx => extract_docx(path),
        DocumentFormat::Xlsx => extract_xlsx(path),
    }
}
```

## Testing Strategy
- Unit-test format detection and unsupported extensions.
- Unit-test plain text extraction with temporary files.
- Unit-test DOCX and XLSX extraction using generated minimal fixtures.
- Unit-test scanned/image-only PDF behavior as a clear local error when no OCR text layer exists.
- Keep unrelated existing tests green when running the required full Rust suite.
- Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- Run `cargo check --manifest-path src-tauri/Cargo.toml`.

## Boundaries
- Always: keep all processing local; return structured metadata with extracted text.
- Always: validate paths and extensions before parsing.
- Always: fail with actionable errors for unsupported, unreadable, encrypted, or image-only documents.
- Ask first: adding a heavyweight native OCR engine, spawning external binaries, changing app permissions, or storing indexed content on disk.
- Never: upload documents, call cloud OCR, silently drop parser errors, or block on sidecar LLM availability.

## Success Criteria
- `extract_document_text` accepts a local file path and returns extracted text plus format metadata.
- `.txt`, `.md`, `.csv`, `.json`, `.pdf`, `.docx`, and `.xlsx` files are routed to local extractors.
- PDF digital extraction preserves the full text content exposed by the PDF text layer.
- Image-only/scanned PDFs return a clear OCR-required error instead of pretending extraction succeeded.
- DOCX extraction includes paragraph text from `word/document.xml`.
- XLSX extraction includes non-empty cell values grouped by sheet.
- Unsupported extensions return a clear error.
- Rust tests cover success and error paths.
- The full Rust test suite is not left blocked by unrelated stale fixtures discovered during verification.
- The implementation compiles with `cargo check --manifest-path src-tauri/Cargo.toml`.

## Open Questions
- Full OCR for scanned PDFs requires choosing and bundling a local OCR engine. This spec treats image-only PDFs as detected-but-not-extracted until that dependency is approved.
