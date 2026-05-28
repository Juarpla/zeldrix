# Spec: Issue 30 - Interactive Structured Results Table

## Objective
Build a frontend table component that receives the strict JSON table object produced by Issue 29 and renders it as an editable, user-correctable results grid. Users must be able to inspect clean cell values immediately, edit values inline, delete rows, and reorder columns with drag and drop.

Assumptions:
- The Issue 29 response shape remains `columns` plus `rows`, with row cells matched to columns by `cell.column`.
- The first integration point is assistant chat output: when an assistant message is valid table JSON, it should render the visual table instead of raw JSON.
- No new dependency is required for this first version; native React state and browser drag-and-drop are enough.

## Tech Stack
- Frontend: Next.js + React + TypeScript
- Styling: local CSS module for the table component
- Runtime integration: existing Tauri/Next frontend, no backend changes

## Commands
- Build: `npm run build`
- Dev: `npm run dev`

## Project Structure
- `src/components/StructuredResultsTable.tsx` contains the interactive table component and exported parser helper.
- `src/components/StructuredResultsTable.module.css` contains scoped styles for the table surface.
- `src/components/ChatMessage.tsx` detects valid structured table JSON in assistant text parts and renders the table.
- `src/lib/types.ts` contains shared TypeScript types for the structured table JSON.
- `specs/SPEC-ISSUE-30-INTERACTIVE-RESULTS-TABLE.md` stores this contract.

## Code Style
Use explicit TypeScript types, small helper functions, and local state updates that keep the table immutable.

```tsx
function validateCellValue(value: string, column: StructuredTableColumn): CellValidation {
  const trimmed = value.trim();
  if (!column.nullable && trimmed.length === 0) {
    return { status: "error", message: "Required value" };
  }
  return { status: "valid" };
}
```

## Testing Strategy
- Use `npm run build` as the required verification for type safety and production compilation.
- Manually verify that valid Issue 29 JSON maps into visible cells.
- Manually verify inline editing, row deletion, and column drag-and-drop behavior in the rendered component.

## Boundaries
- Always: preserve the Issue 29 JSON contract, keep edits local to the UI component, and display validation feedback without blocking user correction.
- Ask first: adding TanStack Table or any drag-and-drop dependency, changing the backend schema, or persisting edited table data.
- Never: mutate the original table prop in place, silently drop cells for known columns, or render malformed JSON as a table.

## Success Criteria
- The component accepts the Issue 29 structured table JSON object and maps it instantly into a tabular UI.
- Cells render clean display values aligned to the declared columns.
- Users can edit cells inline and see reactive basic typography/type validation.
- Users can delete rows.
- Users can reorder columns via drag and drop.
- Assistant chat messages containing valid table JSON render the component instead of raw JSON text.
- `npm run build` passes.

## Open Questions
- Should edited results eventually be exported or sent back into the document editor workflow?
