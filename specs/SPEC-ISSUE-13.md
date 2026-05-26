# SPEC-ISSUE-13: Efecto Visual Shimmer (Sombreado Dinámico) en Bloques de Inferencia Activa

## Objective
Crear un componente de UI y extensión de editor que aplique una animación de carga tipo shimmer (destello degradado) sobre la sección exacta del documento donde la IA está inyectando o reescribiendo contenido. Esta animación debe oscilar suavemente en tonos grises/azules satinados (tanto en modo claro como oscuro) sobre la selección del usuario mientras la IA procesa y renderiza el resultado token por token, e indicar explícitamente qué parte del archivo está en proceso de creación automatizada.

El flujo visual será:
1. **Selección**: El usuario selecciona un párrafo o texto.
2. **Acción**: Al presionar un botón del menú IA (por ejemplo, "Mejorar redacción", que renombraremos/asociaremos con "Corregir estilo"), el fondo del texto seleccionado comienza a oscilar con la animación shimmer satinada.
3. **Inferencia & Streaming**: A medida que los tokens (palabras) se van renderizando en el editor, el shimmer permanece activo sobre el texto recién insertado.
4. **Finalización**: Una vez que el último token es renderizado, la animación shimmer se retira suavemente del texto final.

## Tech Stack
- **Frontend Core**: React, TypeScript, Vite.
- **Editor**: TipTap React (ProseMirror).
- **Styling**: Vanilla CSS (CSS Variables) en `editor.css`.
- **Frameworks**: Framer Motion (para transiciones suaves).

## Commands
```bash
# Iniciar servidor de desarrollo frontend y backend
npm run dev

# Compilar la aplicación
npm run build

# Validar linting y formato
npm run lint
```

## Project Structure
Modificaremos y crearemos los siguientes archivos:
- `specs/SPEC-ISSUE-13.md` [NEW] - Este documento de especificación.
- `src/components/Editor/AIShimmer.ts` [NEW] - Extensión TipTap personalizada para el mark de shimmer.
- `src/components/Editor/TypographyEditor.tsx` [MODIFY] - Integración del shimmer y lógica de renderizado tipo streaming (token-by-token) de la respuesta de la IA.
- `src/components/Editor/AIActionsMenu.tsx` [MODIFY] - Ajustar el menú de acciones para que "Corregir estilo" se presente como "Mejorar redacción" u otra opción clara.
- `src/components/Editor/editor.css` [MODIFY] - Definición del diseño del shimmer con variables HSL, tonos grises/azules satinados, animaciones CSS clave `@keyframes` y soporte para dark mode.

## Code Style

### Extensión de TipTap `AIShimmer.ts`
```typescript
import { Mark, mergeAttributes } from "@tiptap/core";

export const AIShimmer = Mark.create({
  name: "aiShimmer",

  addOptions() {
    return {
      HTMLAttributes: {
        class: "ai-shimmer-active",
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: "span.ai-shimmer-active",
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes),
      0,
    ];
  },
});
```

### CSS del Shimmer (`editor.css`)
```css
@keyframes shimmer-animation {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

.ai-shimmer-active {
  background: linear-gradient(
    90deg,
    rgba(226, 232, 240, 0.4) 25%,
    rgba(203, 213, 225, 0.6) 37%,
    rgba(226, 232, 240, 0.4) 63%
  );
  background-size: 200% 100%;
  animation: shimmer-animation 1.5s ease-in-out infinite;
  border-radius: 4px;
  padding: 2px 0;
  color: inherit;
  transition: background 0.3s ease;
}

@media (prefers-color-scheme: dark) {
  .ai-shimmer-active {
    background: linear-gradient(
      90deg,
      rgba(30, 41, 59, 0.5) 25%,
      rgba(51, 65, 85, 0.8) 37%,
      rgba(30, 41, 59, 0.5) 63%
    );
    background-size: 200% 100%;
  }
}
```

## Testing Strategy
- **Manual Verification**:
  1. Seleccionar un párrafo en el editor.
  2. Abrir el menú de IA y hacer clic en "Mejorar redacción".
  3. Verificar que el fondo cambie de inmediato al shimmer degradado satinado (tonos grises/azules).
  4. Observar que el texto se reemplace gradualmente (typewriter effect) palabra por palabra, manteniendo el shimmer activo.
  5. Confirmar que el shimmer se elimina suavemente tan pronto como el último token de la respuesta de la IA se renderiza.
  6. Verificar que funciona de la misma manera en Dark Mode.

## Boundaries
- **Always do**: Mantener la compatibilidad con todas las demás acciones de IA (Traducir, Resumir, etc.), las cuales también se verán beneficiadas del efecto de shimmer y renderizado tipo streaming.
- **Ask first**: N/A
- **Never do**: Modificar componentes que no estén relacionados con el editor TipTap o el flujo de IA.

## Success Criteria
- Al presionar el botón "Mejorar redacción" (anteriormente "Corregir estilo"), el fondo del texto seleccionado parpadea con la animación shimmer satinada (grises/azules).
- El texto nuevo de la IA se inserta progresivamente simlando un flujo de tokens.
- El shimmer cubre exactamente la parte del documento que está en proceso de creación automatizada.
- Al renderizarse la última palabra, el shimmer se retira del documento, dejando el texto limpio.
