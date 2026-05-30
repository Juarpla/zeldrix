# Spec: Issue 39 Floating Stream View

## Objective
Extend the spotlight quick-reply input so a short query can show an immediate streaming answer below the floating input. The compact command bar should grow downward with a smooth animated height transition while tokens begin appearing on screen.

## Tech Stack
- Next.js 16 app route in `src/app/spotlight/page.tsx`
- React 19 client component state and effects
- Framer Motion 12 for dynamic height and content transitions
- Tailwind CSS 4 utility classes

## Commands
- Build: `npm run build`
- Dev: `npm run dev`

## Project Structure
- `src/app/spotlight/page.tsx` contains the floating quick-reply input and streaming result view.
- `specs/` stores feature specifications.

## Code Style
Keep all UI behavior local to the spotlight route until a backend streaming contract exists. Use explicit English names for UI state and event handlers.

```tsx
const [streamedAnswer, setStreamedAnswer] = useState("");

function handleQuickReplySubmit(event: React.FormEvent<HTMLFormElement>) {
  event.preventDefault();
}
```

## Testing Strategy
- Run `npm run build` to verify TypeScript and production compilation.
- Manual visual check: open `/spotlight`, type a query, press Enter, and confirm the surface expands downward while answer text streams in.

## Boundaries
- Always: preserve the floating transparent spotlight route and Escape-to-hide behavior.
- Always: animate the expanded answer area without introducing native page scrollbars.
- Always: clear stale timers when a new query starts or the component unmounts.
- Ask first: adding new dependencies, changing Tauri window configuration, or wiring real model streaming APIs.
- Never: add sidebars, full chat history, unrelated navigation, or backend assumptions.

## Success Criteria
- Pressing Enter with a non-empty query expands the floating surface below the input.
- The expansion transition is smooth and runs as the first answer tokens appear.
- The input remains focused and usable after submission.
- A new submitted query replaces the previous response stream.
- Empty submissions do not open the response area.
- `npm run build` completes successfully.

## Open Questions
- Real model streaming is out of scope for this issue because no backend streaming command or API contract is specified.
