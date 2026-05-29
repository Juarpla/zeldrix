# Spec: Issue 38 Ultra-Minimal Quick Reply Input

## Objective
Build the floating quick-reply frontend as a single elevated input surface for desktop use. The user should see only a styled text input and a discreet active AI model selector, with a premium rounded shape, deep physical shadow, and clean behavior in light and dark system themes.

## Tech Stack
- Next.js 16 app route in `src/app/spotlight/page.tsx`
- React 19 client component
- Tailwind CSS 4 utility classes
- Existing Tauri window behavior for Escape-to-hide

## Commands
- Build: `npm run build`
- Dev: `npm run dev`

## Project Structure
- `src/app/spotlight/page.tsx` contains the floating quick-reply interface.
- `src/app/globals.css` contains app-wide theme and base element styles.
- `specs/` stores feature specifications.

## Code Style
Use explicit English names and keep UI state local when the behavior is isolated to the component.

```tsx
const [activeModel, setActiveModel] = useState("Zeldrix Local");

<select
  aria-label="Active AI model"
  value={activeModel}
  onChange={(event) => setActiveModel(event.target.value)}
/>
```

## Testing Strategy
- Run `npm run build` to verify TypeScript and production compilation.
- Manual visual check should confirm the route has no native scrollbars, displays a single floating input bar, and responds to system light/dark preferences.

## Boundaries
- Always: keep the interface limited to the input and active model selector.
- Always: preserve Escape-to-hide behavior for the Tauri spotlight window.
- Ask first: adding dependencies, changing Tauri window configuration, or introducing backend model state.
- Never: add panels, native scroll containers, result feeds, sidebars, or unrelated navigation.

## Success Criteria
- The spotlight interface renders as one clean floating rounded input bar.
- A discreet selector indicates and allows changing the currently active AI model.
- The surface uses deep layered shadows and premium gradients for physical elevation.
- The route avoids native scrollbars at normal desktop window sizes.
- Light and dark system themes both have polished corporate styling.
- `npm run build` completes successfully.

## Open Questions
- The selector is local UI state for this issue because no backend active-model contract is specified.
