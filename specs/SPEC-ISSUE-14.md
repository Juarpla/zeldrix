# SPEC-ISSUE-14.1: Exportador Nativo Local a Formatos Corporativos (PDF y DOCX)

## Objective
Implementar exportacion local desde el editor TipTap hacia archivos corporativos en el escritorio del usuario, empezando por PDF como flujo principal de aceptacion y dejando DOCX preparado como ruta soportada por el backend. El usuario objetivo es una persona que redacta documentos corporativos en Zeldrix y necesita generar archivos imprimibles sin depender de servicios externos ni dialogos adicionales.

ASSUMPTIONS I'M MAKING:
1. El contenido fuente del editor es HTML estructurado generado por TipTap.
2. El flujo obligatorio de Issue #14 es PDF; DOCX puede exponerse en el contrato del backend, pero debe responder con un error claro de formato no soportado hasta una tarea futura.
3. "Escritorio de la PC" significa usar el directorio Desktop resuelto localmente por Rust, con fallback a un directorio escribible si no existe.
4. "Formato identico al editor" significa conservar jerarquia, parrafos, listas, negrita, cursiva, codigo inline, links visibles, margenes corporativos y fuente base de la vista actual, dentro de las limitaciones razonables de renderizado local.
5. La exportacion debe funcionar sin red y sin sidecars adicionales.
6. La API Tauri/Wry disponible solo expone impresion con dialogo del sistema, por lo que el flujo de un clic debe seguir escribiendo el archivo desde Rust.

## Tech Stack
- Frontend: Next.js client component, React, TypeScript, TipTap.
- Desktop runtime: Tauri v2.
- Backend: Rust commands registrados en `src-tauri/src/lib.rs`.
- Export rendering: modulo Rust local que genera PDF en disco sin APIs remotas; el parser HTML usa `scraper` y el serializador PDF mantiene estilos inline soportados.

