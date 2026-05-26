# Spec: ISSUE #11 — Formulario Dinámico Asistido por IA para el Relleno de Contratos e Informes

## Objective

Crear un panel lateral split view en el editor que permita:
- **Panel izquierdo:** Recopilar inputs del usuario (campos estructurados o texto libre describiendo la situación)
- **Panel derecho:** Mostrar el editor de documento
- **IA:** Procesar el bloque de texto y extraer inteligentemente los datos requeridos por la plantilla

**Usuario:** Profesional que necesita generar documentos corporativos rápidamente describing una situación en lenguaje natural.

**Éxito:** Si el usuario escribe "Págale 1500 soles a Juan Pérez este fin de mes" en el panel izquierdo, la IA rellena automáticamente los campos `{{monto}}`, `{{beneficiario}}` y `{{fecha}}` del formulario.

## Tech Stack

- **Next.js** 16.2.6+ con App Router
- **React** 19.1
- **TypeScript** 5.8
- **Tailwind CSS** 4.x (ya instalado)
- **shadcn/ui** (componentes: Input, Textarea, Button, Card, ScrollArea)
- **Tauri** (backend Rust con prompt engineering para extracción de entidades)

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
│   └── editor/
│       └── page.tsx              # Modificado: Split view con panel AI
├── components/
│   ├── editor/
│   │   ├── AIAbstractForm.tsx        # NUEVO: Panel izquierdo con formulario
│   │   ├── VariableInput.tsx         # NUEVO: Campo individual del formulario
│   │   ├── AIAbstractPanel.tsx       # NUEVO: Panel wrapper con tabs
│   │   ├── editor.css                # Existing
│   │   └── toolbar.css               # Existing
│   └── ui/
│       ├── input.tsx                 # NUEVO: shadcn input
│       ├── textarea.tsx              # NUEVO: shadcn textarea
│       ├── scroll-area.tsx           # NUEVO: shadcn scroll-area
│       └── tabs.tsx                  # NUEVO: shadcn tabs
└── lib/
    ├── aiService.ts              # Modificado: agregar extractEntities
    └── templates-service.ts      # Existing (para obtener template)
```

## Design

### Split View Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  Header: Template name + "Usar IA" toggle                       │
├─────────────────────────────┬───────────────────────────────────┤
│                             │                                   │
│   AI ABSTRACT PANEL         │         DOCUMENT EDITOR           │
│   (400px fixed width)       │         (flex-1)                  │
│                             │                                   │
│   ┌─────────────────────┐   │                                   │
│   │ [Texto] [Campos]    │   │   Editor con tipografía           │
│   └─────────────────────┘   │                                   │
│                             │                                   │
│   TEXTO LIBRE:              │   Documento con {{variables}}     │
│   ┌─────────────────────┐   │   resaltadas                      │
│   │                     │   │                                   │
│   │ Págale 1500 soles   │   │                                   │
│   │ a Juan Pérez...     │   │                                   │
│   │                     │   │                                   │
│   └─────────────────────┘   │                                   │
│                             │                                   │
│   [Procesar con IA]         │                                   │
│                             │                                   │
│   CAMPOS:                   │                                   │
│   ┌─────────────────────┐   │                                   │
│   │ monto        [1500] │   │                                   │
│   │ beneficiario [Juan] │   │                                   │
│   │ fecha       [fin]   │   │                                   │
│   └─────────────────────┘   │                                   │
│                             │                                   │
│   [Aplicar al Documento]    │                                   │
│                             │                                   │
└─────────────────────────────┴───────────────────────────────────┘
```

### Colores de Variables

- Input focus: `ring-2 ring-amber-400`
- Input filled: `bg-amber-50 dark:bg-amber-900/20 border-amber-300`
- AI processing: `animate-pulse bg-blue-100`

## Code Style

### AIAbstractForm Component

```tsx
interface VariableInputProps {
  name: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  isAIFilled?: boolean;
}
```

### AI Extraction Service

```typescript
interface ExtractedEntity {
  variable: string;
  value: string;
  confidence: number; // 0-1
}

// New function in aiService.ts
export async function extractEntities(
  text: string,
  requiredVariables: string[]
): Promise<ExtractedEntity[]>
```

### Prompt Engineering (Backend)

El prompt para extracción de entidades será:

```
Eres un asistente de extracción de entidades. Del siguiente texto, extrae los valores para cada variable requerida.

Variables a extraer: [lista de variables]
Texto: [input del usuario]

Responde en JSON:
{
  "entities": [
    {"variable": "monto", "value": "1500 soles", "confidence": 0.95},
    {"variable": "beneficiario", "value": "Juan Pérez", "confidence": 0.98}
  ]
}

Reglas:
- Interpreta lenguaje natural (ej: "este fin de mes" → fecha calculada)
- Asigna confidence bajo si hay ambigüedad
- Si no puedes extraer, usa null con confidence 0
```

## Testing Strategy

- **Framework:** Vitest
- **Ubicación:** `src/components/editor/*.test.tsx`
- **Cobertura mínima:**
  1. `VariableInput` renderiza y responde a cambios
  2. `AIAbstractForm` extrae variables de texto libre
  3. AI service mock returns correct entity format
  4. "Aplicar al Documento" reemplaza {{variables}} en el editor

## Boundaries

- **Always:**
  - Split view es responsive (colapsa a tabs en mobile)
  - Animación de loading mientras IA procesa
  - Campos mantienen valores incluso si AI falla
  - El usuario puede editar campos manualmente después de IA
- **Ask first:**
  - Cambiar el mecanismo de split view
  - Agregar más campos AI oltre extraction
- **Never:**
  - Sobrescribir campos que el usuario ya editó manualmente
  - Hacer obligatoria la sección AI
  - Guardar datos de formulario sin aplicar al documento

## Success Criteria

1. ✅ Split view visible con panel izquierdo de 400px y editor derecho flex-1
2. ✅ Panel izquierdo tiene tabs: "Texto Libre" y "Campos Estructurados"
3. ✅ Textarea para entrada de texto libre
4. ✅ Lista de campos estructurados basados en `required_variables` del template
5. ✅ Botón "Procesar con IA" llama al servicio de extracción
6. ✅ Campos se auto-llenan con valores extraídos (ej: monto=1500, beneficiario=Juan Pérez)
7. ✅ Indicador visual de campo completado por IA (borde ámbar)
8. ✅ Botón "Aplicar al Documento" reemplaza {{variables}} en el editor
9. ✅ `npm run build` pasa sin errores

## Open Questions

1. ¿Qué pasa si la IA no puede extraer una variable? → **Campo queda vacío, usuario lo completa manualmente**
2. ¿El texto libre se mantiene después de aplicar? → **Sí, para referencia**
3. ¿Cómo manejar fechas relativas ("este viernes")? → **La IA las interpreta y calcula fecha real**
4. ¿Se puede usar sin seleccionar plantilla? → **Sí, campos vacíos hasta que se seleccione template**