import { invoke } from "@tauri-apps/api/core";
import type { DocumentVersion, SaveDocumentVersionInput } from "./types";

const memoryHistory = new Map<string, DocumentVersion[]>();

export async function saveDocumentVersion(
  input: SaveDocumentVersionInput
): Promise<DocumentVersion> {
  try {
    return await invoke<DocumentVersion>("document_version_save", { input });
  } catch (error) {
    console.warn("Tauri document history unavailable, using memory history:", error);
    return saveMemoryVersion(input);
  }
}

export async function listDocumentVersions(
  documentId: string
): Promise<DocumentVersion[]> {
  try {
    return await invoke<DocumentVersion[]>("document_version_list", {
      documentId,
    });
  } catch (error) {
    console.warn("Tauri document history unavailable, using memory history:", error);
    return memoryHistory.get(documentId) ?? [];
  }
}

function saveMemoryVersion(input: SaveDocumentVersionInput): DocumentVersion {
  const timestamp = Date.now().toString();
  const version: DocumentVersion = {
    id: `${input.document_id}-${timestamp}`,
    document_id: input.document_id,
    created_at: timestamp,
    action_label: input.action_label,
    previous_content: input.previous_content,
    new_content: input.new_content,
  };

  const versions = memoryHistory.get(input.document_id) ?? [];
  const nextVersions = [version, ...versions].slice(0, 50);
  memoryHistory.set(input.document_id, nextVersions);

  return version;
}