## Commands
```bash
npm run build
npm run dev
npm run lint
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Project Structure
- `specs/SPEC-ISSUE-14.md` - especificacion viva del exportador.
- `src/app/editor/page.tsx` - UI de exportacion y estados de exito/error.
- `src/lib/export-service.ts` - wrapper TypeScript para invocar comandos Tauri y fallback controlado.
- `src-tauri/src/exporter/mod.rs` - modulo Rust para parsear HTML TipTap, resolver rutas y generar archivos.
- `src-tauri/src/lib.rs` - registro de comandos Tauri.
- `src-tauri/Cargo.toml` - dependencias locales de generacion si son necesarias.

## Code Style
Frontend wrapper:
```typescript
export async function exportDocumentAsPdf(html: string): Promise<ExportResult> {
  return invoke<ExportResult>("export_document", {
    request: { html, format: "pdf", filename: "documento-zeldrix" },
  });
}
```

Rust command:
```rust
#[tauri::command]
async fn export_document(
    app: tauri::AppHandle,
    request: ExportRequest,
) -> Result<ExportResult, String> {
    exporter::export_document(&app, request)
        .await
        .map_err(|error| error.to_string())
}
```

Conventions:
- Commands Tauri usan tipos owned y devuelven `Result<T, String>` en la frontera IPC.
- El modulo Rust interno usa errores tipados con `thiserror`.
- El frontend no construye rutas absolutas; muestra la ruta retornada por backend.
- La UI evita bloquear el editor: botones deshabilitados mientras exporta y mensajes breves al terminar.

## Testing Strategy
- Unit tests Rust para:
  - resolver nombres seguros de archivo;
  - convertir HTML TipTap a bloques con estilos inline;
  - generar bytes/archivo PDF no vacio desde HTML minimo;
  - conservar listas, headings, negrita, cursiva, codigo inline y links visibles en el PDF;
  - rechazar DOCX con un error explicito porque aun no esta implementado por decision de alcance;
  - rechazar contenido vacio con error claro.
- Build checks:
  - `npm run build`
  - `cd src-tauri && cargo test`
  - `cd src-tauri && cargo clippy --all-targets --all-features --locked -- -D warnings`
- Manual verification:
  1. Ejecutar `npm run dev` dentro de Tauri.
  2. Abrir `/editor`, escribir contenido con titulo, parrafo, lista y negrita.
  3. Hacer clic en `Exportar como PDF`.
  4. Confirmar que se crea un `.pdf` en Desktop.
  5. Abrir el PDF en un lector local y verificar que conserva margenes, fuente base, jerarquia y listas.

## Boundaries
- Always:
  - Generar archivos localmente.
  - Mantener el contenido fuente como HTML TipTap.
  - Registrar todo comando Tauri en `generate_handler!`.
  - Resolver rutas con APIs locales, no strings hardcodeadas.
  - Reportar errores de exportacion al usuario.
- Ask first:
  - Instalar herramientas externas del sistema.
  - Cambiar el modelo de datos del editor.
  - Agregar telemetria o subida de documentos.
  - Hacer que DOCX sea requisito bloqueante del primer merge si requiere una libreria pesada o conversion imperfecta.
- Never:
  - Enviar HTML/documentos a servicios remotos.
  - Guardar fuera de Desktop sin informar el path final.
  - Sobrescribir archivos existentes sin generar un nombre unico.
  - Romper el modo mock del frontend cuando Tauri no esta disponible.

## Implementation Plan
1. Crear modulo Rust `exporter` con contrato `ExportRequest`, `ExportFormat`, `ExportResult` y errores tipados.
2. Implementar sanitizacion minima, envoltorio HTML imprimible y resolucion de Desktop con nombres unicos.
3. Mejorar generacion PDF local para que use runs de texto con fuente normal, bold, italic, bold italic, monospace y color de link.
4. Mantener DOCX como contrato backend no implementado intencionalmente, retornando `UnsupportedDocx`.
5. Registrar `export_document` en Tauri.
6. Crear wrapper TypeScript `export-service.ts`.
7. Agregar boton `Exportar como PDF` al editor con estado de carga y mensajes de resultado.
8. Ejecutar verificaciones automatizadas y registrar limitaciones si alguna libreria impide equivalencia visual completa.

## Tasks
- [ ] Backend contract
  - Acceptance: `export_document` compila, esta registrado en Tauri y retorna ruta/formato.
  - Verify: `cd src-tauri && cargo test`.
  - Files: `src-tauri/src/exporter/mod.rs`, `src-tauri/src/lib.rs`.
- [ ] PDF generation
  - Acceptance: HTML minimo produce un archivo `.pdf` no vacio y con nombre unico; HTML con strong/em/code/a/listas mantiene senales visuales en el PDF.
  - Verify: unit test Rust.
  - Files: `src-tauri/src/exporter/mod.rs`, `src-tauri/Cargo.toml`.
- [ ] DOCX unsupported contract
  - Acceptance: una solicitud `format: docx` no intenta escribir archivo y retorna un error claro de formato no implementado.
  - Verify: unit test Rust.
  - Files: `src-tauri/src/exporter/mod.rs`.
- [ ] Editor UI
  - Acceptance: un clic en `Exportar como PDF` invoca backend, deshabilita el boton mientras corre y muestra el path guardado.
  - Verify: `npm run build`.
  - Files: `src/app/editor/page.tsx`, `src/lib/export-service.ts`.
- [ ] Final verification
  - Acceptance: build y tests pasan; PDF manual se abre correctamente.
  - Verify: comandos de Testing Strategy.
  - Files: no aplica.

## Success Criteria
- El editor muestra una accion clara `Exportar como PDF`.
- Al hacer clic, el backend genera localmente un archivo PDF valido en Desktop.
- El resultado tiene extension `.pdf`, no esta vacio y puede abrirse en un lector PDF estandar.
- El PDF conserva margenes corporativos, fuente base, headings, parrafos, listas, negrita, cursiva, codigo inline, links visibles e imagenes como bloques identificables con alt text cuando el formato de imagen no pueda incrustarse directamente.
- Si la exportacion falla, el usuario ve un error accionable y el editor no pierde contenido.
- Si se solicita DOCX, el backend responde que DOCX no esta implementado intencionalmente en esta version.

## Open Questions
- DOCX queda fuera de alcance para Issue #14; se implementara en una tarea futura si se aprueba la libreria/formato de salida.
- Confirmar si el nombre de archivo debe derivarse de la plantilla/documento o usar un nombre fijo con timestamp.
