# SPEC.md — Issue #06: Editor Tipográfico En Línea (Notion-Style)

## 1. Concepto y Visión

Un editor de texto enriquecido minimalista que combina la fluidez de escritura de Notion con la potencia de TipTap. La experiencia debe ser **distraída**: el usuario escribe y el texto se transforma instantáneamente en estructura tipográfica jerárquica. Sin barras de herramientas visibles, sin modales, solo el texto y su forma. El aspecto visual es refinado, corporativo, con tipografía de alta legibilidad y espaciado generoso.

## 2. Diseño Estético

### Dirección Visual
- **Estilo**: Notion minimalism — limpio, refinado, profesional
- **Referencia**: Documentos corporativos de alta gama (Bloomberg, Linear)
- **Personalidad**: Confianza, precisión, sofisticación discreta

### Paleta de Colores
```css
--bg-primary: #ffffff;
--bg-secondary: #fafafa;
--bg-hover: #f5f5f5;
--text-primary: #1a1a1a;
--text-secondary: #6b6b6b;
--text-placeholder: #b0b0b0;
--accent: #2563eb;
--border: #e5e5e5;
--selection: rgba(37, 99, 235, 0.15);
```

### Tipografía
- **Font Principal**: `"Geist", "Inter", system-ui, sans-serif`
- **Headings**:
  - H1: 2.5rem (40px), font-weight: 700, line-height: 1.2
  - H2: 1.875rem (30px), font-weight: 600, line-height: 1.3
  - H3: 1.375rem (22px), font-weight: 600, line-height: 1.4
- **Body**: 1rem (16px), font-weight: 400, line-height: 1.75
- **Code/Mono**: `"Geist Mono", "SF Mono", monospace`

### Sistema Espacial
- **Padding interno del editor**: 3rem horizontal, 2rem vertical
- **Espaciado entre bloques**: 1rem
- **Margen para títulos**: 1.5rem top, 0.5rem bottom
- **Max-width del contenido**: 720px (lectura óptima)

### Movimiento
- **Placeholder fade-in**: opacity 0→1, 200ms ease-out
- **Cursor blink**: 530ms interval
- **Transformaciones Markdown**: instantáneas (0ms), sin animación perceptible

## 3. Arquitectura del Editor (TipTap)

### 3.1 Extensiones Requeridas

| Extensión | Función |
|-----------|---------|
| `StarterKit` | Base: paragraphs, headings, lists, bold, italic, code |
| `Typography` | Smart quotes, dashes, ellipsis |
| `Placeholder` | Placeholder text cuando vacío |
| `CharacterCount` | Contador de caracteres |
| `Link` | Hipervínculos (Ctrl+K) |

### 3.2 Atajos de Teclado (Markdown Input)

| Input | Resultado | Descripción |
|-------|-----------|-------------|
| `# ` | Heading 1 | Slash + espacio tras # |
| `## ` | Heading 2 | Slash + espacio tras ## |
| `### ` | Heading 3 | Slash + espacio tras ### |
| `- ` | Bullet list | Guión + espacio |
| `* ` | Bullet list | Asterisco + espacio |
| `1. ` | Ordered list | Número + punto + espacio |
| `> ` | Blockquote | Mayor que + espacio |
| `**texto**` | Bold | Asteriscos entourando texto |
| `*texto*` | Italic | Asteriscos simples |
| `` `codigo` `` | Inline code | Backticks |
| `---` | Horizontal rule | Triple guion + enter |

### 3.3 Comandos de Teclado

| Atajo | Acción |
|-------|--------|
| `Ctrl/Cmd + B` | Bold |
| `Ctrl/Cmd + I` | Italic |
| `Ctrl/Cmd + K` | Insertar link |
| `Ctrl/Cmd + Shift + H` | Toggle heading (cycle H1→H2→H3) |
| `Ctrl/Cmd + Shift + 7` | Toggle ordered list |
| `Ctrl/Cmd + Shift + 8` | Toggle bullet list |

## 4. Modelo de Datos

```typescript
// Estado del editor
interface EditorState {
  content: string;           // HTML o JSON del contenido
  wordCount: number;         // Contador de palabras
  charCount: number;         // Contador de caracteres
  isEmpty: boolean;          // Si está vacío
}

// API de callbacks
interface EditorCallbacks {
  onChange?: (content: string) => void;
  onFocus?: () => void;
  onBlur?: () => void;
}
```

