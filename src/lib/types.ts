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

export interface DocumentVersion {
  id: string;
  document_id: string;
  created_at: string;
  action_label: string;
  previous_content: string;
  new_content: string;
}

export interface SaveDocumentVersionInput {
  document_id: string;
  action_label: string;
  previous_content: string;
  new_content: string;
}

export type DocumentFormat = "pdf" | "docx" | "xlsx" | "plaintext";

export interface ExtractedDocument {
  path: string;
  file_name: string;
  format: DocumentFormat;
  text: string;
  char_count: number;
  byte_count: number;
  extraction_millis: number;
}

export type Category =
  | "Legal"
  | "Ventas"
  | "Recursos Humanos"
  | string;

export const CATEGORY_COLORS: Record<string, { bg: string; border: string; badge: "legal" | "ventas" | "recursos_humanos" | "default" }> = {
  "legal": {
    bg: "bg-blue-900/20",
    border: "border-blue-800/50",
    badge: "legal",
  },
  "ventas": {
    bg: "bg-emerald-900/20",
    border: "border-emerald-800/50",
    badge: "ventas",
  },
  "recursos humanos": {
    bg: "bg-violet-900/20",
    border: "border-violet-800/50",
    badge: "recursos_humanos",
  },
};
