# Spec: Issue 35 - User Custom Automation Presets

## Objective
Allow advanced company users to create custom automation cards from the automation hub. A preset stores a visible title, corporate icon, hidden base prompt, and required output component type (`text` or `table`). After saving, the preset appears immediately as an interactive card in the main automation panel and runs the saved prompt against dropped or pasted text.

## Tech Stack
- Frontend: Next.js App Router, React 19, TypeScript, Tailwind CSS, Framer Motion.
- Desktop bridge: Tauri v2 IPC through `@tauri-apps/api/core`.
- Backend: Rust 2021, rusqlite SQLite persistence, local llama.cpp sidecar compatible with OpenAI chat completions.

## Commands
- Build frontend: `npm run build`
- Run desktop/dev app: `npm run dev`
- Rust tests: `cd src-tauri && cargo test`

## Project Structure
- `src/components/Automations/AutomationHub.tsx` renders the automation grid, filters, and creation form trigger.
- `src/components/Automations/AutomationCard.tsx` renders each card and its visual icon.
- `src/components/Automations/WorkflowExecution.tsx` executes a selected shortcut and renders results.
- `src/lib/automations-data.ts` defines the shared automation card contract and factory cards.
- `src/lib/custom-automation-presets-service.ts` provides frontend IPC helpers for user presets.
- `src-tauri/src/custom_automation_presets/` owns SQLite schema, CRUD helpers, prompt execution, and tests.
- `src-tauri/src/lib.rs` registers Tauri state and commands.

## Code Style
Use small typed boundaries and English identifiers.

```typescript
const savedPreset = await createCustomAutomationPreset({
  title,
  iconName,
  basePrompt,
  outputType,
});
```

```rust
pub fn create_custom_preset(
    conn: &Connection,
    input: CustomAutomationPresetInput,
) -> PresetResult<CustomAutomationPreset> {
    validate_preset_input(&input)?;
    // Insert and return the persisted row.
}
```

## Testing Strategy
- Add Rust unit tests for SQLite table creation, input validation, insert/list behavior, and retrieval by id.
- Build verification must cover TypeScript type safety and Tauri command registration through `npm run build` and `cd src-tauri && cargo test`.
- Manual verification: create a preset from the hub, confirm it appears immediately, open it, paste or drop text, execute, and confirm the generated output uses the hidden base prompt.

## Boundaries
- Always: validate title, icon name, base prompt, and output type before persisting; keep prompt text hidden from the card; use owned command arguments for async Tauri calls.
- Ask first: adding new dependencies, replacing the automation hub layout, changing existing factory automation behavior beyond integration points.
- Never: store secrets in presets, call remote services for execution, remove existing factory automations, or expose the hidden base prompt on the card grid.

## Success Criteria
1. A user can open a dynamic creation form from the automation hub and save a custom preset with title, corporate icon, hidden base prompt, and output type.
2. Saving a preset persists it to SQLite and immediately appends a new custom automation card in the hub without a page reload.
3. The custom card can accept pasted text or text dropped onto its input area.
4. Executing the custom card sends the dropped/pasted text to the local sidecar with the saved base prompt and displays the result.
5. Text presets render a prose result; table presets render table HTML when the model returns table-like content or a structured fallback when needed.
6. Existing factory automations continue to render and execute as before.

## Open Questions
- The exact corporate icon library is not defined, so this implementation uses a closed internal icon set based on existing hub icon styles.
- The existing Tauri setup uses an in-memory templates database; this feature follows that local SQLite state pattern unless a later persistence-path requirement is provided.
