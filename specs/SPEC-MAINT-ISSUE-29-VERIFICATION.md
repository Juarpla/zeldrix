# Spec: Issue 29 Verification Cleanup

## Objective
Remove verification blockers found while validating Issue 29 so the backend can be formatted and tested cleanly. The cleanup must stay limited to existing blockers: a formatter-stopping trailing whitespace issue and the vector search performance path exercised by the current test suite.

## Tech Stack
- Rust + Tauri v2
- Standard library collections and sorting utilities

## Commands
- Rust format: `cd src-tauri && cargo fmt`
- Rust tests: `cd src-tauri && cargo test`
- Build: `npm run build`

## Project Structure
- `src-tauri/src/retrieval_engine/prompt_packer.rs` removes the formatting blocker.
- `src-tauri/src/vector_db/mod.rs` keeps the public search API while reducing unnecessary work for top-k search.

## Code Style
Use existing module-local helpers and avoid new dependencies.

```rust
scores.select_nth_unstable_by(limit, compare_scores);
```

## Testing Strategy
- Run `cd src-tauri && cargo fmt`.
- Run `cd src-tauri && cargo test`.
- Re-run `npm run build` after backend changes.

## Boundaries
- Always: preserve public API and search result ordering.
- Ask first: changing test thresholds or adding crates.
- Never: relax performance tests to hide a slow implementation.

## Success Criteria
- `cargo fmt` completes.
- `cargo test` completes.
- `npm run build` completes.
- No unrelated formatting churn remains staged in the working tree.
