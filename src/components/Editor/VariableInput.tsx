"use client";

import React from "react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

interface VariableInputProps {
  name: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  isAIFilled?: boolean;
  placeholder?: string;
}

export function VariableInput({
  name,
  label,
  value,
  onChange,
  isAIFilled = false,
  placeholder,
}: VariableInputProps) {
  return (
    <div className="space-y-1.5">
      <label
        htmlFor={name}
        className="text-xs font-medium text-muted-foreground flex items-center gap-1.5"
      >
        {label}
        {isAIFilled && (
          <span className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 font-normal">
            AI
          </span>
        )}
      </label>
      <Input
        id={name}
        name={name}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder || `Ingresa ${label.toLowerCase()}`}
        className={cn(
          "h-9 text-sm",
          isAIFilled && "bg-amber-50 dark:bg-amber-900/20 border-amber-300 dark:border-amber-700"
        )}
      />
    </div>
  );
}