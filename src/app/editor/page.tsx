"use client";

import { Suspense, useState, useEffect, useCallback, useMemo } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import TypographyEditor from "@/components/Editor/TypographyEditor";
import { AIAbstractPanel } from "@/components/Editor/AIAbstractPanel";
import { VersionHistoryPanel } from "@/components/VersionHistory/VersionHistoryPanel";
import { getTemplateById, mergeTemplate } from "@/lib/templates-service";
import { exportDocumentAsPdf } from "@/lib/export-service";
import {
  listDocumentVersions,
  saveDocumentVersion,
} from "@/lib/version-history-service";
import type { DocumentVersion, Template } from "@/lib/types";

function EditorContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const templateIdParam = searchParams.get("template");

  const [templateId, setTemplateId] = useState<number | null>(
    templateIdParam ? parseInt(templateIdParam, 10) : null
  );
  const [template, setTemplate] = useState<Template | null>(null);
  const [content, setContent] = useState("");
  const [isLoadingTemplate, setIsLoadingTemplate] = useState(false);
  const [isAIPanelLoading, setIsAIPanelLoading] = useState(false);
  const [mergeError, setMergeError] = useState<string | null>(null);
  const [mergeSuccess, setMergeSuccess] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exportPath, setExportPath] = useState<string | null>(null);
  const [versions, setVersions] = useState<DocumentVersion[]>([]);
  const [selectedVersionId, setSelectedVersionId] = useState<string | null>(null);
  const [isHistoryLoading, setIsHistoryLoading] = useState(false);
  const [isRevertingVersion, setIsRevertingVersion] = useState(false);
  const [historyError, setHistoryError] = useState<string | null>(null);

  const documentId = useMemo(
    () => (templateId ? `template-${templateId}` : "freeform-editor"),
    [templateId]
  );

  const refreshHistory = useCallback(async () => {
    setIsHistoryLoading(true);
    setHistoryError(null);

    try {
      const nextVersions = await listDocumentVersions(documentId);
      setVersions(nextVersions);
      setSelectedVersionId((current) => {
        if (current && nextVersions.some((version) => version.id === current)) {
          return current;
        }
        return nextVersions[0]?.id ?? null;
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "No se pudo cargar el historial.";
      setHistoryError(String(message));
    } finally {
      setIsHistoryLoading(false);
    }
  }, [documentId]);

  const recordAIChange = useCallback(
    async ({
      actionLabel,
      previousContent,
      newContent,
    }: {
      actionLabel: string;
      previousContent: string;
      newContent: string;
    }) => {
      if (previousContent === newContent) return;

      setHistoryError(null);
      try {
        const version = await saveDocumentVersion({
          document_id: documentId,
          action_label: actionLabel,
          previous_content: previousContent,
          new_content: newContent,
        });
        await refreshHistory();
        setSelectedVersionId(version.id);
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "No se pudo guardar el historial.";
        setHistoryError(String(message));
      }
    },
    [documentId, refreshHistory]
  );

  const handleRevertVersion = useCallback(
    async (version: DocumentVersion) => {
      setIsRevertingVersion(true);
      setHistoryError(null);

      const currentContent = content;
      setContent(version.previous_content);

      try {
        const revertVersion = await saveDocumentVersion({
          document_id: documentId,
          action_label: `Reversion: ${version.action_label}`,
          previous_content: currentContent,
          new_content: version.previous_content,
        });
        await refreshHistory();
        setSelectedVersionId(revertVersion.id);
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "No se pudo registrar la reversion.";
        setHistoryError(String(message));
      } finally {
        setIsRevertingVersion(false);
      }
    },
    [content, documentId, refreshHistory]
  );

  // Load template if ID is provided
  useEffect(() => {
    if (templateId) {
      setIsLoadingTemplate(true);
      getTemplateById(templateId)
        .then((t) => {
          setTemplate(t);
          if (t) {
            setContent(t.base_text);
          }
          setIsLoadingTemplate(false);
        })
        .catch(() => {
          setIsLoadingTemplate(false);
        });
    } else {
      setTemplate(null);
      setContent("");
    }
  }, [templateId]);

  useEffect(() => {
    if (typeof window !== "undefined") {
      const importedText = sessionStorage.getItem("zeldrix_import_editor_text");
      if (importedText) {
        setContent(importedText);
        sessionStorage.removeItem("zeldrix_import_editor_text");
      }
    }
  }, []);

  useEffect(() => {
    void refreshHistory();
  }, [refreshHistory]);

  // Apply extracted values to document using Rust Merge Engine
  const handleApplyToDocument = useCallback(
    async (values: Record<string, string>) => {
      setMergeError(null);
      setMergeSuccess(false);
      try {
        const merged = await mergeTemplate(content, values);
        await recordAIChange({
          actionLabel: "IA: aplicar variables de plantilla",
          previousContent: content,
          newContent: merged,
        });
        setContent(merged);
        setMergeSuccess(true);
        // Clear success message after 4 seconds
        setTimeout(() => setMergeSuccess(false), 4000);
      } catch (err: any) {
        console.error("Merge engine failed:", err);
        setMergeError(err?.message || String(err));
      }
    },
    [content, recordAIChange]
  );

  // Extract required variables from template
  const requiredVariables = template?.required_variables || [];

  const handleExportPdf = useCallback(async () => {
    setIsExporting(true);
    setExportError(null);
    setExportPath(null);

    try {
      const result = await exportDocumentAsPdf(
        content,
        template?.name ?? "documento-zeldrix"
      );
      setExportPath(result.path);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "No se pudo exportar el documento.";
      setExportError(String(message));
    } finally {
      setIsExporting(false);
    }
  }, [content, template?.name]);

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Left Panel: AI Abstract Form */}
      <div className="w-[400px] shrink-0 border-r border-gray-200 bg-white overflow-hidden">
        <AIAbstractPanel
          templateId={templateId}
          templateName={template?.name}
          requiredVariables={requiredVariables}
          onApplyToDocument={handleApplyToDocument}
          onLoadingChange={setIsAIPanelLoading}
        />
      </div>

      {/* Right Panel: Document Editor */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Editor Header */}
        <div className="bg-white border-b border-gray-200 px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-semibold text-gray-900">
                Editor de Documentos
              </h1>
              {template ? (
                <p className="text-sm text-gray-500 mt-0.5">
                  Plantilla: <span className="font-medium">{template.name}</span>
                </p>
              ) : (
                <p className="text-sm text-gray-400 mt-0.5">
                  Sin plantilla - escribe libremente
                </p>
              )}
            </div>
            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={handleExportPdf}
                disabled={isExporting || isLoadingTemplate || content.trim().length === 0}
                className="inline-flex items-center gap-2 rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white shadow-sm transition-colors hover:bg-gray-800 disabled:cursor-not-allowed disabled:opacity-50"
                title="Guardar PDF en el escritorio"
              >
                {isExporting ? (
                  <svg
                    className="h-4 w-4 animate-spin"
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
                ) : (
                  <svg
                    className="h-4 w-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={2}
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      d="M19.5 14.25v3.375A2.625 2.625 0 0 1 16.875 20.25h-9.75A2.625 2.625 0 0 1 4.5 17.625V14.25m4.5-6 3-3m0 0 3 3m-3-3v10.5"
                    />
                  </svg>
                )}
                {isExporting ? "Exportando..." : "Exportar como PDF"}
              </button>
              {template && (
                <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-amber-50 border border-amber-200">
                  <span className="text-xs font-medium text-amber-700">
                    {requiredVariables.length} variables
                  </span>
                </div>
              )}
              {isAIPanelLoading && (
                <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-blue-50 border border-blue-200">
                  <svg
                    className="animate-spin h-3 w-3 text-blue-600"
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
                  <span className="text-xs font-medium text-blue-700">
                    Procesando IA
                  </span>
                </div>
              )}
            </div>
          </div>

          {/* Shortcuts hint */}
          <div className="mt-3 p-3 bg-blue-50 rounded-lg border border-blue-100">
            <h2 className="text-xs font-medium text-blue-900 mb-2">
              Atajos de teclado
            </h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs text-blue-700">
              <span>
                <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono"># </kbd>{" "}
                Título 1
              </span>
              <span>
                <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">## </kbd>{" "}
                Título 2
              </span>
              <span>
                <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">**</kbd>{" "}
                Negrita
              </span>
              <span>
                <kbd className="px-1.5 py-0.5 bg-blue-100 rounded font-mono">- </kbd>{" "}
                Lista
              </span>
            </div>
          </div>
        </div>

        {/* Editor Content */}
        <div className="flex-1 overflow-auto p-6">
          {mergeError && (
            <div className="max-w-3xl mx-auto mb-4 p-4 bg-rose-50 border border-rose-200 rounded-lg text-rose-800 text-sm flex items-center justify-between animate-in fade-in slide-in-from-top-2 duration-200">
              <div className="flex items-center gap-2">
                <svg className="w-5 h-5 text-rose-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <span><strong>Error de combinación:</strong> {mergeError}</span>
              </div>
              <button onClick={() => setMergeError(null)} className="text-rose-500 hover:text-rose-700 text-xs font-semibold">
                Descartar
              </button>
            </div>
          )}
          {mergeSuccess && (
            <div className="max-w-3xl mx-auto mb-4 p-4 bg-emerald-50 border border-emerald-200 rounded-lg text-emerald-800 text-sm flex items-center gap-2 animate-in fade-in slide-in-from-top-2 duration-200">
              <svg className="w-5 h-5 text-emerald-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span>¡Combinación exitosa! Todos los marcadores de posición fueron inyectados de forma segura mediante el motor de correspondencia.</span>
            </div>
          )}
          {exportPath && (
            <div className="max-w-3xl mx-auto mb-4 p-4 bg-emerald-50 border border-emerald-200 rounded-lg text-emerald-800 text-sm flex items-center justify-between gap-3 animate-in fade-in slide-in-from-top-2 duration-200">
              <div className="flex items-center gap-2 min-w-0">
                <svg className="w-5 h-5 text-emerald-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
                </svg>
                <span className="truncate">
                  PDF guardado en <span className="font-medium">{exportPath}</span>
                </span>
              </div>
              <button onClick={() => setExportPath(null)} className="text-emerald-600 hover:text-emerald-800 text-xs font-semibold">
                Descartar
              </button>
            </div>
          )}
          {exportError && (
            <div className="max-w-3xl mx-auto mb-4 p-4 bg-rose-50 border border-rose-200 rounded-lg text-rose-800 text-sm flex items-center justify-between gap-3 animate-in fade-in slide-in-from-top-2 duration-200">
              <div className="flex items-center gap-2">
                <svg className="w-5 h-5 text-rose-500 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m0 3.75h.008v.008H12V16.5Zm8.25-4.5a8.25 8.25 0 1 1-16.5 0 8.25 8.25 0 0 1 16.5 0Z" />
                </svg>
                <span><strong>Error de exportación:</strong> {exportError}</span>
              </div>
              <button onClick={() => setExportError(null)} className="text-rose-500 hover:text-rose-700 text-xs font-semibold">
                Descartar
              </button>
            </div>
          )}
          {historyError && (
            <div className="max-w-6xl mx-auto mb-4 p-4 bg-amber-50 border border-amber-200 rounded-lg text-amber-800 text-sm flex items-center justify-between gap-3 animate-in fade-in slide-in-from-top-2 duration-200">
              <span>
                <strong>Historial local:</strong> {historyError}
              </span>
              <button onClick={() => setHistoryError(null)} className="text-amber-600 hover:text-amber-800 text-xs font-semibold">
                Descartar
              </button>
            </div>
          )}
          <div className="mx-auto grid max-w-6xl gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
            <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6 min-h-[600px]">
              {isLoadingTemplate ? (
                <div className="flex items-center justify-center h-64">
                  <div className="flex items-center gap-3">
                    <svg
                      className="animate-spin h-5 w-5 text-gray-400"
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
                    <span className="text-sm text-gray-500">
                      Cargando plantilla...
                    </span>
                  </div>
                </div>
              ) : (
                <TypographyEditor
                  content={content}
                  onChange={setContent}
                  onAIChange={recordAIChange}
                  placeholder="Escribe tu documento aquí..."
                />
              )}
            </div>
            <VersionHistoryPanel
              versions={versions}
              selectedVersionId={selectedVersionId}
              onSelectVersion={setSelectedVersionId}
              onRevertVersion={handleRevertVersion}
              isLoading={isHistoryLoading}
              isReverting={isRevertingVersion}
            />
          </div>
        </div>

        {/* Footer with template selection link */}
        <div className="bg-white border-t border-gray-200 px-6 py-3">
          <div className="flex items-center justify-between max-w-6xl mx-auto">
            <div className="flex items-center gap-4">
              <button
                onClick={() => router.push("/templates")}
                className="text-sm text-blue-600 hover:text-blue-700 hover:underline flex items-center gap-1"
              >
                <svg
                  className="w-4 h-4"
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
                Cambiar plantilla
              </button>
              <span className="text-gray-300">|</span>
              <button
                onClick={() => router.push("/automations")}
                className="text-sm text-blue-600 hover:text-blue-700 hover:underline flex items-center gap-1"
              >
                <svg
                  className="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z"
                  />
                </svg>
                Centro de Automatizaciones
              </button>
            </div>
            <span className="text-xs text-gray-400">
              {template ? `ID: ${template.id}` : "Sin plantilla"}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

function EditorLoading() {
  return (
    <div className="flex h-screen bg-gray-50 items-center justify-center">
      <div className="flex flex-col items-center gap-3">
        <svg
          className="animate-spin h-8 w-8 text-gray-400"
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
        <span className="text-sm text-gray-500">Cargando editor...</span>
      </div>
    </div>
  );
}

export default function EditorPage() {
  return (
    <Suspense fallback={<EditorLoading />}>
      <EditorContent />
    </Suspense>
  );
}
