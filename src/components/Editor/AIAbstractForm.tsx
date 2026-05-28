"use client";

import React, { useState, useEffect } from "react";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { VariableInput } from "./VariableInput";
import { Skeleton } from "@/components/ui/skeleton";
import { extractEntities, type ExtractedEntity } from "@/lib/aiService";

interface AIAbstractFormProps {
  templateId: number | null;
  requiredVariables: string[];
  onApplyToDocument: (values: Record<string, string>) => void;
  onLoadingChange?: (loading: boolean) => void;
  freeText: string;
  onFreeTextChange: (text: string) => void;
}

export function AIAbstractForm({
  templateId,
  requiredVariables,
  onApplyToDocument,
  onLoadingChange,
  freeText,
  onFreeTextChange,
}: AIAbstractFormProps) {
  const [fieldValues, setFieldValues] = useState<Record<string, string>>({});
  const [aiFilledFields, setAIFilledFields] = useState<Set<string>>(new Set());
  const [isProcessing, setIsProcessing] = useState(false);
  const [hasProcessed, setHasProcessed] = useState(false);

  // Reset when template changes
  useEffect(() => {
    setFieldValues({});
    setAIFilledFields(new Set());
    setHasProcessed(false);
  }, [templateId]);

  // Notify parent of loading state
  useEffect(() => {
    onLoadingChange?.(isProcessing);
  }, [isProcessing, onLoadingChange]);

  const handleProcessWithAI = async () => {
    if (!freeText.trim() || requiredVariables.length === 0) return;

    setIsProcessing(true);
    setHasProcessed(true);

    try {
      const entities = await extractEntities(freeText, requiredVariables);

      // Update field values with extracted entities
      const newFieldValues = { ...fieldValues };
      const newAIFilledFields = new Set<string>();

      entities.forEach((entity) => {
        if (entity.value) {
          newFieldValues[entity.variable] = entity.value;
          newAIFilledFields.add(entity.variable);
        }
      });

      setFieldValues(newFieldValues);
      setAIFilledFields(newAIFilledFields);
    } catch (error) {
      console.error("Error extracting entities:", error);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleFieldChange = (variable: string, value: string) => {
    setFieldValues((prev) => ({ ...prev, [variable]: value }));
    // Remove AI badge if user manually edits
    if (aiFilledFields.has(variable)) {
      setAIFilledFields((prev) => {
        const next = new Set(prev);
        next.delete(variable);
        return next;
      });
    }
  };

  const handleApplyToDocument = () => {
    onApplyToDocument(fieldValues);
  };

  const allFieldsFilled = requiredVariables.every(
    (v) => fieldValues[v]?.trim()
  );

  if (!templateId) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center p-6">
        <div className="w-12 h-12 rounded-full bg-muted flex items-center justify-center mb-4">
          <svg
            className="w-6 h-6 text-muted-foreground"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={1.5}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 0 0 2.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 0 0-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75 2.25 2.25 0 0 0-.1-.664m-5.801 0A2.251 2.251 0 0 1 13.5 2.25H15c1.012 0 1.87.688 2.006 1.699A2.25 2.25 0 0 0 18 4.5v2.25m-7.5 0v7.5m-7.5 0v-7.5m-7.5 0h7.5c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125h-7.5Z"
            />
          </svg>
        </div>
        <p className="text-sm text-muted-foreground">
          Selecciona una plantilla para cargar los campos del formulario
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Free Text Input Section */}
      <div className="p-4 space-y-3">
        <div>
          <label className="text-sm font-medium text-foreground mb-2 block">
            Describe la situación
          </label>
          <p className="text-xs text-muted-foreground mb-3">
            Escribe en lenguaje natural y la IA extraerá los datos automáticamente
          </p>
          <Textarea
            value={freeText}
            onChange={(e) => onFreeTextChange(e.target.value)}
            placeholder="Ej: Págale 1500 soles a Juan Pérez este fin de mes por el servicio de consultoría"
            className="min-h-[120px] resize-none"
          />
        </div>
        <Button
          onClick={handleProcessWithAI}
          disabled={!freeText.trim() || requiredVariables.length === 0}
          className="w-full"
          size="sm"
        >
          {isProcessing ? (
            <>
              <svg
                className="animate-spin -ml-1 mr-2 h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  className="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="4"
                />
                <path
                  className="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
              </svg>
              Procesando...
            </>
          ) : (
            <>
              <svg
                className="w-4 h-4 mr-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M9.813 15.904 9 18.75l-.813-2.846a4.5 4.5 0 0 0-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 0 0 3.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 0 0 3.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 0 0-3.09 3.09ZM18.259 8.715 18 9.75l-.259-1.035a3.375 3.375 0 0 0-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 0 0 2.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 0 0 2.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 0 0-2.456 2.456Z"
                />
              </svg>
              Procesar con IA
            </>
          )}
        </Button>
      </div>

      <Separator />

      {/* Structured Fields Section */}
      <div className="flex-1 overflow-hidden flex flex-col">
        <div className="px-4 py-3">
          <h3 className="text-sm font-medium">Campos del documento</h3>
          <p className="text-xs text-muted-foreground">
            {requiredVariables.length} variables a completar
          </p>
        </div>

        <ScrollArea className="flex-1 px-4">
          <div className="space-y-4 pb-4">
            {isProcessing ? (
              // Show skeleton while processing
              <>
                {requiredVariables.map((variable) => (
                  <div key={variable} className="space-y-1.5">
                    <Skeleton className="h-3 w-24" />
                    <Skeleton className="h-9 w-full" />
                  </div>
                ))}
              </>
            ) : (
              // Show actual inputs
              requiredVariables.map((variable) => (
                <VariableInput
                  key={variable}
                  name={variable}
                  label={formatVariableLabel(variable)}
                  value={fieldValues[variable] || ""}
                  onChange={(value) => handleFieldChange(variable, value)}
                  isAIFilled={aiFilledFields.has(variable)}
                  placeholder={getPlaceholderForVariable(variable)}
                />
              ))
            )}
          </div>
        </ScrollArea>
      </div>

      {/* Apply Button */}
      <div className="p-4 border-t">
        <Button
          onClick={handleApplyToDocument}
          disabled={!allFieldsFilled && !hasProcessed}
          className="w-full"
          variant={allFieldsFilled ? "default" : "outline"}
        >
          <svg
            className="w-4 h-4 mr-2"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 0 1 2.25-2.25h13.5A2.25 2.25 0 0 1 21 7.5v11.25m-18 0A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75m-18 0v-7.5A2.25 2.25 0 0 1 5.25 9h13.5A2.25 2.25 0 0 1 21 11.25v7.5"
            />
          </svg>
          Aplicar al Documento
        </Button>
        {!allFieldsFilled && hasProcessed && (
          <p className="text-xs text-muted-foreground text-center mt-2">
            Completa los campos vacíos manualmente
          </p>
        )}
      </div>
    </div>
  );
}