## 5. Estructura de Archivos

```
src/components/
├── Editor/
│   ├── TypographyEditor.tsx    # Componente principal
│   ├── EditorToolbar.tsx       # Barra de herramientas minimalista (solo cuando hay selección)
│   └── editor.css              # Estilos del editor
```

## 6. Interfaz Visual

### Layout del Editor
```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│         ┌─────────────────────────────────────┐             │
│         │                                     │             │
│         │  [Placeholder: "Escribe algo..."]   │             │
│         │                                     │             │
│         │  # Título Principal                  │             │
│         │                                     │             │
│         │  Texto del párrafo normal con       │             │
│         │  múltiples líneas que se ajusta     │             │
│         │  al ancho máximo de lectura.        │             │
│         │                                     │             │
│         │  ## Subtítulo                       │             │
│         │                                     │             │
│         │  - Elemento de lista                │             │
│         │  - Otro elemento                    │             │
│         │                                     │             │
│         │  Texto con **negrita** y            │             │
│         │  *cursiva* en la misma línea.       │             │
│         │                                     │             │
│         └─────────────────────────────────────┘             │
│                                                             │
│         ┌─────────────────────────────────────┐             │
│         │  128 palabras · 742 caracteres      │             │
│         └─────────────────────────────────────┘             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Estados del Editor

| Estado | Comportamiento Visual |
|--------|----------------------|
| **Vacío** | Muestra placeholder gris claro centrado |
| **Escribiendo** | Cursor parpadeante, texto aparece instantáneamente |
| **Título** | Texto más grande, bold, sin prefijo `#` visible |
| **Lista** | Bullet point visible a la izquierda, indentación consistente |
| **Código inline** | Fondo gris claro, tipografía monospace |
| **Link** | Texto azul con underline, tooltip en hover |
| **Selección** | Fondo azul translúcido |

## 7. Comportamiento del Renderizado

### Requisitos de Rendimiento
- **FPS objetivo**: 60fps constante durante escritura
- **Latencia de transformación**: < 16ms (sin frames dropped)
- **Approach**: Usar CSS `contenteditable` nativo de TipTap, no virtualization necesaria para texto simple

### Rendering Strategy
1. TipTap usa `contenteditable` nativo del navegador
2. Las transformaciones Markdown ocurren en el evento `input` via TipTap Input Rules
3. El rendering es directo al DOM, sin reconciliación React costosa
4. El componente es un "controlled editor" pero solo para callbacks, no para cadakeystroke

## 8. Criterios de Aceptación

| # | Criterio | Verificación |
|---|----------|--------------|
| 1 | Escribir `# ` transforma instantáneamente a H1 | Escribir `# prueba` y ver heading |
| 2 | Escribir `## ` transforma a H2 | Escribir `## subtitulo` |
| 3 | Escribir `### ` transforma a H3 | Escribir `### titulo pequeno` |
| 4 | Escribir `- ` o `* ` crea bullet list | Escribir `- item` |
| 5 | Escribir `**texto**` muestra bold inline | Escribir `**negrita**` |
| 6 | Escribir `*texto*` muestra italic inline | Escribir `*cursiva*` |
| 7 | Contador de palabras/caracteres se actualiza en tiempo real | Escribir y observar contador |
| 8 | Placeholder visible cuando editor vacío | Limpiar editor |
| 9 | Selection toolbar aparece al seleccionar texto | Seleccionar texto |
| 10 | No hay frames dropped durante escritura rápida | Escribir rápido |

## 9. Dependencias a Instalar

```bash
npm install @tiptap/react @tiptap/starter-kit @tiptap/extension-placeholder @tiptap/extension-typography @tiptap/extension-character-count @tiptap/extension-link
```

## 10. Notas de Implementación

- El editor debe ser usado como componente standalone, no integrado en el chat (aún)
- La página principal puede usar este editor como block de notas o para entrada de prompts estructurados
- El CSS usa CSS Modules o scoped styles para evitar colisiones
- Soporte para dark mode con variables CSS adaptadas