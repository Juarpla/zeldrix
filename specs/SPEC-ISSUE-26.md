# Spec: Issue 26 — Repetitive Tasks Automation & "One-Click" Cards Hub

## Objective

Build a highly visual, premium, and interactive "One-Click" Automation Hub for non-technical office workers. The system will present a beautiful grid of factory-preconfigured, heavy workflow automations (like "Resumir Correo", "Estructurar Minuta", "Extraer Tabla de PDF") with premium hover micro-animations and an intuitive workflow execution state. Clicking a card will trigger and display the progression of a heavy office automation workflow, displaying modern step-by-step progress, interactive feedback, and immediate actionable results.

**User story:** As an office employee, I want to trigger complex, repetitive tasks (like parsing PDFs, structuring meeting minutes, or summarizing long emails) with a single click, see the live execution status in a premium UI, and get my result immediately without needing technical knowledge.

---

## Tech Stack

- **Frontend:** React + TypeScript (Next.js App Router, `'use client'` for interactive elements)
- **Styling:** Tailwind CSS (v4) with vanilla CSS animations
- **Animation:** `framer-motion` for premium micro-animations, scale transforms, and fade transitions
- **Icons:** Inline responsive SVG icons

---

## Commands

```
Dev:   npm run dev
Build: npm run build
Lint:  npm run lint
```

---

## Project Structure

### New files
```
src/app/automations/page.tsx                     → Automation Hub page route
src/components/Automations/AutomationHub.tsx     → Main component coordinating the grid and workflows
src/components/Automations/AutomationCard.tsx    → Premium card with framer-motion micro-animations
src/components/Automations/WorkflowExecution.tsx → Progress panel & details for active workflow execution
src/lib/automations-data.ts                     → Pre-configured factory shortcuts & mock data
```

---

## Data Model

### `AutomationShortcut` Type
```typescript
export interface AutomationShortcut {
  id: string;
  title: string;
  description: string;
  category: 'email' | 'documents' | 'data-extraction' | 'meetings';
  difficulty: 'light' | 'medium' | 'heavy';
  estimatedSeconds: number;
  icon: string; // Key to SVG lookup
  steps: {
    label: string;
    description: string;
  }[];
  defaultInputs?: {
    label: string;
    placeholder: string;
    type: 'text' | 'textarea' | 'file';
  }[];
}
```

---

## Feature Behaviour

### 1. Grid of Factory-Installed Shortcuts
The page displays a grid of cards including preloaded automations:
- **Resumir Correo (Summarize Email):** Shortens long newsletters or emails into bullet points.
- **Estructurar Minuta (Structure Minutes):** Takes messy transcripts/notes and outputs a beautifully formatted meeting minute document.
- **Extraer Tabla de PDF (Extract Table from PDF):** Parses tabular structures out of a uploaded document.
- **Redactar Respuesta (Draft Reply):** Automatically writes standard templates based on brief bullet points.

### 2. Premium Design & Micro-animations
- Beautiful gradients (e.g. indigo, emerald, purple tints).
- Hover animations using `framer-motion` (smooth scaling `scale: 1.03`, glowing box-shadow, icon translation).
- Badges displaying Category and Difficulty levels ("Heavy", "Medium", "Light").

### 3. "One-Click" Workflow Trigger & Execution
- Clicking on a card or its primary action button opens the modern workflow simulator directly.
- Standard "one-click" action is supported by presenting a quick-start interface. If input is required, a premium sidebar/modal lets the user paste their content or upload a mock file, then click "Ejecutar Tarea" (Execute Task).
- A step-by-step progress tracker illustrates:
  1. Ingestion/parsing of input data (with simulated delay).
  2. Core AI processing/information extraction (with active progress spinner and pulsing text).
  3. Formatting and final output creation.
- Once finished, the output is displayed inside a beautiful read-only typography block with one-click actions:
  - **Copiar al portapapeles** (Copy to Clipboard)
  - **Abrir en Editor** (Open in Document Editor) - redirects to `/editor` passing the text as query parameters or state!

---

## Code Style

- Strict English for code identifiers, comments, and structure.
- Spanish/English localized UI for end users (user-facing labels in Spanish to match the request terms).
- Modular, self-documenting code. Split the Main Hub, Card Grid, and Execution details.

---

## Testing Strategy

- **Verification:** Run Next.js builds to guarantee zero TS compilation issues.
- **Manual Checklist:**
  1. Navigate to `/automations` and verify the premium layout and factory cards.
  2. Hover over a card → verify scale and glow.
  3. Click "Resumir Correo" → triggers the workflow screen.
  4. Paste standard text or hit "Ejecutar" with defaults.
  5. Check progress bar states updating sequentially.
  6. Verify final output display and actions ("Copiar", "Abrir en Editor").

---

## Boundaries

- **Always:** Use custom-tailored theme colors matching the existing Zeldrix dark/light mode standards.
- **Never:** Load unoptimized heavy libraries for simple progress bars; rely on native Framer Motion or pure CSS.

---

## Success Criteria

1. Non-technical users can trigger any factory-preconfigured workflow with a single click.
2. The UI features responsive Tailwind layout grids supporting fluid scale transitions on hover.
3. Live workflow execution displays clear status changes with individual steps turning completed green.
4. Output copy-to-clipboard is fully functional and provides a visual success toast/alert.
5. "Open in Editor" button successfully transitions to `/editor` preloaded with the generated text.
6. Zero TypeScript compile errors.
