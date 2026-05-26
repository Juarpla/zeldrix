// AI Service - Frontend wrapper for AI text transformations
// Calls the Tauri backend command ai_transform_text

import { invoke } from "@tauri-apps/api/core";

export type AIActionType = "formal" | "style" | "translate" | "summarize";

export interface AITransformResult {
  success: boolean;
  result?: string;
  error?: string;
}

/**
 * Transform text using AI based on the specified action
 * @param action - The type of transformation (formal, style, translate, summarize)
 * @param text - The text to transform
 * @returns The transformed text or an error message
 */
export async function transformText(
  action: AIActionType,
  text: string
): Promise<AITransformResult> {
  try {
    const result = await invoke<string>("ai_transform_text", {
      action,
      text,
    });

    return {
      success: true,
      result,
    };
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : String(error);

    // Provide user-friendly error messages
    let userMessage = errorMessage;
    if (errorMessage.includes("Sidecar not running")) {
      userMessage =
        "El servicio de IA no está disponible. Asegúrate de que el modelo esté cargado.";
    } else if (errorMessage.includes("not responding")) {
      userMessage =
        "El servicio de IA no está respondiendo. Espera unos segundos y prueba de nuevo.";
    } else if (errorMessage.includes("timeout")) {
      userMessage =
        "La solicitud tardó demasiado. Intenta con un texto más corto.";
    }

    return {
      success: false,
      error: userMessage,
    };
  }
}

/**
 * Check if the AI service (sidecar) is healthy and ready
 */
export async function isAIServiceReady(): Promise<boolean> {
  try {
    const isHealthy = await invoke<boolean>("sidecar_health");
    return isHealthy;
  } catch {
    return false;
  }
}

/**
 * Get the current sidecar status
 */
export async function getSidecarStatus(): Promise<{
  running: boolean;
  port?: number;
  model?: string;
}> {
  try {
    const status = await invoke<{
      running: boolean;
      port: number | null;
      model: string | null;
    }>("sidecar_status");

    return {
      running: status.running,
      port: status.port ?? undefined,
      model: status.model ?? undefined,
    };
  } catch {
    return { running: false };
  }
}