import { invoke } from "@tauri-apps/api/core";

export type ExportFormat = "pdf" | "docx";

export interface ExportResult {
  path: string;
  format: ExportFormat;
}

interface ExportRequest {
  html: string;
  format: ExportFormat;
  filename?: string;
}

export async function exportDocument({
  html,
  format,
  filename,
}: ExportRequest): Promise<ExportResult> {
  return await invoke<ExportResult>("export_document", {
    request: {
      html,
      format,
      filename,
    },
  });
}

export async function exportDocumentAsPdf(
  html: string,
  filename?: string
): Promise<ExportResult> {
  return exportDocument({
    html,
    format: "pdf",
    filename,
  });
}
