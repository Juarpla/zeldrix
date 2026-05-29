# Spec: Issue 33 - Batch Progress Monitor UI

## Objective
Build a React batch progress monitor for massive automation queues so users can see the global processing state at a glance. The UI must show how many files are complete out of the total, which file is currently active, how many remain pending, and an estimated time remaining.

Assumptions:
- This issue is frontend-only and consumes queue-like state already produced by the batch processing feature from Issue 32.
- The first integration appears in the automation execution screen for heavy batch-style work.
- Failed items count as processed for completion math because they no longer block the queue.
- The ETA can be derived from average completed item duration when timestamps exist, and from automation progress simulation when the UI is showing mock execution.

## Tech Stack
- Frontend: React 19, TypeScript, Next.js App Router
- Styling: Tailwind CSS utility classes
- Animation: Framer Motion, already available in the project
- Native bridge: Tauri event and command APIs where live queue data is available

## Commands
- Build: `npm run build`
- Dev: `npm run dev`

## Project Structure
- `src/components/Automations/BatchQueueMonitor.tsx` owns the visual queue monitor, ETA math, status labels, and per-file animations.
- `src/components/Automations/WorkflowExecution.tsx` renders the monitor during heavy batch automation execution.
- `specs/SPEC-ISSUE-33-BATCH-PROGRESS-MONITOR.md` stores this contract.

## Code Style
Keep UI state derivation in small helper functions and keep component props serializable. Prefer descriptive names and Tailwind classes already used by the automation screen.

```tsx
const completedCount = items.filter((item) => item.status === "completed" || item.status === "failed").length;
const pendingCount = items.filter((item) => item.status === "pending").length;
const activeItem = items.find((item) => item.status === "processing");
```

## Testing Strategy
- Run `npm run build` to verify TypeScript and production compilation.
- Manually verify the automation execution view shows a numeric progress indicator and individual file loading animations.
- Check responsive layout at narrow widths through the build output and Tailwind classes.

## Boundaries
- Always: show `processed / total`, pending count, active file name, and ETA when there are items.
- Always: animate the active file row and show a distinct visual state for completed, failed, processing, and pending items.
- Ask first: changing backend queue contracts, adding dependencies, or introducing persistent queue history.
- Never: block execution because ETA cannot be calculated, hide failed rows, or count pending items as processed.

## Success Criteria
- Users see a clear numeric indicator such as `14 / 30 Procesados - 45s restantes`.
- Users can identify the active file currently being processed.
- Users can see how many files remain pending.
- Each visible file row has an individual status indicator, with an animated loading state for the active file.
- The monitor works with live queue items and with local simulated queue items used by the automation execution preview.
- `npm run build` completes successfully.

## Open Questions
- Should a later issue add cancellation/retry controls directly to this panel?
