# Spec: Issue 37 - Resilient Global Shortcut Listener (Atajo de Teclado Global Resiliente)

## Objective
Implement a native OS global keyboard shortcut (e.g., `Option+Space` on macOS and `Alt+Space` on Windows/Linux) to instantly toggle the visibility of the `spotlight` floating window.
- When pressed:
  - If the spotlight window is hidden: show it, bring it to front, and focus it immediately (within 100ms), which automatically focuses the text area on the frontend.
  - If the spotlight window is visible: hide it cleanly, which returns focus to the previously active application.
- The registration must be resilient and registered during backend startup using the official Tauri v2 global-shortcut plugin (`tauri-plugin-global-shortcut`).

## Tech Stack
- Desktop bridge: Tauri v2 + Rust backend.
- Plugin: `tauri-plugin-global-shortcut` (v2).
- Frontend: Next.js App Router + React 19 + TypeScript.

## Commands
- Run desktop/dev app: `npm run dev`
- Build/Check Rust: `cd src-tauri && cargo check`
- Rust linting: `cd src-tauri && cargo clippy`

## Project Structure
- `src-tauri/Cargo.toml` -> Add `tauri-plugin-global-shortcut` dependency.
- `src-tauri/src/lib.rs` -> Register the global-shortcut plugin, bind the shortcut key combinations, and handle the toggling logic of the spotlight window.
- `src-tauri/capabilities/default.json` -> Grant permissions for `global-shortcut:default` (though we handle the listener on the backend/Rust side, it is good practice to add permission).

## Code Style
Idiomatic Rust style, clean Tauri v2 patterns.
```rust
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

// Register shortcut in setup
let shortcut = if cfg!(target_os = "macos") {
    Shortcut::new(Some(Modifiers::ALT), Code::Space) // Option+Space
} else {
    Shortcut::new(Some(Modifiers::ALT), Code::Space) // Alt+Space
};

app.global_shortcut().on_shortcut(shortcut, |app, shortcut, event| {
    if event.state() == ShortcutState::Pressed {
        // Toggle spotlight logic
    }
})?;
```

## Testing Strategy
- Compile and build verification: Ensure the project builds successfully on macOS/desktop.
- Manual Verification:
  1. Start the app.
  2. Press `Option+Space` (on macOS) or `Alt+Space` (on Windows/Linux) from any application.
  3. Verify that the spotlight window appears instantly (<100ms) and focus is directed to the textarea input.
  4. Press the shortcut combination again.
  5. Verify that the spotlight window is hidden cleanly, and the focus is returned to the previously active application.

## Boundaries
- Always: Register the OS-appropriate shortcut safely on startup; catch and handle registration errors so the app does not crash if the shortcut is already in use by another app.
- Ask first: Changing the default hotkey to a non-standard combination or adding other external crates.
- Never: Register a global shortcut that overrides critical OS-level shortcuts (like Cmd+Space / Win+Space which are usually bound to native Spotlight/Start search).

## Success Criteria
1. The global shortcut (`Option+Space` on macOS, `Alt+Space` on Windows/Linux) is successfully registered on startup via `tauri-plugin-global-shortcut`.
2. Pressing the shortcut toggles the visibility of the `spotlight` window instantly (<100ms).
3. Showing the window automatically gains focus and brings it to the front so the user can immediately type into the text area.
4. Hiding the window returns the keyboard focus to the previous active application on the system.
5. Graceful fallback/logging if the global shortcut registration fails (e.g. if already bound by another process).

## Open Questions
- None. The shortcut behavior is fully aligned with standard floating bars.
