# Spec: Issue 20 - Graphical Interface for Corporate Knowledge Base Explorer

## Objective
Design and implement the file administration dashboard for the Corporate Knowledge Base Explorer in the Zeldrix application. The interface will provide a visually stunning, responsive, and state-of-the-art dashboard that enables administrators to manage virtual indexed folders, monitor vector processing queues, see disk space usage, view total successfully processed documents, and react to corrupt file warnings.

### User Story
As a Zeldrix Knowledge Base Administrator, I want a premium graphical user interface to:
1. See high-level status of the database: total successfully processed documents, disk space usage, and active warnings for corrupt or incompatible files.
2. View and navigate through virtual folders containing indexed files.
3. Manage files inside an interactive, search-enabled, and filterable table.
4. Witness real-time vector processing states: a green badge when a file is successfully indexed, and an animated spinner/progress loader when it is in the vectorization queue.
5. Identify corrupt files with distinctive warning indicators and expand details to understand why indexing failed (e.g. OCR required, unsupported format, etc.).
6. Simulate adding new documents to the knowledge base to inspect vector processing states and corruption alarms dynamically.

## Tech Stack
- Frontend: Next.js 16+ App Router, React 19, TypeScript
- Styling: Tailwind CSS 4.0 (configured in PostCSS), Framer Motion 12+ (for fluid animations and premium micro-interactions)
- UI Primitives: Lucide React (icons), custom Tailwind components (for glassmorphic dark/light aesthetics and seamless tables)
- IPC Boundary: Tauri v2 `@tauri-apps/api/core` (optional hooks for actual vector DB querying, falling back gracefully to mock simulator for full-fidelity demonstration)

## Commands
- Run Frontend Dev Server: `npm run dev`
- Build Frontend: `npm run build`
- Run Linter: `npm run lint`

## Project Structure
We will create and modify the following files:
- `src/app/knowledge-base/page.tsx` [NEW]: The Next.js App Router entry page for the Knowledge Base Explorer dashboard.
- `src/components/KnowledgeBase/KnowledgeBaseDashboard.tsx` [NEW]: The main dashboard shell rendering folder grids, KPI metric cards, alert banners, and the interactive table.
- `src/components/KnowledgeBase/FolderCard.tsx` [NEW]: Premium folder visualization cards with hover transformations and file counters.
- `src/components/KnowledgeBase/MetricCard.tsx` [NEW]: Visual KPI metrics for successfully processed docs, disk space (radial progress or linear bar), and corruption alerts.
- `src/components/KnowledgeBase/FileTable.tsx` [NEW]: Highly interactive file list displaying format icons, name, folder, file size, status badges (green indexed vs. animated vectorizer), and row actions.
- `src/components/KnowledgeBase/ImportSimulator.tsx` [NEW]: A tool built into the dashboard allowing administrators to mock-import files (clean or corrupt) to test states dynamically.
- `src/components/KnowledgeBase/types.ts` [NEW]: Domain TypeScript models for Knowledge Base folders, files, and statistics.

## Code Style
According to the `AGENTS.md` and project coding standards:
- Strictly English for code, variable identifiers, function names, and comments.
- Avoid redundant or cluttering comments; prefer self-documenting, descriptive identifiers.
- Split large components into focused helper functions or modular sub-components.
- Leverage React 19 state patterns and clean hooks.
- Apply high-end UI details: HSL colors, smooth transitions (`transition-all duration-300`), glassmorphic panels (`backdrop-blur-md bg-white/70 dark:bg-gray-900/70 border border-white/20`), and Framer Motion micro-animations.

### Example Code Snippet:
```typescript
export interface IndexedFile {
  id: string;
  name: string;
  sizeBytes: number;
  format: 'pdf' | 'docx' | 'xlsx' | 'txt';
  virtualFolder: string;
  indexingStatus: 'completed' | 'processing' | 'corrupt';
  errorMessage?: string;
  addedAt: string;
}
```

## Testing Strategy
- **Visual & Layout Check:** Open the knowledge base route `/knowledge-base` in the application browser. Verify that the layout adjusts flawlessly to multiple viewport sizes.
- **Interactive State Validation:**
  - Verify that adding a simulated document sets its status to `processing` with a custom animated loading icon.
  - Verify that it transitions to `completed` with a green badge, or `corrupt` with a warning alert if it is simulated as invalid.
  - Verify that deleting a file updates the KPI counts (successful files count) and disk space occupied.
  - Verify that search query text filters table records instantly.
  - Verify that clicking folder cards filters table records by that folder.

## Boundaries
- **Always:** Use custom modern loaders for processing states rather than browser defaults.
- **Always:** Pre-populate the view with realistic, engaging default mock corporate records to avoid a blank, unimpressive slate.
- **Never:** Break layout alignment on smaller desktop screens; use responsive grid and overflow controls where necessary.

## Success Criteria
1. Beautiful page rendered at `/knowledge-base` with premium glassmorphism, responsive grids, cohesive color choices, and smooth micro-interactions.
2. KPI metric cards presenting:
   - Successfully processed document counts.
   - Distinctive disk space progress indicator (e.g., modern visual chart or sleek gauge).
   - An alert panel detailing corrupt files (OCR required, unsupported format, etc.) if any are present.
3. Virtual folder visualization allowing users to filter files by folder categories.
4. Interactive files table with robust columns, live search, and action menus.
5. In compliance with the Acceptance Criteria:
   - Green badge (e.g. `Indexed`) on successfully vectorized documents.
   - Animated spinner/shimmering processing icon (e.g. `Vectorizing...`) on queued/active files.
6. A built-in mock import simulator to easily demonstrate these states, processing animations, and error handling.
7. Verification that the code builds and runs cleanly without Next.js compile errors or TypeScript warnings.

## Open Questions
- Should this dashboard integrate directly with the existing `src-tauri` vector database commands?
  *To ensure the user gets a fully functional, flawless demo immediately without requiring sidecars to be compiled or databases populated manually, we will populate the page with realistic initial state, connect to Tauri invoke handlers if available, and provide an elegant client-side simulation system so the dashboard is robust, fully interactive, and demonstratable under any setup.*
