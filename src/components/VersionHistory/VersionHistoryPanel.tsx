"use client";

import { useMemo } from "react";
import type { DocumentVersion } from "@/lib/types";
import { diffText, htmlToPlainText } from "@/lib/text-diff";

interface VersionHistoryPanelProps {
  versions: DocumentVersion[];
  selectedVersionId: string | null;
  onSelectVersion: (versionId: string) => void;
  onRevertVersion: (version: DocumentVersion) => void;
  isLoading?: boolean;
  isReverting?: boolean;
}

export function VersionHistoryPanel({
  versions,
  selectedVersionId,
  onSelectVersion,
  onRevertVersion,
  isLoading = false,
  isReverting = false,
}: VersionHistoryPanelProps) {
  const selectedVersion =
    versions.find((version) => version.id === selectedVersionId) ?? versions[0];
  const diffSegments = useMemo(() => {
    if (!selectedVersion) return [];

    return diffText(
      htmlToPlainText(selectedVersion.previous_content),
      htmlToPlainText(selectedVersion.new_content)
    );
  }, [selectedVersion]);

  return (
    <aside className="min-h-[600px] rounded-xl border border-gray-200 bg-white shadow-sm">
      <div className="border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <div>
            <h2 className="text-sm font-semibold text-gray-900">
              Auditoria de IA
            </h2>
            <p className="mt-0.5 text-xs text-gray-500">
              {versions.length} {versions.length === 1 ? "version" : "versiones"}
            </p>
          </div>
          <span className="rounded-full border border-emerald-200 bg-emerald-50 px-2 py-1 text-xs font-medium text-emerald-700">
            Local
          </span>
        </div>
      </div>

      <div className="grid max-h-[720px] grid-rows-[180px_minmax(0,1fr)]">
        <div className="overflow-auto border-b border-gray-200 p-3">
          {isLoading ? (
            <div className="rounded-lg border border-gray-200 bg-gray-50 p-3 text-sm text-gray-500">
              Cargando historial...
            </div>
          ) : versions.length === 0 ? (
            <div className="rounded-lg border border-dashed border-gray-300 bg-gray-50 p-3 text-sm text-gray-500">
              Los cambios de IA apareceran aqui.
            </div>
          ) : (
            <div className="space-y-2">
              {versions.map((version) => (
                <button
                  key={version.id}
                  type="button"
                  onClick={() => onSelectVersion(version.id)}
                  className={`w-full rounded-lg border px-3 py-2 text-left transition-colors ${
                    selectedVersion?.id === version.id
                      ? "border-gray-900 bg-gray-900 text-white"
                      : "border-gray-200 bg-white text-gray-700 hover:border-gray-300 hover:bg-gray-50"
                  }`}
                >
                  <span className="block truncate text-sm font-medium">
                    {version.action_label}
                  </span>
                  <span
                    className={`mt-1 block text-xs ${
                      selectedVersion?.id === version.id
                        ? "text-gray-300"
                        : "text-gray-500"
                    }`}
                  >
                    {formatTimestamp(version.created_at)}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="flex min-h-0 flex-col">
          <div className="flex items-center justify-between gap-3 border-b border-gray-200 px-4 py-3">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold text-gray-900">
                {selectedVersion?.action_label ?? "Sin version seleccionada"}
              </h3>
              <p className="text-xs text-gray-500">
                Rojo eliminado, verde agregado
              </p>
            </div>
            <button
              type="button"
              onClick={() => selectedVersion && onRevertVersion(selectedVersion)}
              disabled={!selectedVersion || isReverting}
              className="inline-flex shrink-0 items-center gap-2 rounded-lg bg-rose-600 px-3 py-2 text-xs font-semibold text-white transition-colors hover:bg-rose-700 disabled:cursor-not-allowed disabled:opacity-50"
              title="Revertir el documento al estado anterior"
            >
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
                  d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3"
                />
              </svg>
              {isReverting ? "Revirtiendo" : "Revertir"}
            </button>
          </div>

          <div className="min-h-0 flex-1 overflow-auto p-4">
            {selectedVersion ? (
              <div className="whitespace-pre-wrap rounded-lg border border-gray-200 bg-gray-50 p-4 text-sm leading-7 text-gray-800">
                {diffSegments.map((segment, index) => {
                  if (segment.type === "added") {
                    return (
                      <ins
                        key={`${segment.type}-${index}`}
                        className="rounded bg-emerald-100 px-1 text-emerald-800 no-underline"
                      >
                        {segment.text}
                      </ins>
                    );
                  }

                  if (segment.type === "removed") {
                    return (
                      <del
                        key={`${segment.type}-${index}`}
                        className="rounded bg-rose-100 px-1 text-rose-800"
                      >
                        {segment.text}
                      </del>
                    );
                  }

                  return <span key={`${segment.type}-${index}`}>{segment.text}</span>;
                })}
              </div>
            ) : (
              <div className="rounded-lg border border-dashed border-gray-300 bg-gray-50 p-4 text-sm text-gray-500">
                Selecciona una version para comparar el cambio.
              </div>
            )}
          </div>
        </div>
      </div>
    </aside>
  );
}

function formatTimestamp(value: string): string {
  const timestamp = Number(value);
  if (!Number.isFinite(timestamp)) return value;

  return new Intl.DateTimeFormat("es", {
    dateStyle: "short",
    timeStyle: "short",
  }).format(new Date(timestamp));
}
