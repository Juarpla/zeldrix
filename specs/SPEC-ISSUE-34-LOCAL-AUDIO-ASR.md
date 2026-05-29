# Spec: Issue 34 - Local Audio ASR Pipeline

## Objective
Add a backend-only local audio transcription pipeline for executive dictations and meeting voice notes. The Rust backend must accept standard local audio files, route them through the existing local llama.cpp multimodal server using `gemma-4-E2B-it`, and return faithful Spanish text without any cloud service calls.

## Tech Stack
- Backend: Rust, Tauri v2 commands
- Model Interface: local llama.cpp OpenAI-compatible Chat Completions API
- Audio Transport: base64 `data:` URLs inside multimodal `audio_url` message parts
- Existing Model: `gemma-4-E2B-it`

## Commands
- Rust tests: `cd src-tauri && cargo test`
- App build: `npm run build`
- Dev server: `npm run dev`

## Project Structure
- `src-tauri/src/multimodal/mod.rs`: add local ASR command and helpers for audio validation, encoding, prompt construction, and response extraction.
- `src-tauri/src/lib.rs`: register the ASR Tauri command in `tauri::generate_handler!`.
- `specs/SPEC-ISSUE-34-LOCAL-AUDIO-ASR.md`: capture scope, acceptance criteria, and verification.

## Code Style
Use small, focused Rust helpers with English identifiers and user-facing errors that explain what failed. Keep async commands using owned arguments, release shared-state locks before awaiting network I/O, and avoid `unwrap()` outside tests.

```rust
#[tauri::command]
pub async fn transcribe_audio_local(
    sidecar_state: State<'_, SidecarState>,
    audio_path: String,
) -> Result<String, String> {
    let port = resolve_running_sidecar_port(&sidecar_state)?;
    transcribe_audio_file(port, audio_path).await
}
```

## Testing Strategy
- Unit test accepted audio MIME detection for WAV and other standard extensions.
- Unit test rejection of unsupported media extensions.
- Unit test extraction of text from llama.cpp chat completion JSON.
- Compile-check the Tauri command registration through `cd src-tauri && cargo test`.
- Manual verification with a one-minute Spanish WAV requires a running local `llama-server` with the audio-capable model/projector assets.

## Boundaries
- Always: keep processing local to `127.0.0.1`, validate file existence and extension before inference, return only the transcript text.
- Ask first: adding new Rust dependencies, changing sidecar startup flags beyond what ASR needs, adding frontend UI.
- Never: call cloud transcription APIs, upload audio off-device, persist transcript/audio without an explicit request.

## Success Criteria
1. A new Tauri command accepts a local WAV file path and returns a Spanish transcript string from the local llama.cpp server.
2. The command rejects missing or unsupported files before sending inference requests.
3. The backend calls only the local `http://127.0.0.1:{port}/v1/chat/completions` endpoint.
4. `cd src-tauri && cargo test` passes.

## Open Questions
- Frontend capture/upload UI is out of scope for this issue unless requested separately.
- Real transcription quality depends on the local model and llama.cpp build supporting audio input.
