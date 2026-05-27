# Spec: Issue 22 - Extended Context Window Prompt Injection Pipeline for Gemma 4

## Objective
Develop a final prompt packager/formatter in the Rust backend of the Zeldrix application. This module will:
1. Accept a list of retrieved document fragments (context chunks), a user query, a system instruction template, and prompt configuration limits.
2. Structure the prompt cleanly to isolate the system instructions, the retrieved context documents with metadata, and the user's question.
3. Instruct the model (Gemma 4) to rely strictly on the provided context and respond only if the answer is present, preventing hallucination.
4. Manage token/character budgets dynamically, prioritizing the system instructions and user query, and fitting as many highly-relevant context chunks as possible within the configured limit, utilizing Gemma 4's extended context window (up to 128K tokens).

## Tech Stack
- Backend: Rust (Tauri command ecosystem)
- Serialization: `serde`, `serde_json`
- State/Utility Integration: Reuses `TokenEstimator` from `document_ingestion` if applicable, or implements a robust standalone string token estimator.

## Commands
- Run Tests: `cargo test`
- Build Tauri App: `npm run build`

## Project Structure
We will modify and create the following files:
- `src-tauri/src/retrieval_engine/prompt_packer.rs` [NEW]: Implements the core prompt packing logic, budget calculation, system template formatting, and tests.
- `src-tauri/src/retrieval_engine/mod.rs` [MODIFY]: Register the new packer module, expose the prompt packaging logic, and add a Tauri command.
- `src-tauri/src/lib.rs` [MODIFY]: Register the Tauri command `pack_context_prompt` in the main handler.

## Code Style
- **Strict English**: All identifiers, code, and necessary documentation/logs must be in English.
- **Self-Documenting Code**: Keep function and variable names extremely descriptive. No redundant comments.
- **Helper Functions**: Split complex packing logic into small, testable helpers (e.g., token estimation, prompt formatting, document formatting).

## Testing Strategy
We will implement unit and integration tests inside `src-tauri/src/retrieval_engine/prompt_packer.rs` to verify:
1. Dynamic document selection: Chunks that exceed the token budget are omitted, while chunks within the budget are successfully packed in order of descending relevance.
2. Complete prompt structure correctness: The prompt output formats the documents cleanly with their corresponding metadata (e.g., document number, file path, ID) and correctly isolates the user's query at the end.
3. Strict instructions: The prompt successfully includes the default system template that instructs Gemma 4 to answer only using the provided documents and to refuse to answer if the context lacks information.
4. Edge cases: Empty context, extremely small budgets, large context lists up to thousands of tokens, and correct handling of different token estimators.

## Boundaries
- **Always**: Keep the user query and system instruction template intact. If the budget is too small to fit even the system instructions and query, return an error.
- **Never**: Hardcode system prompts without allowing customizable system instruction overrides. Expose clear defaults while letting the caller provide custom templates.

## Success Criteria
1. The Rust function `pack_context_prompt` successfully packages context chunks, a user query, and system instructions into a clean formatted string.
2. If the total estimated tokens exceed the configured maximum, the packager correctly discards lower-relevance documents first until the final prompt fits the budget.
3. Expose a Tauri command `format_inference_prompt` so that the frontend can format prompts before invoking the AI.
4. Tests run and pass cleanly without compile errors or lint warnings.
