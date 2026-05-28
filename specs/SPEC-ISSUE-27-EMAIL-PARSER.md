# Spec: Issue 27 — Email Thread Parser & Noise Cleaner

## Objective

Create an advanced string processing module in Rust to clean and optimize complex email threads (such as Outlook, Gmail, etc.) before feeding the text to a Local LLM. The module will systematically strip away metadata headers, redundant signatures, corporate disclaimers, and trash line breaks. This reduces token counts by up to 80% while retaining the pure semantic conversation history of up to 10+ nested replies.

**User story:** As an AI assistant backend, I need to clean noisy email thread blocks (From/To headers, corporate boilerplate, disclaimers, signatures) so that the LLM consumes fewer tokens and is not confused by redundant signatures or corporate disclaimers.

---

## Tech Stack

- **Backend:** Rust 1.70+
- **Regex Crate:** `regex` (highly optimized and compiled regex patterns using `lazy_static` or `once_cell` for high performance).

---

## Commands

```bash
# Run backend tests
cargo test --manifest-path src-tauri/Cargo.toml email_parser
```

---

## Project Structure

### New files
```
src-tauri/src/email_parser/mod.rs    → Main parsing & cleaning logic
```

### Modified files
```
src-tauri/src/lib.rs                  → Module registration & Tauri command exposure
```

---

## Code Style

- Strict English for code identifiers, comments, and structure.
- Self-documenting code with descriptive names, as specified in `AGENTS.md`.
- Comprehensive inline tests for multi-layered email threads.

---

## Testing Strategy

- **Unit tests:**
  - `test_remove_headers`: Verification of standard and Spanish headers (From, De, To, Para, Cc, Subject, Asunto, Date, Fecha).
  - `test_remove_signatures`: Stripping of common Spanish and English signatures (e.g., "Saludos cordiales", "Best regards").
  - `test_remove_disclaimers`: Stripping of common corporate boilerplates and privacy notifications.
  - `test_clean_email_thread`: Comprehensive integration test simulating a 10-reply Outlook nested email thread.
  - `test_optimize_line_breaks`: Collapsing consecutive newlines and removing garbage characters.

---

## Boundaries

- **Always:** Compile regular expressions using `lazy_static` or `once_cell` (via `lazy_static` which is already in `Cargo.toml`) to avoid re-compilation on every clean call.
- **Never:** Lose the actual conversation body text or edit the message contents themselves.
- **Always:** Handle both Spanish and English email header structures cleanly.

---

## Success Criteria

1. A raw, messy Outlook email thread of 10 replies is successfully cleaned, retaining only the pure messaging contents (e.g. chronological dialog).
2. Redundant corporate disclaimers (English & Spanish) are fully removed.
3. Signatures containing phone numbers, company names, or email addresses are successfully removed.
4. Token footprint of the output is significantly optimized (collapsing redundant blank lines).
5. Zero compilation errors and all tests pass.
6. A Tauri command `clean_email_thread(text: String) -> Result<String, String>` is exposed for frontend consumption.
