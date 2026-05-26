"use client";

import TypographyEditor from "@/components/Editor/TypographyEditor";
import { useState } from "react";

export default function EditorPage() {
  const [content, setContent] = useState("");

  return (
    <div className="min-h-screen bg-gray-50 py-12 px-4">
      <div className="max-w-3xl mx-auto">
        <header className="mb-8">
          <h1 className="text-3xl font-semibold text-gray-900 mb-2">
            Editor Tipográfico
          </h1>
          <p className="text-gray-500">
            Escribe de forma fluida. Los comandos Markdown se transforman
            instantáneamente.
          </p>
        </header>

        <div className="mb-6 p-4 bg-blue-50 rounded-lg border border-blue-100">
          <h2 className="text-sm font-medium text-blue-900 mb-2">
            Atajos de teclado
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-3 gap-2 text-xs text-blue-700">
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono"># </kbd>{" "}
              Título 1
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">## </kbd>{" "}
              Título 2
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">### </kbd>{" "}
              Título 3
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">- </kbd>{" "}
              Lista
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">**</kbd>{" "}
              Negrita
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">*</kbd>{" "}
              Cursiva
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">`</kbd>{" "}
              Código
            </span>
            <span>
              <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">---</kbd>{" "}
              Línea
            </span>
          </div>
        </div>

        <TypographyEditor
          content={content}
          onChange={setContent}
          placeholder="Empieza a escribir..."
        />

        {content && (
          <div className="mt-8 p-4 bg-gray-100 rounded-lg">
            <h3 className="text-sm font-medium text-gray-700 mb-2">
              HTML generado:
            </h3>
            <pre className="text-xs text-gray-600 overflow-x-auto whitespace-pre-wrap">
              {content}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}