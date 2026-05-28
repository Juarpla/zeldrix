# Spec: SPEC-ISSUE-28-THINKING-MODE (Thinking Mode Pipeline)

## Objective
The objective is to implement a structured reasoning (Thinking Mode) pipeline for Zeldrix. When a user submits a chaotic text about a delayed or complex project, Zeldrix will invoke native Thinking Mode capabilities of Gemma 4. 
The system prompt will force the model to chronologically break down the information from the mail/text, evaluate implicit responsibilities, and generate three strict lists:
1. **Acuerdos** (Agreements)
2. **Conflictos** (Conflicts/Risks)
3. **Matriz de Responsabilidades** (Responsibility Matrix with deadlines)

The backend must capture the `<thinking>` tag (or similar thinking block format) from the model separately, so that the raw thinking process is isolated from the final structured result. The UI must then display the final structured result and potentially the thinking flow in a clear, premium, separated format.

## Tech Stack
- Frontend: React + TypeScript + Tailwind CSS (Vite / Next.js)
- Backend: Rust + Tauri v2
- Model: Gemma 4 via llama.cpp (running locally)

## Commands
- Build: `npm run build`
- Dev: `npm run dev`
- Rust Tests: `cd src-tauri && cargo test`

## Project Structure
- `src-tauri/src/lib.rs` (tauri commands registration)
- `src-tauri/src/thinking_mode/mod.rs` [NEW] (core logic for parsing thinking tags and structured prompts)
- `src/lib/automations-data.ts` (adding/updating automation shortcut data)
- `src/components/Automations/WorkflowExecution.tsx` (modifying UI execution workflow to support thinking mode displays and real LLM invocation)

## Code Style
Idiomatic Rust following `rust-best-practices` and clean TypeScript following `next-best-practices`.
No unnecessary comments. Use self-documenting naming.

## Success Criteria
- A Tauri command `ai_analyze_thinking_mode(text: String) -> Result<ThinkingModeResponse, String>` is implemented.
- The command calls the llama.cpp server using a specific system prompt to trigger thinking and structuring.
- The command parses out the `<thinking>...</thinking>` (or `<thought>...</thought>`) section and separates it from the final response.
- The backend parses/returns a structured response containing the agreements (Acuerdos), conflicts (Conflictos), and the matrix of responsibilities (Matriz de Responsabilidades) with deadlines, as well as the thinking log.
- An automation shortcut is registered or modified in the frontend (e.g. `structure-minutes` or a new `thinking-minutes` shortcut) that triggers this command.
- The UI handles the loading/processing states and displays both the thinking process and the final structured lists in a clean, highly polished dark/glassmorphic UI.

## Open Questions
- Should the frontend call the existing `ai_transform_text` or a new specialized `ai_analyze_thinking_mode`?
  - *Decision:* We will add a dedicated Tauri command `ai_analyze_thinking_mode` to handle the specific thinking parser logic, making it robust and modular.
