import { invoke } from "@tauri-apps/api/core";

import type { ExtractedDocument } from "./types";

export async function extractDocumentText(path: string): Promise<ExtractedDocument> {
  return await invoke<ExtractedDocument>("extract_document_text", { path });
}
