"use client";

import React, { useEffect, useState } from "react";
import { TemplateCard } from "./TemplateCard";
import { TemplateModal } from "./TemplateModal";
import { getTemplates } from "@/lib/templates-service";
import type { Template } from "@/lib/types";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";

interface CategorySection {
  category: string;
  templates: Template[];
}

const CATEGORY_ORDER = ["legal", "ventas", "recursos humanos"];

const CATEGORY_LABELS: Record<string, string> = {
  "legal": "Legal",
  "ventas": "Ventas",
  "recursos humanos": "Recursos Humanos",
};

const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  "legal": (
    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 3v17.25m0 0c-1.472 0-2.882.265-4.185.75M12 20.25c1.472 0 2.882.265 4.185.75M18.75 4.97A48.416 48.416 0 0 0 12 4.5c-2.291 0-4.545.16-6.75.47m13.5 0c1.01.143 2.01.317 3 .52m-3-.52 2.62 10.726c.122.499-.106 1.028-.589 1.202a5.988 5.988 0 0 1-2.031.352 5.988 5.988 0 0 1-2.031-.352c-.483-.174-.711-.703-.59-1.202L18.75 4.97Zm-12.5 0 2.62 10.726c.122.499-.106 1.028-.589 1.202a5.989 5.989 0 0 1-2.031.352 5.989 5.989 0 0 1-2.031-.352c-.483-.174-.711-.703-.59-1.202L5.25 4.97Z" />
    </svg>
  ),
  "ventas": (
    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 0 1 3 19.875v-6.75ZM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V8.625ZM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V4.125Z" />
    </svg>
  ),
  "recursos humanos": (
    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
      <path strokeLinecap="round" strokeLinejoin="round" d="M15 19.128a9.38 9.38 0 0 0 2.625.372 9.337 9.337 0 0 0 4.121-.952 4.125 4.125 0 0 0-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 0 1 8.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0 1 11.964-3.07M12 6.375a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0Zm8.25 2.25a2.625 2.625 0 1 1-5.25 0 2.625 2.625 0 0 1 5.25 0Z" />
    </svg>
  ),
};

export function TemplateCatalog() {
  const [templates, setTemplates] = useState<Template[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTemplateId, setSelectedTemplateId] = useState<number | null>(null);
  const [modalOpen, setModalOpen] = useState(false);

  useEffect(() => {
    getTemplates()
      .then((data) => {
        setTemplates(data);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  // Group templates by category
  const groupedTemplates = templates.reduce<CategorySection[]>((acc, template) => {
    const existing = acc.find((g) => g.category.toLowerCase() === template.category.toLowerCase());
    if (existing) {
      existing.templates.push(template);
    } else {
      acc.push({ category: template.category, templates: [template] });
    }
    return acc;
  }, []);

  // Sort by category order
  groupedTemplates.sort((a, b) => {
    const aIndex = CATEGORY_ORDER.indexOf(a.category.toLowerCase());
    const bIndex = CATEGORY_ORDER.indexOf(b.category.toLowerCase());
    if (aIndex === -1 && bIndex === -1) return a.category.localeCompare(b.category);
    if (aIndex === -1) return 1;
    if (bIndex === -1) return -1;
    return aIndex - bIndex;
  });

  const handleCardClick = (templateId: number) => {
    setSelectedTemplateId(templateId);
    setModalOpen(true);
  };

  if (loading) {
    return (
      <div className="space-y-8">
        <div className="space-y-2">
          <Skeleton className="h-8 w-64" />
          <Skeleton className="h-4 w-96" />
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {[1, 2, 3].map((i) => (
            <div key={i} className="space-y-3">
              <Skeleton className="h-4 w-32" />
              <div className="border rounded-xl p-4 space-y-3">
                <Skeleton className="h-6 w-3/4" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-2/3" />
              </div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-10">
      <div>
        <h1 className="text-3xl font-bold tracking-tight mb-2">Catálogo de Plantillas</h1>
        <p className="text-muted-foreground">
          Selecciona una plantilla corporativa para generar documentos con IA
        </p>
      </div>

      {groupedTemplates.map((group) => (
        <section key={group.category} className="space-y-4">
          <div className="flex items-center gap-3">
            <span className="text-muted-foreground">
              {CATEGORY_ICONS[group.category.toLowerCase()]}
            </span>
            <h2 className="text-xl font-semibold tracking-wide">
              {CATEGORY_LABELS[group.category.toLowerCase()] || group.category}
            </h2>
            <span className="text-sm text-muted-foreground">
              ({group.templates.length} plantilla{group.templates.length !== 1 ? "s" : ""})
            </span>
          </div>
          <Separator />
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {group.templates.map((template) => (
              <TemplateCard
                key={template.id}
                id={template.id}
                name={template.name}
                category={template.category}
                required_variables={template.required_variables}
                onClick={() => handleCardClick(template.id)}
              />
            ))}
          </div>
        </section>
      ))}

      <TemplateModal
        templateId={selectedTemplateId}
        open={modalOpen}
        onOpenChange={setModalOpen}
      />
    </div>
  );
}