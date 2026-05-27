# AGENTS.md — Agent Instructions for This Project

## Mandatory: Spec-Driven Development for ALL Code Modifications

**Before making ANY modification to code in this project, you MUST load and follow the spec-driven-development skill.**

This is not optional. It applies to:
- New features
- Bug fixes
- Refactoring
- Configuration changes
- Dependency updates
- Documentation changes that affect code behavior
- Any change that touches more than a single line

The skill is located at: `.agents/skills/spec-driven-development/SKILL.md`

Load it with: `skill: spec-driven-development`

### What This Means in Practice

1. **No code without a spec.** Do not write implementation code until a spec exists and has been reviewed.
2. **Spec first, always.** Even for small changes, write the acceptance criteria before touching code.
3. **Verify against the spec.** Run tests or manual checks to confirm every spec item passes.
4. **Update the spec when scope changes.** The spec is a living document — if requirements shift, update the spec first, then the code.
5. **Store specs in `specs/`.** Every spec file must be saved inside the `specs/` folder at the project root, using a descriptive filename (e.g., `specs/SPEC-ISSUE-09.md`, `specs/SPEC-FEATURE-NAME.md`). Never leave spec files scattered in the root directory.

### Exceptions

The only exceptions where you may skip the full spec workflow:
- Single-character typos in comments or strings
- Formatting/whitespace-only changes
- Changes explicitly requested by the human with "skip the spec, just do it"

Even in these cases, state what you're doing and why you're skipping the spec.

## Mandatory: Next.js Best Practices for UI/UX Work

**When a prompt relates to UI or UX — components, pages, styling, routing, data fetching for display, error boundaries, loading states, metadata, images, fonts, or any frontend-facing code — you MUST load and follow the next-best-practices skill.**

The skill is located at: `.agents/skills/next-best-practices/SKILL.md`

Load it with: `skill: next-best-practices`

This covers:
- File conventions and routing patterns
- React Server Component boundaries
- Data fetching patterns (Server Components, Server Actions, Route Handlers)
- Error handling and Suspense boundaries
- Image and font optimization
- Metadata and OG images
- Hydration errors, bundling, and async patterns

## Mandatory: Tauri v2 for Desktop/Mobile Work

**When a prompt relates to Tauri — `src-tauri/` configuration, Rust commands (`#[tauri::command]`), IPC patterns (`invoke`, `emit`, channels), capabilities/permissions, window management, bundling, or any Tauri-specific code — you MUST load and follow the tauri-v2 skill.**

The skill is located at: `.agents/skills/tauri-v2/SKILL.md`

Load it with: `skill: tauri-v2`

This covers:
- `tauri.conf.json` configuration and build setup
- Rust command registration and invocation from frontend
- IPC patterns: invoke, emit, channels
- Capabilities and permissions (`capabilities/default.json`)
- State management with `Mutex<T>` and `State<T>`
- Error handling across the IPC boundary with `Result<T, E>` and serde
- Window access (`WebviewWindow`) and event emission
- Mobile compatibility (`lib.rs` structure, `mobile_entry_point`)
- Plugin installation and permission setup
- Troubleshooting: white screens, command not found, permission denied, mobile build failures

## Mandatory: Rust Best Practices for Rust Code

**When writing, reviewing, or modifying any Rust code — new functions, refactoring, error handling, ownership patterns, performance optimization, or tests — you MUST load and follow the rust-best-practices skill.**

The skill is located at: `.agents/skills/rust-best-practices/SKILL.md`

Load it with: `skill: rust-best-practices`

This covers:
- Borrowing and ownership patterns (`&T` over `.clone()`, `&str` over `String`)
- Error handling (`Result<T, E>`, `thiserror` vs `anyhow`, no `unwrap()` outside tests)
- Performance (benchmark with `--release`, avoid cloning in loops, iterator patterns)
- Linting (`cargo clippy`, key lints to watch)
- Testing (descriptive names, one assertion per test, doc tests, snapshot testing)
- Generics and dispatch (static vs dynamic)
- Type state pattern for compile-time state safety
- Documentation (`//` for why, `///` for what/how, `#![deny(missing_docs)]`)

## Mandatory: llama.cpp for Local LLM Inference

**When a prompt relates to local LLM inference, model deployment, quantization, or running models on non-NVIDIA hardware (CPU, Apple Silicon, AMD/Intel GPUs) — you MUST load and follow the llama-cpp skill.**

The skill is located at: `.agents/skills/llama-cpp/SKILL.md`

Load it with: `skill: llama-cpp`

This covers:
- Building and installing llama.cpp (Metal, CUDA, ROCm)
- GGUF model download and conversion
- Running inference (CLI and server mode)
- Quantization format selection (Q2_K through Q8_0)
- Hardware acceleration (Apple Silicon, NVIDIA, AMD)
- OpenAI-compatible server setup
- Context size configuration and batch processing
- Performance optimization and benchmarks

## Mandatory: rusqlite Guide for SQLite Database Work

**When a prompt relates to SQLite databases using rusqlite — connection management, CRUD operations, transactions, full-text search (FTS5), migrations, or any database-specific code — you MUST load and follow the rusqlite-guide skill.**

The skill is located at: `.agents/skills/rusqlite-guide/SKILL.md`

Load it with: `skill: rusqlite-guide`

This covers:
- Connection initialization and configuration
- CRUD operations and prepared statements
- Transaction handling and savepoints
- FTS5 full-text search integration
- Database migrations
- Error handling with `Result<T, E>`

## Project Context

- **Stack:** Tauri + React + TypeScript + Vite
- **Build:** `npm run build`
- **Dev:** `npm run dev`
- **Test:** (define as project evolves)

## Code Principles

1. **Think Before Coding** — State assumptions explicitly. Present multiple interpretations when ambiguous. Push back when warranted. Stop when confused.
2. **Simplicity First** — Minimum code that solves the problem. No speculative features. No abstractions for single-use code.
3. **Surgical Changes** — Touch only what you must. Match existing style. Remove orphans your changes create.
4. **Goal-Driven Execution** — Define success criteria. Loop until verified. Transform tasks into verifiable goals.
5. **Minimize Comments & Prefer Self-Documenting Code** — Avoid adding comments during code implementations unless strictly necessary. Instead, write highly descriptive and comprehensive names for functions, constants, and variables.
6. **Modular & Readable Functions** — If a function's name cannot be made comprehensive or its logic is complex, split it into as many small, focused helper functions as possible so that any reviewer or reader can easily follow and understand.
7. **English Only** — All source code, including function names, variable and constant identifiers, documentation, logs, and any strictly necessary comments, must be written in English only.

