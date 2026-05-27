'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { IndexedFile, KnowledgeBaseStats } from './types';

interface MetricCardProps {
  title: string;
  value: string | number;
  description: string;
  icon: React.ReactNode;
  trend?: {
    value: string;
    positive: boolean;
  };
  onClick?: () => void;
  className?: string;
}

export function MetricCard({ title, value, description, icon, trend, onClick, className = '' }: MetricCardProps) {
  const CardWrapper = (onClick ? motion.button : 'div') as any;
  const extraProps = onClick ? { whileHover: { y: -2 }, whileTap: { scale: 0.99 } } : {};

  return (
    <CardWrapper
      onClick={onClick}
      {...extraProps}
      className={`rounded-2xl border border-gray-200/80 dark:border-gray-800 bg-white/70 dark:bg-gray-900/60 p-6 backdrop-blur-md shadow-sm flex items-start justify-between w-full text-left ${onClick ? 'cursor-pointer hover:bg-white/90 dark:hover:bg-gray-950/80 transition-colors' : ''} ${className}`}
    >
      <div className="flex-1 min-w-0 pr-4">
        <span className="text-sm font-medium text-gray-500 dark:text-gray-400 tracking-wide uppercase">
          {title}
        </span>
        <h4 className="text-3xl font-extrabold text-gray-900 dark:text-white mt-2 tracking-tight">
          {value}
        </h4>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1.5 font-medium truncate">
          {description}
        </p>
        
        {trend && (
          <div className="flex items-center gap-1 mt-3">
            <span className={`inline-flex items-center text-xs font-semibold px-2 py-0.5 rounded-full ${
              trend.positive 
                ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-400' 
                : 'bg-rose-50 text-rose-700 dark:bg-rose-950/30 dark:text-rose-400'
            }`}>
              {trend.positive ? '+' : ''}{trend.value}
            </span>
            <span className="text-xs text-gray-400 font-medium">vs last week</span>
          </div>
        )}
      </div>

      <div className="rounded-xl p-3 bg-gray-50 dark:bg-gray-800/40 text-gray-700 dark:text-gray-300 shrink-0">
        {icon}
      </div>
    </CardWrapper>
  );
}

interface DiskSpaceMetricProps {
  stats: KnowledgeBaseStats;
}

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export function DiskSpaceMetric({ stats }: DiskSpaceMetricProps) {
  const percentage = Math.min(100, Math.round((stats.diskSpaceUsedBytes / stats.diskSpaceMaxBytes) * 100));

  return (
    <div className="rounded-2xl border border-gray-200/80 dark:border-gray-800 bg-white/70 dark:bg-gray-900/60 p-6 backdrop-blur-md shadow-sm w-full">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-500 dark:text-gray-400 tracking-wide uppercase">
          Disk Space Occupied
        </span>
        <div className="rounded-xl p-3 bg-gray-50 dark:bg-gray-800/40 text-gray-700 dark:text-gray-300 shrink-0">
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.75}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M5.25 14.25h13.5m-13.5 3h13.5m-13.5-6h13.5m-13.5-3h13.5m-13.5-3h13.5m-13.5 15H21A2.25 2.25 0 0 0 22.5 16.5v-9a2.25 2.25 0 0 0-2.25-2.25H1.5A2.25 2.25 0 0 0 0 7.5v9a2.25 2.25 0 0 0 2.25 2.25h3" />
          </svg>
        </div>
      </div>

      <div className="mt-4 flex items-baseline gap-2">
        <span className="text-3xl font-extrabold text-gray-900 dark:text-white tracking-tight">
          {percentage}%
        </span>
        <span className="text-sm font-medium text-gray-400 dark:text-gray-500">
          ({formatBytes(stats.diskSpaceUsedBytes)} / {formatBytes(stats.diskSpaceMaxBytes)})
        </span>
      </div>

      {/* Modern, segmented linear progress bar */}
      <div className="mt-4 w-full bg-gray-100 dark:bg-gray-800 h-2.5 rounded-full overflow-hidden flex gap-0.5">
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: `${percentage}%` }}
          transition={{ duration: 1, ease: 'easeOut' }}
          className={`h-full rounded-full bg-gradient-to-r ${
            percentage > 90 
              ? 'from-rose-500 to-red-600' 
              : percentage > 75 
                ? 'from-amber-500 to-orange-600' 
                : 'from-blue-600 to-indigo-600'
          }`}
        />
      </div>

      <div className="flex justify-between items-center mt-3 text-xs font-semibold text-gray-400 dark:text-gray-500">
        <span>0% Empty</span>
        <span>{percentage > 90 ? 'Critical Capacity' : 'Optimized Space'}</span>
      </div>
    </div>
  );
}

