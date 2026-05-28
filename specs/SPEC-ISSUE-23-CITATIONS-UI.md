# Spec: Issue 23 — Semantic Traceability & Citations UI

## Objective

Build a complete end-to-end citation system that traces AI responses back to their source documents. When the retrieval engine (`retrieve_relevant_context`) provides document fragments that the AI uses to generate a response, each referenced fragment is surfaced in the chat UI as an interactive badge. Clicking the badge opens a read-only audit panel showing the exact source text fragment and its provenance metadata (file name, chunk ID, similarity score).

**User story:** As an operator auditing AI-generated answers, I click a citation badge on any AI message and immediately see the original text segment from the source document so I can verify the answer is grounded in real data.

---

## Tech Stack

- **Frontend:** React + TypeScript (Next.js App Router, `'use client'` where needed)
- **Backend:** Rust via Tauri v2 commands
- **Styling:** Tailwind CSS utility classes (as used throughout the project)
- **IPC:** `invoke` from `@tauri-apps/api/core`

---

## Commands

```
Dev:   npm run dev
Build: npm run build
Test:  cargo test  (Rust unit tests)
Lint:  npx eslint src/
```

---

## Project Structure

### New files
```
src/components/CitationBadge.tsx          → Interactive inline badge rendered inside AI messages
src/components/CitationDrawer.tsx         → Slide-in panel showing source fragment on badge click
src/lib/citation-types.ts                 → Shared TypeScript types for the citations system
```

### Modified files
```
src/components/ChatMessage.tsx            → Render CitationBadge list below AI text; accept citations prop
src/components/MultimodalChat.tsx         → Pass citations returned by backend to ChatMessage
src/lib/multimodal.ts                     → Extend MultimodalMessage with optional citations array
src-tauri/src/retrieval_engine/mod.rs     → Add page_number field to RetrievalResult; expose get_citation_fragment command
src-tauri/src/lib.rs                      → Register get_citation_fragment command in invoke_handler
```

---

## Data Model

### Backend — Extended `RetrievalResult`

```rust
pub struct RetrievalResult {
    pub id: String,          // chunk ID (already exists)
    pub text: String,        // source text fragment (already exists)
    pub file_path: String,   // full path to source file (already exists)
    pub similarity: f32,     // cosine similarity score (already exists)
    pub page_number: Option<u32>,  // NEW: page hint from ingestion metadata (None if unavailable)
}
```

### Frontend — `Citation` type (`src/lib/citation-types.ts`)

```typescript
export interface Citation {
  chunkId: string;
  filePath: string;
  fileName: string;        // basename extracted from filePath
  pageNumber: number | null;
  fragmentText: string;
  similarityScore: number; // 0–1
}
```

### Frontend — Extended `MultimodalMessage` (`src/lib/multimodal.ts`)

```typescript
export interface MultimodalMessage {
  role: 'user' | 'assistant';
  content: ContentPart[];
  citations?: Citation[];   // NEW: only present on assistant messages
}
```

---

## Feature Behaviour

### Backend — `get_citation_fragment` Tauri command

- **Input:** `chunk_id: String`
- **Output:** `Result<RetrievalResult, String>`
- **Purpose:** Allows the frontend to fetch the full fragment text on demand by chunk ID, querying `VectorDbState` directly. This keeps the initial chat response payload lightweight and avoids sending all fragment text upfront; the drawer fetches on click.
- **Fallback:** If the chunk cannot be found in the vector DB, return an `Err` with a human-readable message.

### Frontend — Citation flow

1. `MultimodalChat` calls `retrieve_relevant_context` before sending to `chat_complete_multimodal`.
2. The returned `RetrievalResult[]` is stored as `citations: Citation[]` on the assistant `MultimodalMessage`.
3. `ChatMessage` receives the `citations` array and renders a `CitationBadge` for each item below the message text.
4. Clicking a `CitationBadge` calls `get_citation_fragment(chunkId)` and opens `CitationDrawer` with the result.
5. `CitationDrawer` is a fixed right-side panel (or bottom sheet on small viewports) that shows:
   - File name and full path
   - Page number (if available)
   - Similarity score as a percentage
   - Full fragment text in a styled read-only block
   - A close button

### CitationBadge visual design

- Pills/chips with a document icon (`📄`) and short label: `manual_2025.pdf – Pág. 4` or `manual_2025.pdf` if no page info.
- Color-coded by similarity: `≥ 0.85` → green tint, `0.65–0.84` → amber tint, `< 0.65` → muted gray.
- Hover tooltip shows full file path + similarity percentage.
- `aria-label` and `role="button"` for accessibility.

---

## Code Style

- All identifiers, logs, and comments in English only.
- No inline comments unless strictly necessary — use descriptive names.
- Split logic into small focused helpers (e.g., `extractFileName`, `similarityToColorClass`).
- No `any` types; use strict TypeScript throughout.

---

## Testing Strategy

**Rust unit tests** (inside `retrieval_engine/mod.rs`):
- `test_get_citation_fragment_returns_known_chunk`: Insert a record into an in-memory `VectorDatabase`, call the internal query logic, assert the returned `RetrievalResult` fields match.
- `test_get_citation_fragment_missing_chunk_returns_error`: Query a non-existent chunk ID; assert `Err` is returned.

**Manual verification checklist:**
1. Send a message in the chat that triggers RAG context retrieval.
2. Confirm citation badges appear beneath the AI reply.
3. Click a badge → drawer opens with correct file name and fragment text.
4. Confirm the drawer closes on Escape key and on the close button.
5. Confirm no badge appears on user messages.
6. Confirm badge colors change correctly with varying similarity scores.

---

## Boundaries

- **Always:** Keep `RetrievalResult` backwards-compatible (add `page_number` as `Option<u32>` with a default of `None` so existing callers are unaffected).
- **Always:** Sanitize fragment text before rendering (treat as plain text, not HTML).
- **Ask first:** Any change to the `retrieve_relevant_context` command signature beyond adding `page_number` to the result.
- **Never:** Store raw source document bytes in state; only store fragment text already present in the vector DB record.

---

## Success Criteria

1. AI assistant messages in `MultimodalChat` display one `CitationBadge` per retrieved context fragment.
2. Clicking a badge opens `CitationDrawer` within 300 ms (local in-memory DB lookup).
3. `CitationDrawer` shows: file name, page number (when available), similarity percentage, and the full fragment text.
4. Color coding of badges accurately reflects the similarity tier (green/amber/gray).
5. The drawer is keyboard-accessible: focus trap, Escape closes it.
6. Rust unit tests for `get_citation_fragment` pass with `cargo test`.
7. No TypeScript errors; `npx eslint src/` reports zero new errors.
8. `page_number: None` in existing vector DB records causes no runtime errors; the badge simply omits the page reference.
