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