interface AlertPanelProps {
  corruptFiles: IndexedFile[];
  onReindex: (id: string) => void;
  onDelete: (id: string) => void;
}

export function AlertPanel({ corruptFiles, onReindex, onDelete }: AlertPanelProps) {
  const [isExpanded, setIsExpanded] = useState(false);

  if (corruptFiles.length === 0) return null;

  return (
    <div className="rounded-2xl border border-red-500/20 bg-red-500/5 dark:bg-red-500/5 backdrop-blur-md overflow-hidden">
      <div 
        onClick={() => setIsExpanded(!isExpanded)}
        className="px-6 py-4 flex items-center justify-between cursor-pointer select-none hover:bg-red-500/10 dark:hover:bg-red-500/10 transition-colors duration-200"
      >
        <div className="flex items-center gap-3">
          <div className="rounded-xl p-2.5 bg-red-100 dark:bg-red-950/40 text-red-600 dark:text-red-400 animate-pulse shrink-0">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
            </svg>
          </div>
          <div>
            <h4 className="text-sm font-bold text-red-800 dark:text-red-300">
              Corrupt Ingestion Warnings Detected
            </h4>
            <p className="text-xs text-red-600 dark:text-red-400 mt-0.5 font-semibold">
              {corruptFiles.length} {corruptFiles.length === 1 ? 'file' : 'files'} failed vector processing. Needs immediate administrative review.
            </p>
          </div>
        </div>

        <button className="text-red-500 dark:text-red-400 p-1.5 rounded-lg hover:bg-red-500/10 dark:hover:bg-red-500/10 transition-colors">
          <motion.svg
            animate={{ rotate: isExpanded ? 180 : 0 }}
            transition={{ duration: 0.2 }}
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            strokeWidth={2}
          >
            <path strokeLinecap="round" strokeLinejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
          </motion.svg>
        </button>
      </div>

      <AnimatePresence initial={false}>
        {isExpanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            transition={{ duration: 0.25, ease: 'easeInOut' }}
            className="border-t border-red-500/15 overflow-hidden bg-red-500/[0.02]"
          >
            <div className="p-6 space-y-4 max-h-[350px] overflow-y-auto">
              {corruptFiles.map((file) => (
                <div 
                  key={file.id} 
                  className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-4 rounded-xl border border-red-500/10 bg-white/40 dark:bg-gray-900/40 backdrop-blur-md shadow-sm"
                >
                  <div className="flex items-start gap-3 min-w-0">
                    {/* Format Badge/Icon */}
                    <div className="rounded-lg p-2 bg-red-50 dark:bg-red-950/20 text-red-500 shrink-0 font-bold text-xs uppercase tracking-wider">
                      {file.format}
                    </div>
                    <div className="min-w-0">
                      <h5 className="text-sm font-bold text-gray-900 dark:text-gray-100 truncate">
                        {file.name}
                      </h5>
                      <div className="flex items-center gap-2 mt-1 text-xs text-gray-500">
                        <span className="font-semibold">{file.virtualFolder}</span>
                        <span className="w-1 h-1 rounded-full bg-gray-300 dark:bg-gray-700" />
                        <span>{formatBytes(file.sizeBytes)}</span>
                      </div>
                      <p className="text-xs text-red-600 dark:text-red-400 font-semibold mt-2 border-l-2 border-red-500 pl-2 bg-red-500/[0.03] py-1 rounded">
                        Error log: {file.errorMessage || 'Unknown vector indexing corruption error'}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 shrink-0 self-end sm:self-center">
                    <button
                      onClick={() => onReindex(file.id)}
                      className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-red-600 dark:bg-red-500 text-white text-xs font-bold shadow hover:bg-red-700 dark:hover:bg-red-600 transition-colors"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99" />
                      </svg>
                      Re-index
                    </button>
                    <button
                      onClick={() => onDelete(file.id)}
                      className="p-1.5 rounded-lg border border-red-500/20 text-red-500 dark:text-red-400 hover:bg-red-500/10 dark:hover:bg-red-500/10 transition-colors"
                      title="Remove permanently"
                    >
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                      </svg>
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
