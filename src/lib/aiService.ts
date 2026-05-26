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

// ============================================================================
// Entity Extraction for Document Templates
// ============================================================================

export interface ExtractedEntity {
  variable: string;
  value: string;
  confidence: number; // 0-1
}

interface ExtractionRule {
  patterns: RegExp[];
  variable: string;
  transform?: (match: string) => string;
}

/**
 * Extract entities from free text based on required variables
 * Uses pattern matching with confidence scores
 */
export async function extractEntities(
  text: string,
  requiredVariables: string[]
): Promise<ExtractedEntity[]> {
  // Try Tauri backend first
  try {
    const entities = await invoke<ExtractedEntity[]>("extract_entities", {
      text,
      variables: requiredVariables,
    });
    return entities;
  } catch {
    // Fall back to mock implementation
    return mockExtractEntities(text, requiredVariables);
  }
}

/**
 * Mock implementation for entity extraction
 * Demonstrates the feature when Tauri backend is not available
 */
function mockExtractEntities(
  text: string,
  requiredVariables: string[]
): ExtractedEntity[] {
  const entities: ExtractedEntity[] = [];
  const normalizedText = text.toLowerCase();

  // Extraction rules for common patterns
  const rules: ExtractionRule[] = [
    // Money patterns
    {
      patterns: [
        /(\d+(?:\.\d{3})*(?:\s*(?:soles|dolares|euros|USD|EUR))?)/i,
        /(?:monto|cantidad|importe|precio)[:\s]*(\d+(?:\.\d{3})*)/i,
      ],
      variable: "monto",
      transform: (match) => match.replace(/[^\d.]/g, "") + " soles",
    },
    // Beneficiary patterns
    {
      patterns: [
        /(?:a|en beneficio de|para)[\s]+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)/i,
        /(?:beneficiario|pagalo a|pagar a)[\s]+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)/i,
      ],
      variable: "beneficiario",
    },
    // Date patterns
    {
      patterns: [
        /(?:este|la)?[\s]*(fin de mes|mes|semana|año)/i,
        /(?:para|el|fecha)[:\s]*(\d{1,2}[\s/-]\d{1,2}[\s/-]\d{2,4})/i,
        /(?:fecha|cuando)[:\s]*([A-Z][a-z]+(?:\s+\d{1,2})?)/i,
      ],
      variable: "fecha",
      transform: (match) => calculateDate(match),
    },
    // Company patterns
    {
      patterns: [
        /(?:empresa|compañia|sociedad)[\s]+([A-Z][A-Za-z]+(?:\s+[A-Z][A-Za-z]+)*(?:\s+S\.?A\.?|S\.?R\.?L\.?|E\.?I\.?R\.?L\.?)?)/i,
      ],
      variable: "empresa",
    },
    // Service patterns
    {
      patterns: [
        /(?:servicio de|por|concepto)[:\s]*([A-Z][a-z]+(?:\s+[a-z]+)*)/i,
      ],
      variable: "servicio",
    },
    // Description patterns
    {
      patterns: [
        /(?:descripcion|detalle|observacion)[:\s]*(.+?)(?:\.|$)/i,
      ],
      variable: "descripcion",
    },
  ];

  // Process each required variable
  for (const variable of requiredVariables) {
    const normalizedVar = variable.toLowerCase();
    const matchingRule = rules.find((rule) =>
      rule.variable.toLowerCase() === normalizedVar ||
      normalizedVar.includes(rule.variable.toLowerCase())
    );

    if (matchingRule) {
      for (const pattern of matchingRule.patterns) {
        const match = text.match(pattern);
        if (match && match[1]) {
          let value = match[1].trim();
          if (matchingRule.transform) {
            value = matchingRule.transform(value);
          }
          entities.push({
            variable,
            value,
            confidence: 0.85,
          });
          break;
        }
      }
    }

    // Try fuzzy matching for name variables
    if (!entities.find((e) => e.variable === variable)) {
      if (normalizedVar.includes("nombre")) {
        const nameMatch = text.match(/([A-Z][a-z]+(?:\s+[A-Z][a-z]+){1,2})/);
        if (nameMatch) {
          entities.push({
            variable,
            value: nameMatch[1],
            confidence: 0.8,
          });
        }
      }
    }
  }

  return entities;
}

/**
 * Calculate date from relative expressions
 */
function calculateDate(relativeDate: string): string {
  const now = new Date();
  const normalized = relativeDate.toLowerCase();

  if (normalized.includes("fin de mes")) {
    const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);
    return lastDay.toLocaleDateString("es-ES", {
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  }

  if (normalized.includes("semana")) {
    const nextWeek = new Date(now);
    nextWeek.setDate(now.getDate() + 7);
    return nextWeek.toLocaleDateString("es-ES", {
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  }

  if (normalized.includes("mes")) {
    return now.toLocaleDateString("es-ES", {
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  }

  return relativeDate;
}