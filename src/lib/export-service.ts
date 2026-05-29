import { invoke } from "@tauri-apps/api/core";
import type { StructuredTableColumn } from "@/lib/types";

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

export interface StructuredTableXlsxExportRequest {
  columns: StructuredTableColumn[];
  rows: Record<string, string>[];
  filename?: string;
}

export interface StructuredTableXlsxExportResult {
  path: string;
  format: "xlsx";
}

export async function exportStructuredTableAsXlsx({
  columns,
  rows,
  filename,
}: StructuredTableXlsxExportRequest): Promise<StructuredTableXlsxExportResult> {
  return await invoke<StructuredTableXlsxExportResult>("export_structured_table_xlsx", {
    request: {
      columns,
      rows,
      filename,
    },
  });
}
