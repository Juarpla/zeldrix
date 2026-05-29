# Spec: Issue 31 - Native XLSX Export From AI Tables

## Objective
Build a native backend export path that turns the current editable structured results table into a valid Microsoft Excel `.xlsx` workbook. Users should press "Descargar Excel" from the table UI and receive a local file that opens in Excel without format warnings while preserving the visible column order and row values.

Assumptions:
- The export source is the Issue 30 in-memory edited table state, not the original model JSON.
- The first version writes one worksheet named `Resultados`.
- No new Rust dependency is required; the backend can assemble the OpenXML workbook with the existing `zip` crate.

## Tech Stack
- Frontend: Next.js + React + TypeScript client component
- Desktop bridge: Tauri v2 `invoke`
- Backend: Rust
- XLSX container: native OpenXML parts zipped with the existing `zip` dependency

## Commands
- Build: `npm run build`
- Rust tests: `cd src-tauri && cargo test`
- Dev: `npm run dev`

## Project Structure
- `src-tauri/src/table_xlsx_exporter.rs` contains the export request/response types, workbook writer, filename handling, and unit tests.
- `src-tauri/src/lib.rs` registers the Tauri command.
- `src/lib/export-service.ts` exposes the typed frontend invoke helper.
- `src/components/StructuredResultsTable.tsx` adds the export button using current edited state.
- `specs/SPEC-ISSUE-31-NATIVE-XLSX-EXPORT.md` stores this contract.

## Code Style
Use explicit request/response structs and small helper functions. Keep the IPC boundary owned and serializable, and keep workbook assembly testable without Tauri app state.

```rust
pub fn build_table_xlsx(table: &ExportTable) -> Result<Vec<u8>, TableXlsxExportError> {
    validate_table(table)?;
    write_workbook_parts(table)
}
```

## Testing Strategy
- Unit-test that a valid table export returns ZIP/XLSX bytes with expected workbook entries.
- Unit-test XML escaping for headers and cell values.
- Unit-test rejection of empty columns or empty output rows.
- Run `cd src-tauri && cargo test`.
- Run `npm run build` to verify frontend and Tauri bindings compile.

## Boundaries
- Always: preserve visible column order, preserve edited cell text exactly, generate a unique local `.xlsx` path, and style headers cleanly.
- Ask first: adding a new crate, changing the structured table JSON schema, or introducing a save dialog/plugin workflow.
- Never: require network access, shell out to Office/LibreOffice, write malformed ZIP/XML parts, or silently export an empty table.

## Success Criteria
- A Tauri command `export_structured_table_xlsx(request)` exists and is registered.
- The frontend table shows a "Descargar Excel" action.
- The command writes a valid `.xlsx` file locally and returns its path.
- The workbook contains one worksheet with headers in row 1 and data beginning row 2.
- Header cells use a clean bold/fill style.
- Cell values and column order match the current visible table state.
- Rust tests and production build pass.

## Open Questions
- Should a later version offer a save dialog so the user can choose the destination interactively?