// Helper to format variable names into readable labels
function formatVariableLabel(variable: string): string {
  return variable
    .replace(/_/g, " ")
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

// Helper to get contextual placeholder based on variable name
function getPlaceholderForVariable(variable: string): string {
  const normalized = variable.toLowerCase();
  if (normalized.includes("monto") || normalized.includes("precio") || normalized.includes("cantidad")) {
    return "Ej: 1500 soles";
  }
  if (normalized.includes("fecha")) {
    return "Ej: 15 de junio de 2026";
  }
  if (normalized.includes("nombre") && normalized.includes("cliente")) {
    return "Ej: Juan Pérez";
  }
  if (normalized.includes("nombre") && normalized.includes("proveedor")) {
    return "Ej: Empresa ABC";
  }
  if (normalized.includes("empresa")) {
    return "Ej: TechCorp S.A.";
  }
  if (normalized.includes("email")) {
    return "Ej: juan@email.com";
  }
  if (normalized.includes("telefono")) {
    return "Ej: +51 999 123 456";
  }
  if (normalized.includes("direccion")) {
    return "Ej: Av. Principal 123, Lima";
  }
  if (normalized.includes("descripcion") || normalized.includes("detalle")) {
    return "Ej: Servicio de consultoría mensual";
  }
  if (normalized.includes("servicio")) {
    return "Ej: Desarrollo de software";
  }
  return `Ingresa ${variable.replace(/_/g, " ")}`;
}