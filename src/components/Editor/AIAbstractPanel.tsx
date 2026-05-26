"use client";

import React from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AIAbstractForm } from "./AIAbstractForm";

interface AIAbstractPanelProps {
  templateId: number | null;
  templateName?: string;
  requiredVariables: string[];
  onApplyToDocument: (values: Record<string, string>) => void;
  onLoadingChange?: (loading: boolean) => void;
}

export function AIAbstractPanel({
  templateId,
  templateName,
  requiredVariables,
  onApplyToDocument,
  onLoadingChange,
}: AIAbstractPanelProps) {
  return (
    <div className="flex flex-col h-full bg-background border-r">
      {/* Header */}
      <div className="px-4 py-3 border-b bg-muted/30">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-amber-500 to-orange-600 flex items-center justify-center">
            <svg
              className="w-4 h-4 text-white"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M9.813 15.904 9 18.75l-.813-2.846a4.5 4.5 0 0 0-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 0 0 3.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 0 0 3.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 0 0-3.09 3.09Z"
              />
            </svg>
          </div>
          <div className="flex-1 min-w-0">
            <h2 className="text-sm font-semibold truncate">
              Asistente de IA
            </h2>
            {templateName && (
              <p className="text-xs text-muted-foreground truncate">
                {templateName}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex-1 overflow-hidden flex flex-col">
        <Tabs defaultValue="abstract" className="flex-1 flex flex-col px-4 pt-4">
          <TabsList className="w-full">
            <TabsTrigger value="abstract" className="flex-1 text-xs">
              <svg
                className="w-4 h-4 mr-1.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 0 0 .238-.912c.07-.23.135-.453.18-.677l.076-.076c-.11-.147-.18-.302-.18-.493 0-.189.068-.367.194-.5a1.41 1.41 0 0 1 .507-.122l1.216.485a1.127 1.127 0 0 1 1.414.78l1.308 2.305a1.125 1.125 0 0 1-.177 1.412l-1.003.827a1.125 1.125 0 0 1-.432.5c-.191.15-.417.224-.654.224-.235 0-.466-.075-.667-.224l-1.308-2.305a1.125 1.125 0 0 1-.26-1.43l1.003-.827c.284-.24.44-.607.44-.992a.93.93 0 0 0-.237-.945l-1.216-.485a1.127 1.127 0 0 1-.507-.122 1.412 1.412 0 0 1-.194-.5c.126-.133.194-.311.194-.5 0-.19-.07-.346-.18-.493l-.076-.076c.11-.224.18-.447.18-.677a1.2 1.2 0 0 0-.18-.677l-.076-.076c-.054-.112-.14-.212-.22-.127-.332-.184-.582-.496-.645-.87l-.213-1.281Z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                />
              </svg>
              Texto Libre
            </TabsTrigger>
            <TabsTrigger value="fields" className="flex-1 text-xs">
              <svg
                className="w-4 h-4 mr-1.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M8.25 6.75h12M8.25 12h12m-12 5.25h12M3.75 6.75h.007v.008H3.75V6.75Zm.375 0h.007v.008H3.75V6.75Z"
                />
              </svg>
              Campos
            </TabsTrigger>
          </TabsList>

          <TabsContent value="abstract" className="flex-1 mt-0 -mx-4 overflow-hidden">
            <AIAbstractForm
              templateId={templateId}
              requiredVariables={requiredVariables}
              onApplyToDocument={onApplyToDocument}
              onLoadingChange={onLoadingChange}
            />
          </TabsContent>

          <TabsContent value="fields" className="flex-1 mt-0 -mx-4 overflow-hidden">
            <AIAbstractForm
              templateId={templateId}
              requiredVariables={requiredVariables}
              onApplyToDocument={onApplyToDocument}
              onLoadingChange={onLoadingChange}
            />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}