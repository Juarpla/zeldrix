"use client";

import React from "react";

interface VariableHighlightProps {
  text: string;
}

function extractVariables(text: string): Array<{ type: "text" | "variable"; content: string }> {
  const parts: Array<{ type: "text" | "variable"; content: string }> = [];
  const regex = /\{\{([^}]+)\}\}/g;
  let lastIndex = 0;
  let match;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push({ type: "text", content: text.slice(lastIndex, match.index) });
    }
    parts.push({ type: "variable", content: match[1] });
    lastIndex = regex.lastIndex;
  }

  if (lastIndex < text.length) {
    parts.push({ type: "text", content: text.slice(lastIndex) });
  }

  return parts;
}

export function VariableHighlight({ text }: VariableHighlightProps) {
  const parts = extractVariables(text);

  return (
    <div className="whitespace-pre-wrap font-mono text-sm leading-relaxed">
      {parts.map((part, index) =>
        part.type === "variable" ? (
          <span
            key={index}
            className="inline-flex items-center gap-1 px-1.5 py-0.5 mx-0.5 rounded bg-amber-100 dark:bg-amber-900/30 border border-amber-400 dark:border-amber-600 text-amber-700 dark:text-amber-400 font-semibold"
            title="Variable que la IA rellenará"
          >
            <svg
              className="w-3 h-3"
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
            {`{{${part.content}}}`}
          </span>
        ) : (
          <span key={index} className="text-gray-800 dark:text-gray-200">
            {part.content}
          </span>
        )
      )}
    </div>
  );
}

export function extractVariableNames(text: string): string[] {
  const regex = /\{\{([^}]+)\}\}/g;
  const variables: string[] = [];
  let match;

  while ((match = regex.exec(text)) !== null) {
    if (!variables.includes(match[1])) {
      variables.push(match[1]);
    }
  }

  return variables;
}