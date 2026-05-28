# Spec: Issue 24 - Local OCR Module for Scanned Documents and Reports

## Objective
Provide high-performance, completely local optical character recognition (OCR) capabilities within the Zeldrix application. By leveraging the native Image Understanding (multimodal) capacities of the running `gemma-4-E2B-it` model through llama.cpp, users can upload document images, invoices, or blurred screenshots of receipts and extract clean, readable digital text into the system's editor pipeline.

### Acceptance Criteria
- Given a blurred screenshot or image of a corporate receipt/invoice, the Rust backend correctly processes the image through the llama.cpp multimodal server completion pipeline.
- The system returns the extracted legible text in a clean, copyable text area in the UI (within a new "OCR" / "Digitalizar" tab in the AI Assistant Panel).
- The user can instantly transfer the OCR-extracted text to the description/free-text area of the template assistant for entity extraction.
- **Graceful Degradation:** If the active model does not support multimodal inputs (e.g. started without the mmproj file), the "Digitalizar" UI disables image drop/selection options and displays a clear message informing the user that local OCR is disabled and requires a multimodal configuration.

## Tech Stack
- **Backend:** Rust, Tauri command architecture
- **Model Interface:** llama.cpp OpenAI-compatible Chat Completions API
- **Frontend:** React, TypeScript, TailwindCSS/CSS styling, Framer Motion for premium micro-animations

## Commands
- Run Rust Tests: `cargo test`
- Build Tauri App: `npm run build`
- Dev Server: `npm run dev`

## Project Structure
We will create or modify the following files:
- `src-tauri/src/sidecar/mod.rs` [MODIFY]: Add `multimodal` field to `SidecarStatus`.
- `src-tauri/src/multimodal/mod.rs` [MODIFY]: Add the `process_ocr_local` command.
- `src-tauri/src/lib.rs` [MODIFY]: Update `sidecar_status` to determine multimodal active state and register `process_ocr_local`.
- `src/lib/aiService.ts` [MODIFY]: Expose the `multimodal` capability field from sidecar status check.
- `src/components/Editor/AIAbstractPanel.tsx` [MODIFY]: Add the "OCR" (Digitalizar) tab.
- `src/components/Editor/LocalOcrForm.tsx` [NEW]: Create the premium local OCR component with file drop/selection, multimodal status check, processing animations, and action buttons.

## Code Style
- Follow the rules in `AGENTS.md`: English only, self-documenting code, short modular functions, minimal comments.
- Match Next.js/React best practices: handle loading, success, and error states cleanly.
- Premium UI aesthetics: curated color gradients, drop zone shimmer effects, responsive layout, glassmorphic touches.

## Testing Strategy
- **Manual Verification:** Build the app, trigger the OCR dropzone with a sample blurred receipt image, confirm the loading indicator shows, and confirm the text extracts successfully into the clean result field.
- **Error Handling:** Verify graceful errors are returned if the sidecar is offline or a non-image file is provided.

## Success Criteria
1. The new Tauri command `process_ocr_local` compiles and works correctly.
2. The user can open the document editor, select the "Digitalizar" (OCR) tab in the AI Assistant, select or drag/drop an image, and see it processed.
3. The extracted text is displayed in a clean text area with copy-to-clipboard and copy-to-free-text actions.
