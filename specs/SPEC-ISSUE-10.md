# Spec: ISSUE #10 — Interfaz Visual de Selección y Previsualización de Módulos de Plantillas

## Objective

Crear la sección de catálogo de plantillas corporativas donde el usuario puede:
- Ver tarjetas elegantes segmentadas por categoría (Legal, Ventas, Recursos Humanos)
- Hacer clic en una plantilla para abrir un modal de previsualización
- Ver un skeleton loading mientras carga la estructura
- Visualizar el formato estructural con variables destacadas que la IA rellenará

**Usuario:** Usuario final que necesita seleccionar una plantilla corporativa para generar documentos con IA.

**Éxito:** El usuario puede navegar el catálogo, seleccionar una plantilla, y ver qué variables serán completadas por la IA antes de usar la plantilla.

## Tech Stack

- **Next.js** 16.2.6+ con App Router
- **React** 19.1
- **TypeScript** 5.8
- **Tailwind CSS** 4.x (necesario para shadcn/ui)
- **shadcn/ui** (componentes base: Card, Modal/Dialog, Skeleton, Badge)
- **Tauri** (para invocar comandos del backend Rust)

## Commands

```bash
Build:   npm run build
Dev:     npm run dev
Lint:    npm run lint
Tauri:   npm run tauri
```

## Project Structure

```
src/
├── app/
│   ├── layout.tsx              # Layout raíz (sin cambios)
│   ├── page.tsx                # Home - mantiene MultimodalChat
│   ├── templates/
│   │   └── page.tsx            # Nueva página: Catálogo de plantillas
│   └── globals.css             # Tailwind + CSS existente
├── components/
│   ├── templates/
│   │   ├── TemplateCatalog.tsx     # Componente principal del catálogo
│   │   ├── TemplateCard.tsx        # Tarjeta individual de plantilla
│   │   ├── TemplateModal.tsx       # Modal de previsualización
│   │   ├── TemplateSkeleton.tsx    # Skeleton loading
│   │   └── VariableHighlight.tsx   # Componente para variables destacadas
│   └── ui/                     # Componentes shadcn/ui
│       ├── card.tsx
│       ├── dialog.tsx
│       ├── skeleton.tsx
│       ├── badge.tsx
│       ├── button.tsx
│       └── separator.tsx
└── lib/
    ├── templates-service.ts    # Servicio para invocar comandos Tauri
    └── types.ts                # Tipos TypeScript para plantillas
```

## Code Style

### Tipos TypeScript

```typescript
export interface Template {
  id: number;
  name: string;
  category: string;
  required_variables: string[];
  system_prompt: string;
  base_text: string;
}

export interface CategoryGroup {
  category: string;
  templates: Template[];
}
```

### Componente TemplateCard

```tsx
// Tarjeta elegante con gradiente sutil basado en categoría
// Muestra: nombre, categoría badge, preview de variables
// Hover: elevación + borde destacado
```

### Modal de Previsualización

```tsx
// Usa Dialog de shadcn/ui
// Estructura:
// - Header: nombre + categoría badge
// - Body: TemplateSkeleton mientras carga, luego TemplatePreview
// - Footer: botón "Usar Plantilla" + "Cerrar"
// - VariableHighlight: muestra {{variable}} con colores distintivos
```

### Skeleton Loading

```tsx
// Patrón de加载 esqueleto para estructura de documento
// Líneas de diferentes anchos simulando texto
// Variables mostradas como rectángulos pulsantes
```

## Design Language

### Categorías y Colores

| Categoría | Color Primario | Gradiente | Badge |
|-----------|----------------|-----------|-------|
| Legal | #1e40af (azul profundo) | bg-blue-900/20 → bg-blue-800/20 | blue |
| Ventas | #047857 (verde esmeralda) | bg-emerald-900/20 → bg-emerald-800/20 | green |
| Recursos Humanos | #7c3aed (violeta) | bg-violet-900/20 → bg-violet-800/20 | violet |

### Variables Destacadas

- Background: `bg-amber-100 dark:bg-amber-900/30`
- Border: `border-amber-400 dark:border-amber-600`
- Text: `text-amber-700 dark:text-amber-400`
- Font: monospace para distinción
- Icono: 🔷 o similar para indicar "aquí la IA rellena"

### Animaciones

- Cards: hover scale 1.02, transición 200ms ease-out
- Modal: fade-in + scale desde 95% → 100%
- Skeleton: pulse animation 1.5s infinite
- Variables: glow sutil al hacer hover

## Testing Strategy

- **Framework:** Vitest para tests de componentes React
- **Ubicación:** `src/components/templates/*.test.tsx`
- **Cobertura mínima:**
  1. `TemplateCard` renderiza correctamente según categoría
  2. `TemplateModal` muestra skeleton y luego contenido
  3. `VariableHighlight` extrae y muestra variables de `{{variable}}`
  4. `TemplateCatalog` agrupa plantillas por categoría
- **Test de integración:** Verificar que el modal se abre al hacer click en card

## Boundaries

- **Always:**
  - Usar componentes shadcn/ui como base
  - Implementar skeleton loading para mejor UX
  - Categorizar plantillas con colores consistentes
  - Resaltar variables con formato distintivo
- **Ask first:**
  - Agregar nuevas dependencias
  - Cambiar estructura de datos de plantillas
  - Modificar el schema de la base de datos
- **Never:**
  - Hardcodear datos de plantillas (deben venir del backend)
  - Implementar CRUD de plantillas en este issue
  - Modificar la página de chat existente

## Success Criteria

1. ✅ La página `/templates` muestra un catálogo de tarjetas segmentadas por categoría
2. ✅ Cada tarjeta muestra: nombre, categoría con badge de color, lista de variables
3. ✅ Hacer clic en una tarjeta abre un modal con skeleton loading
4. ✅ El modal muestra la estructura del documento con variables destacadas en `{{variable}}`
5. ✅ Las variables usan color distintivo (ámbar) y fuente monospace
6. ✅ El modal tiene botones "Usar Plantilla" y "Cerrar"
7. ✅ La página es responsive (grid adapts de 1 a 3 columnas)
8. ✅ `npm run build` pasa sin errores
9. ✅ `npm run lint` pasa sin errores

## Open Questions

1. ¿El botón "Usar Plantilla" navega a una página de editor con la plantilla precargada? → **Sí, pasar template_id como query param a `/editor?template=ID`**
2. ¿Se requiere búsqueda/filtrado de plantillas? → **No para este issue, puede ser futuro**
3. ¿Las categorías son fijas o dinámicas desde la DB? → **Dinámicas ( agrupar por category field)**