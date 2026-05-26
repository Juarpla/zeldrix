"use client";

import React from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

interface TemplateCardProps {
  id: number;
  name: string;
  category: string;
  required_variables: string[];
  onClick: () => void;
}

const CATEGORY_STYLES: Record<string, { gradient: string; border: string; badge: "legal" | "ventas" | "recursos_humanos" }> = {
  "legal": {
    gradient: "from-blue-900/30 to-blue-800/20",
    border: "hover:border-blue-500/60",
    badge: "legal",
  },
  "ventas": {
    gradient: "from-emerald-900/30 to-emerald-800/20",
    border: "hover:border-emerald-500/60",
    badge: "ventas",
  },
  "recursos humanos": {
    gradient: "from-violet-900/30 to-violet-800/20",
    border: "hover:border-violet-500/60",
    badge: "recursos_humanos",
  },
};

function getCategoryStyle(category: string) {
  const normalized = category.toLowerCase();
  return CATEGORY_STYLES[normalized] || {
    gradient: "from-gray-900/30 to-gray-800/20",
    border: "hover:border-gray-500/60",
    badge: "default" as const,
  };
}

export function TemplateCard({ id, name, category, required_variables, onClick }: TemplateCardProps) {
  const style = getCategoryStyle(category);

  return (
    <Card
      className={cn(
        "cursor-pointer transition-all duration-200 hover:scale-[1.02] hover:shadow-lg border-muted-foreground/20",
        style.border
      )}
      onClick={onClick}
    >
      <div className={cn("h-2 rounded-t-xl bg-gradient-to-r", style.gradient)} />
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-2">
          <CardTitle className="text-lg font-semibold leading-tight line-clamp-2">
            {name}
          </CardTitle>
          <Badge variant={style.badge} className="shrink-0 capitalize">
            {category}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          <p className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
            Variables a completar
          </p>
          <div className="flex flex-wrap gap-1.5">
            {required_variables.slice(0, 4).map((variable) => (
              <span
                key={variable}
                className="inline-flex items-center px-2 py-0.5 rounded text-xs font-mono bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 border border-amber-300 dark:border-amber-700"
              >
                {variable}
              </span>
            ))}
            {required_variables.length > 4 && (
              <span className="inline-flex items-center px-2 py-0.5 rounded text-xs bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400">
                +{required_variables.length - 4} más
              </span>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}