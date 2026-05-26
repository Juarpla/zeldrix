"use client";

import React, { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { TemplateSkeleton } from "./TemplateSkeleton";
import { VariableHighlight } from "./VariableHighlight";
import { getTemplateById } from "@/lib/templates-service";
import type { Template } from "@/lib/types";
import { extractVariableNames } from "./VariableHighlight";

interface TemplateModalProps {
  templateId: number | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function TemplateModal({ templateId, open, onOpenChange }: TemplateModalProps) {
  const [template, setTemplate] = useState<Template | null>(null);
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  useEffect(() => {
    if (open && templateId !== null) {
      setLoading(true);
      setTemplate(null);
      getTemplateById(templateId)
        .then((t) => {
          setTemplate(t);
          setLoading(false);
        })
        .catch(() => {
          setLoading(false);
        });
    }
  }, [open, templateId]);

  const handleUseTemplate = () => {
    if (template) {
      onOpenChange(false);
      router.push(`/editor?template=${template.id}`);
    }
  };

  const variables = template ? extractVariableNames(template.base_text) : [];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        {loading || !template ? (
          <>
            <DialogHeader>
              <div className="flex items-center gap-3">
                <div className="h-6 w-48 bg-muted animate-pulse rounded" />
                <div className="h-5 w-20 bg-muted animate-pulse rounded-full" />
              </div>
            </DialogHeader>
            <Separator className="my-4" />
            <TemplateSkeleton />
          </>
        ) : (
          <>
            <DialogHeader>
              <div className="flex items-center gap-3">
                <DialogTitle className="text-xl">{template.name}</DialogTitle>
                <Badge variant={template.category.toLowerCase().replace(" ", "_") as "legal" | "ventas" | "recursos_humanos"} className="capitalize">
                  {template.category}
                </Badge>
              </div>
              <DialogDescription className="text-sm mt-1">
                Preview de la estructura del documento con las variables que la IA completará
              </DialogDescription>
            </DialogHeader>

            <Separator className="my-4" />

            {/* Variables summary */}
            <div className="mb-4">
              <p className="text-xs text-muted-foreground font-medium uppercase tracking-wide mb-2">
                Variables a completar ({variables.length})
              </p>
              <div className="flex flex-wrap gap-2">
                {variables.map((v) => (
                  <span
                    key={v}
                    className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs font-mono bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 border border-amber-400 dark:border-amber-600"
                  >
                    <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 0 0 .238-.912c.07-.23.135-.453.18-.677l.076-.076c-.11-.147-.18-.302-.18-.493 0-.189.068-.367.194-.5a1.41 1.41 0 0 1 .507-.122l1.216.485a1.127 1.127 0 0 1 1.414.78l1.308 2.305a1.125 1.125 0 0 1-.177 1.412l-1.003.827a1.125 1.125 0 0 1-.432.5c-.191.15-.417.224-.654.224-.235 0-.466-.075-.667-.224l-1.308-2.305a1.125 1.125 0 0 1-.26-1.43l1.003-.827c.284-.24.44-.607.44-.992a.93.93 0 0 0-.237-.945l-1.216-.485a1.127 1.127 0 0 1-.507-.122 1.412 1.412 0 0 1-.194-.5c.126-.133.194-.311.194-.5 0-.19-.07-.346-.18-.493l-.076-.076c.11-.224.18-.447.18-.677a1.2 1.2 0 0 0-.18-.677l-.076-.076c-.054-.112-.14-.212-.22-.127-.332-.184-.582-.496-.645-.87l-.213-1.281Z" />
                      <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" />
                    </svg>
                    {v}
                  </span>
                ))}
              </div>
            </div>

            {/* Document preview */}
            <div className="bg-muted/30 dark:bg-muted/10 rounded-lg p-4 border border-border">
              <p className="text-xs text-muted-foreground font-medium uppercase tracking-wide mb-3">
                Estructura del documento
              </p>
              <VariableHighlight text={template.base_text} />
            </div>

            <DialogFooter className="gap-2 mt-6">
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cerrar
              </Button>
              <Button onClick={handleUseTemplate}>
                Usar Plantilla
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}