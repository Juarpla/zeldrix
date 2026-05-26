# SPEC.md — ISSUE #04: Capa de Entrada Multimodal Intercalada (Visión y Audio con Gemma 4)

## 1. Objective

Crear el pipeline en el backend para procesar formatos no textuales (imágenes PNG/JPEG, audio WAV/MP3) que soporta nativamente gemma-4-E2B-it. El usuario puede arrastrar una imagen o fragmento de audio, y el sistema lo codifica usando los esquemas requeridos por el backend multimodal de llama.cpp, enviándolo de forma intercalada con el texto del prompt.

**User Story**: Como usuario, quiero arrastrar una imagen de factura y pedir "Extrae el total" para obtener un análisis estructurado del documento visual de forma local.

**Criterio de Aceptación**: Enviar un prompt como `[Imagen de factura] + "Extrae el total"` debe retornar una cadena de texto estructurada analizando el documento visualmente de forma local.

**Nota importante**: El modelo y el mmproj son **archivos separados** descargados desde el mismo enlace de Hugging Face:
- `gemma-4-E2B-it-IQ4_XS.gguf` (modelo principal)
- `mmproj-gemma-4-E2B-it-IQ4_XS.gguf` (proyector multimodal)

El servidor debe iniciarse con ambos: `llama-server -m model.gguf --mmproj mmproj.gguf`

## 2. Technical Stack

- **Backend**: Rust con Tauri v2, reqwest para HTTP al llama-server
- **Frontend**: React con Next.js (App Router), drag & drop con react-dropzone
- **Modelo**: gemma-4-E2B-it-GGUF de ggml-org con soporte multimodal nativo ( `-hf` flag)
- **Sidecar**: llama-server corriendo localmente, OpenAI-compatible API en `/v1/chat/completions`

## 3. Arquitectura

### 3.1 Backend (Rust)

```
src-tauri/src/
├── multimodal/
│   ├── mod.rs              # Estado y comandos Tauri
│   ├── encoder.rs          # Codificación Base64 de imágenes/audio
│   └── types.rs            # Tipos para contenido multimodal
├── chat/
│   ├── mod.rs              # Módulo de chat con multimodalidad
│   ├── session.rs          # Gestión de sesiones de chat
│   └── message.rs          # Modelo de mensajes multimodales
```

### 3.2 Estado Global

```rust
// Tipos para contenido multimodal
pub enum MediaContent {
    Image { data: Vec<u8>, mime_type: String },
    Audio { data: Vec<u8>, mime_type: String },
}

pub struct MultimodalMessage {
    pub role: String,
    pub content: Vec<ContentPart>,  // text, image_url, audio_url
}

pub enum ContentPart {
    Text(String),
    ImageUrl { url: String },       // data:image/png;base64,...
    AudioUrl { url: String },       // data:audio/wav;base64,...
}
```

### 3.3 Comandos Tauri

| Comando | Descripción |
|---------|-------------|
| `chat_complete_multimodal(messages)` | Envía mensaje con contenido multimodal y retorna respuesta |
| `encode_image(path)` | Codifica imagen a base64 para uso en prompts |
| `encode_audio(path)` | Codifica audio a base64 para uso en prompts |

### 3.4 Flujo de Chat Multimodal

```
1. Frontend: Usuario arrastra imagen/audio
2. Frontend: FileReader convierte a Base64, construye ContentPart[]
3. Frontend: Envía mensaje con content: [{type: "text", text}, {type: "image_url", image_url: {...}}]
4. Backend: Recibe mensaje, reenvía a llama-server /v1/chat/completions
5. Backend: Retorna respuesta de texto del modelo
```

### 3.5 Formato de Request a llama-server

```json
{
  "model": "gemma-4-E2B-it",
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "Extrae el total de esta factura"},
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,iVBORw0KGgo..."
          }
        }
      ]
    }
  ],
  "stream": false
}
```

## 4. Frontend (React)

### 4.1 Componentes

```
src/components/
├── MultimodalChat.tsx       # Contenedor principal de chat multimodal
├── MediaDropZone.tsx        # Zona de drag & drop para archivos
├── MediaPreview.tsx         # Preview de archivos adjuntos
├── ChatMessage.tsx          # Mensaje individual con contenido multimodal
└── ChatInput.tsx            # Input con textarea + zona de drop
```

### 4.2 Modelo de Datos (TypeScript)

```typescript
type ContentPart =
  | { type: 'text'; text: string }
  | { type: 'image_url'; image_url: { url: string } }
  | { type: 'audio_url'; audio_url: { url: string } };

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: ContentPart[];
  timestamp: Date;
}

interface MediaFile {
  id: string;
  type: 'image' | 'audio';
  name: string;
  size: number;
  mimeType: string;
  dataUrl: string;  // base64 data URL
}
```

