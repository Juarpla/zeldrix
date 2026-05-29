# Spec: Issue 36 - Spotlight Floating Window Architecture (Raycast Core)

## Objective
Configure Tauri to register an independent hidden secondary window (`spotlight`) that has no native window borders, has a transparent/translucent background, and is mathematically centered on the user's screen. The window should be loaded in memory in a hidden state on app startup. Even if the main window is closed, the spotlight window remains in memory waiting to be shown. We also want to provide a command bar UI in this window to quickly perform AI tasks.

## Tech Stack
- Frontend: Next.js App Router, React 19, TypeScript, Tailwind CSS.
- Desktop bridge: Tauri v2 IPC through `@tauri-apps/api/core`.
- Backend: Rust 2021, `tauri::WebviewWindowBuilder`.

## Commands
- Build frontend: `npm run build`
- Run desktop/dev app: `npm run dev`
- Rust linting: `cd src-tauri && cargo clippy`

## Project Structure
- `src/app/spotlight/page.tsx` renders the minimalist command bar.
- `src-tauri/src/lib.rs` initializes the spotlight window, sets transparent/undecorated properties, configures window event intercepts, and handles showing/hiding commands.
- `src-tauri/capabilities/default.json` grants permissions to the spotlight window.

## Code Style
Follow standard patterns with clean Rust and TypeScript definitions.

```rust
// Register spotlight window in lib.rs setup
let spotlight = tauri::WebviewWindowBuilder::new(
    app,
    "spotlight",
    tauri::WebviewUrl::App("/spotlight".into())
)
.title("Spotlight")
.resizable(false)
.decorations(false)
.transparent(true)
.visible(false)
.inner_size(680.0, 380.0)
.center()
.build()
.expect("failed to build spotlight window");
```

## Testing Strategy
- Compile and build verification: Ensure the project builds successfully on desktop.
- Manual Verification:
  1. Start the app, verify the main window opens.
  2. Verify that a hidden "spotlight" window is built in the background.
  3. Close the main window, verify the app remains running in memory.
  4. Trigger showing the spotlight window via a shortcut or system tray/ipc event and verify it is centered, has no window borders, has transparent corners/background, and displays the minimalist command bar.

## Boundaries
- Always: Build the spotlight window hidden during app setup; mathematically center it using `.center()`; configure window close intercepts so closing the main window hides it instead of terminating the app if desired.
- Ask first: Registering a global keyboard shortcut with OS-level hooks, or installing new external OS-level key hook crates.
- Never: Exit the process on main window close unless explicitly requested; expose default OS decorations on the spotlight window.

## Success Criteria
1. The spotlight window is created dynamically during Tauri startup with label `spotlight`, pointing to `/spotlight`.
2. The spotlight window has native OS decorations disabled (`decorations: false`), resizable disabled (`resizable: false`), is translucent/transparent, and is mathematically centered on the screen.
3. The spotlight window is loaded hidden (`visible: false`) waiting to be called.
4. Closing the main window hides the main window instead of terminating the app, keeping the background process and the spotlight window alive.
5. Minimalist frontend command bar UI is built at `/spotlight` featuring an AI prompt search input.
6. A Tauri command `toggle_spotlight` is implemented to show/hide the spotlight window.

## Open Questions
- Should we register a global hotkey (like Alt+Space)? We will register a global hotkey via Tauri's global shortcut handling, or provide the API commands for frontend toggling.
