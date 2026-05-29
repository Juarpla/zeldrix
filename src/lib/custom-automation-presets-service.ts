import { invoke } from "@tauri-apps/api/core";
import { AutomationShortcut } from "./automations-data";

export type CustomAutomationIconName = "translate" | "table" | "briefcase" | "email" | "reply";
export type CustomAutomationOutputType = "text" | "table";

export interface CustomAutomationPreset {
  id: number;
  title: string;
  icon_name: CustomAutomationIconName;
  base_prompt: string;
  output_type: CustomAutomationOutputType;
}

export interface CustomAutomationPresetInput {
  title: string;
  icon_name: CustomAutomationIconName;
  base_prompt: string;
  output_type: CustomAutomationOutputType;
}

export interface CustomAutomationRunRequest {
  preset_id: number;
  input_text: string;
}

export interface CustomAutomationRunResult {
  output_type: CustomAutomationOutputType;
  content: string;
}

export const CORPORATE_AUTOMATION_ICONS: Array<{
  id: CustomAutomationIconName;
  label: string;
}> = [
  { id: "translate", label: "Traducción" },
  { id: "table", label: "Tabla" },
  { id: "briefcase", label: "Corporativo" },
  { id: "email", label: "Correo" },
  { id: "reply", label: "Respuesta" },
];

export async function listCustomAutomationPresets(): Promise<CustomAutomationPreset[]> {
  return invoke<CustomAutomationPreset[]>("custom_automation_preset_list");
}

export async function createCustomAutomationPreset(
  input: CustomAutomationPresetInput,
): Promise<CustomAutomationPreset> {
  return invoke<CustomAutomationPreset>("custom_automation_preset_create", { input });
}

export async function runCustomAutomationPreset(
  request: CustomAutomationRunRequest,
): Promise<CustomAutomationRunResult> {
  return invoke<CustomAutomationRunResult>("custom_automation_preset_run", { request });
}

export function customPresetToShortcut(preset: CustomAutomationPreset): AutomationShortcut {
  const isTable = preset.output_type === "table";

  return {
    id: `custom-${preset.id}`,
    title: preset.title,
    description: isTable
      ? "Preset personalizado con salida tabular definida por el usuario."
      : "Preset personalizado con prompt corporativo definido por el usuario.",
    category: "custom",
    difficulty: "medium",
    estimatedSeconds: 4,
    iconName: preset.icon_name,
    inputs: [
      {
        id: "customInputText",
        label: "Texto de Entrada",
        placeholder: "Pega o arrastra aquí el bloque de texto que procesará este preset...",
        type: "textarea",
      },
    ],
    steps: [
      { label: "Lectura del texto", description: "Validando el bloque de entrada del usuario." },
      { label: "Prompt personalizado", description: "Aplicando la lógica corporativa guardada en el preset." },
      { label: "Formato de salida", description: isTable ? "Preparando una respuesta tabular." : "Preparando una respuesta textual." },
    ],
    mockOutput: "",
    isCustomPreset: true,
    outputType: preset.output_type,
  };
}
