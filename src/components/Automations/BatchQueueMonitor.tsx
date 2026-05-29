"use client";

import { motion } from "framer-motion";

export type BatchQueueItemStatus = "pending" | "processing" | "completed" | "failed";

export interface BatchQueueMonitorItem {
  id: string;
  fileName: string;
  status: BatchQueueItemStatus;
  queuedAt?: string;
  startedAt?: string | null;
  completedAt?: string | null;
  error?: string | null;
}

interface BatchQueueMonitorProps {
  items: BatchQueueMonitorItem[];
  fallbackEstimatedSeconds?: number;
}

const processedStatuses = new Set<BatchQueueItemStatus>(["completed", "failed"]);

function formatRemainingTime(seconds: number) {
  if (!Number.isFinite(seconds) || seconds <= 0) {
    return "calculando";
  }

  if (seconds < 60) {
    return `${Math.ceil(seconds)}s restantes`;
  }

  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.ceil(seconds % 60);
  return `${minutes}m ${remainingSeconds}s restantes`;
}

function secondsBetween(start?: string | null, end?: string | null) {
  if (!start || !end) {
    return null;
  }

  const startMs = new Date(start).getTime();
  const endMs = new Date(end).getTime();

  if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs <= startMs) {
    return null;
  }

  return (endMs - startMs) / 1000;
}

function calculateEstimatedRemainingSeconds(
  items: BatchQueueMonitorItem[],
  fallbackEstimatedSeconds = 0,
) {
  const remainingItems = items.filter((item) => !processedStatuses.has(item.status)).length;
  if (remainingItems === 0) {
    return 0;
  }

  const completedDurations = items
    .map((item) => secondsBetween(item.startedAt, item.completedAt))
    .filter((duration): duration is number => duration !== null);

  if (completedDurations.length > 0) {
    const averageSeconds =
      completedDurations.reduce((total, duration) => total + duration, 0) / completedDurations.length;
    return averageSeconds * remainingItems;
  }

  return fallbackEstimatedSeconds;
}

function getStatusLabel(status: BatchQueueItemStatus) {
  switch (status) {
    case "completed":
      return "Completado";
    case "failed":
      return "Error";
    case "processing":
      return "Procesando";
    case "pending":
      return "En espera";
  }
}

function StatusMark({ status }: { status: BatchQueueItemStatus }) {
  if (status === "processing") {
    return (
      <span className="relative flex h-3 w-3 shrink-0">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-indigo-400 opacity-75" />
        <span className="relative inline-flex h-3 w-3 rounded-full bg-indigo-500" />
      </span>
    );
  }

  const styles = {
    completed: "bg-emerald-500 text-white",
    failed: "bg-rose-500 text-white",
    pending: "bg-gray-200 text-gray-500 dark:bg-gray-800 dark:text-gray-400",
  } satisfies Record<Exclude<BatchQueueItemStatus, "processing">, string>;

  return (
    <span className={`flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-[10px] font-black ${styles[status]}`}>
      {status === "completed" ? "✓" : status === "failed" ? "!" : ""}
    </span>
  );
}

export function BatchQueueMonitor({ items, fallbackEstimatedSeconds = 0 }: BatchQueueMonitorProps) {
  if (items.length === 0) {
    return null;
  }

  const totalCount = items.length;
  const processedCount = items.filter((item) => processedStatuses.has(item.status)).length;
  const pendingCount = items.filter((item) => item.status === "pending").length;
  const activeItem = items.find((item) => item.status === "processing");
  const progressPercentage = Math.round((processedCount / totalCount) * 100);
  const estimatedRemainingSeconds = calculateEstimatedRemainingSeconds(items, fallbackEstimatedSeconds);

  return (
    <section className="w-full max-w-3xl rounded-xl border border-gray-200 bg-white p-4 text-left shadow-sm dark:border-gray-800 dark:bg-gray-950/80">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-1">
          <p className="text-xs font-bold uppercase tracking-wide text-gray-400">Cola global de automatización</p>
          <h4 className="text-lg font-black text-gray-950 dark:text-white">
            {processedCount} / {totalCount} Procesados - {formatRemainingTime(estimatedRemainingSeconds)}
          </h4>
          <p className="text-sm font-medium text-gray-500 dark:text-gray-400">
            Activo: {activeItem?.fileName ?? "sin procesamiento activo"} · {pendingCount} en espera
          </p>
        </div>
        <div className="grid grid-cols-3 gap-2 text-center text-xs font-bold">
          <div className="rounded-lg bg-emerald-50 px-3 py-2 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300">
            {processedCount}
            <span className="block text-[10px] uppercase">listos</span>
          </div>
          <div className="rounded-lg bg-indigo-50 px-3 py-2 text-indigo-700 dark:bg-indigo-950/30 dark:text-indigo-300">
            {activeItem ? 1 : 0}
            <span className="block text-[10px] uppercase">activo</span>
          </div>
          <div className="rounded-lg bg-gray-100 px-3 py-2 text-gray-600 dark:bg-gray-900 dark:text-gray-300">
            {pendingCount}
            <span className="block text-[10px] uppercase">cola</span>
          </div>
        </div>
      </div>

      <div className="mt-4 h-2 overflow-hidden rounded-full bg-gray-100 dark:bg-gray-800">
        <motion.div
          className="h-full rounded-full bg-indigo-500"
          animate={{ width: `${progressPercentage}%` }}
          transition={{ duration: 0.35, ease: "easeOut" }}
        />
      </div>

      <div className="mt-4 max-h-64 space-y-2 overflow-y-auto pr-1">
        {items.map((item) => (
          <motion.div
            key={item.id}
            layout
            className={`flex min-h-12 items-center gap-3 rounded-lg border px-3 py-2 transition-colors ${
              item.status === "processing"
                ? "border-indigo-200 bg-indigo-50/70 dark:border-indigo-900 dark:bg-indigo-950/30"
                : "border-gray-100 bg-gray-50/60 dark:border-gray-850 dark:bg-gray-900/60"
            }`}
          >
            <StatusMark status={item.status} />
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-bold text-gray-900 dark:text-gray-100">{item.fileName}</p>
              <p className="truncate text-xs text-gray-500 dark:text-gray-400">
                {item.error ? item.error : getStatusLabel(item.status)}
              </p>
            </div>
            {item.status === "processing" && (
              <motion.div
                className="h-1.5 w-16 overflow-hidden rounded-full bg-indigo-100 dark:bg-indigo-950"
                aria-label="Archivo en procesamiento"
              >
                <motion.div
                  className="h-full w-1/2 rounded-full bg-indigo-500"
                  animate={{ x: ["-100%", "220%"] }}
                  transition={{ repeat: Infinity, duration: 1.1, ease: "easeInOut" }}
                />
              </motion.div>
            )}
          </motion.div>
        ))}
      </div>
    </section>
  );
}