### 4.3 Interfaz Visual

**Layout principal**:
- Panel lateral izquierdo: Lista de conversaciones (futuro)
- Área central: Chat con mensajes multimodales
- Zona inferior: Input con textarea + drop zone

**Drop Zone**:
- Borde punteado cuando está vacío
- Acepta drag & drop de PNG, JPEG, WAV, MP3
- Muestra preview de archivos adjuntos antes de enviar
- Botón para adjuntar archivos (alternativa a drag)

**Mensajes**:
- Mensaje de usuario: Alineado a la derecha, fondo azul claro
- Mensaje de asistente: Alineado a la izquierda, fondo gris claro
- Archivos adjuntos: Thumbnail para imágenes, icono para audio

## 5. Project Structure

```
zeldrix/
├── src/
│   ├── app/
│   │   ├── page.tsx           # Redirect a /chat
│   │   ├── layout.tsx
│   │   └── chat/
│   │       └── page.tsx       # Página principal de chat multimodal
│   ├── components/
│   │   ├── MultimodalChat.tsx
│   │   ├── MediaDropZone.tsx
│   │   ├── MediaPreview.tsx
│   │   ├── ChatMessage.tsx
│   │   └── ChatInput.tsx
│   └── lib/
│       └── multimodal.ts      # Utilidades frontend para codificación
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs             # Registrar comandos multimodales
│   │   ├── multimodal/
│   │   │   ├── mod.rs
│   │   │   ├── encoder.rs
│   │   │   └── types.rs
│   │   └── chat/
│   │       ├── mod.rs
│   │       ├── session.rs
│   │       └── message.rs
│   └── capabilities/
│       └── default.json       # Agregar permisos necesarios
```

## 6. Commands

```bash
# Build
npm run build

# Dev
npm run dev

# Type check
npx tsc --noEmit
```

## 7. Code Style

### Rust (Backend)

```rust
// Error handling con Result
#[tauri::command]
async fn chat_complete_multimodal(
    messages: Vec<MultimodalMessage>,
) -> Result<String, String> {
    // Conversión a formato OpenAI
    let oai_messages: Vec<ChatMessage> = messages
        .into_iter()
        .map(convert_to_openai_format)
        .collect();

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://127.0.0.1:{}/v1/chat/completions", port))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.text().await.unwrap())
}
```

### TypeScript (Frontend)

```typescript
// Componente de drop zone
export function MediaDropZone({ onFilesDropped }: Props) {
  const handleDrop = useCallback((acceptedFiles: File[]) => {
    const processed = acceptedFiles.map(file => ({
      id: crypto.randomUUID(),
      type: file.type.startsWith('image/') ? 'image' : 'audio',
      name: file.name,
      size: file.size,
      mimeType: file.type,
      dataUrl: '', //Filled by FileReader
    }));
    onFilesDropped(processed);
  }, [onFilesDropped]);

  // ... implementation
}
```

## 8. Testing Strategy

- **Unit tests (Rust)**: Encoder para Base64, conversión de tipos
- **Unit tests (TypeScript)**: Helpers de codificación, parsing de mensajes
- **Integration**: Chat multimodal end-to-end con imagen real

## 9. Boundaries

**Always**:
- Usar `Result<T, E>` para operaciones fallibles en Rust
- Validar tipos MIME de archivos antes de procesar
- Codificar archivos a Base64 antes de enviar al servidor

**Ask first**:
- Cambiar el formato de mensajes entre frontend y backend
- Agregar nuevos tipos de contenido (video, documentos)

**Never**:
- Enviar archivos binarios directamente sin codificar
- Usar `unwrap()` en código de producción

## 10. Success Criteria

1. ✅ Drop de imagen PNG/JPEG muestra preview en el chat
2. ✅ Mensaje con imagen se envía correctamente al llama-server
3. ✅ Respuesta del modelo analiza el contenido de la imagen
4. ✅ Audio WAV/MP3 se procesa correctamente (experimental)
5. ✅ Múltiples archivos multimodales pueden enviarse en un mismo mensaje
6. ✅ Error claro si el archivo no es soportado

## 11. Open Questions

1. ¿El modelo gemma-4-E2B-it ya tiene el mmproj bundle o necesita descargarse por separado?
   - **Respuesta**: Los modelos de ggml-org con `-hf` ya incluyen el mmproj integrado
2. ¿Audio está suficientemente estable para incluir en v1?
   - **Decisión**: Incluir como feature experimental con disclaimer
3. ¿Se necesita guardar historial de chat?
   - **Decisión**: MVP no incluye persistencia - solo sesión en memoria