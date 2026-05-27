'use client';

import { motion } from 'framer-motion';
import { VirtualFolder } from './types';

interface FolderCardProps {
  folder: VirtualFolder;
  isSelected: boolean;
  onClick: () => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export default function FolderCard({ folder, isSelected, onClick }: FolderCardProps) {
  // Map color names to modern, premium tailwind gradient/border configurations
  const colorMap: Record<string, { bg: string; text: string; iconBg: string; border: string; glow: string }> = {
    blue: {
      bg: 'from-blue-500/10 to-indigo-500/5 hover:from-blue-500/15 hover:to-indigo-500/10',
      text: 'text-blue-600 dark:text-blue-400',
      iconBg: 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400',
      border: 'border-blue-500/20 hover:border-blue-500/40',
      glow: 'shadow-blue-500/10',
    },
    emerald: {
      bg: 'from-emerald-500/10 to-teal-500/5 hover:from-emerald-500/15 hover:to-teal-500/10',
      text: 'text-emerald-600 dark:text-emerald-400',
      iconBg: 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400',
      border: 'border-emerald-500/20 hover:border-emerald-500/40',
      glow: 'shadow-emerald-500/10',
    },
    purple: {
      bg: 'from-purple-500/10 to-fuchsia-500/5 hover:from-purple-500/15 hover:to-fuchsia-500/10',
      text: 'text-purple-600 dark:text-purple-400',
      iconBg: 'bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400',
      border: 'border-purple-500/20 hover:border-purple-500/40',
      glow: 'shadow-purple-500/10',
    },
    amber: {
      bg: 'from-amber-500/10 to-orange-500/5 hover:from-amber-500/15 hover:to-orange-500/10',
      text: 'text-amber-600 dark:text-amber-400',
      iconBg: 'bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400',
      border: 'border-amber-500/20 hover:border-amber-500/40',
      glow: 'shadow-amber-500/10',
    },
  };

  const currentTheme = colorMap[folder.color] || colorMap.blue;

  return (
    <motion.button
      onClick={onClick}
      whileHover={{ y: -4, scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      className={`relative w-full text-left rounded-2xl border p-5 backdrop-blur-md transition-all duration-300 bg-gradient-to-br ${
        isSelected
          ? 'border-gray-900 dark:border-white bg-white/20 dark:bg-gray-800/30 shadow-lg ring-1 ring-gray-900/5 dark:ring-white/10'
          : `bg-white/80 dark:bg-gray-900/60 shadow-sm ${currentTheme.border} ${currentTheme.bg} ${currentTheme.glow}`
      }`}
    >
      <div className="flex items-start justify-between">
        {/* Folder Icon Wrapper */}
        <div className={`rounded-xl p-3 ${currentTheme.iconBg}`}>
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            strokeWidth={1.75}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h1.52c.318 0 .628-.097.891-.276L9.3 7.845a2.25 2.25 0 0 1 1.205-.35h7.245c.807 0 1.5.504 1.75 1.27l1.25 3.83c.083.25.12.514.12.78v5.875a2.25 2.25 0 0 1-2.25 2.25H4.5a2.25 2.25 0 0 1-2.25-2.25V12.75Z"
            />
          </svg>
        </div>

        {/* Selected indicator badge */}
        {isSelected && (
          <span className="flex h-2 w-2 relative">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-gray-900 dark:bg-white opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-gray-900 dark:bg-white"></span>
          </span>
        )}
      </div>

      <div className="mt-5">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 tracking-tight truncate">
          {folder.name}
        </h3>
        
        <div className="flex items-center gap-3 mt-1.5 text-sm text-gray-500 dark:text-gray-400">
          <span className="flex items-center gap-1">
            <svg className="w-4 h-4 opacity-70" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
            </svg>
            {folder.fileCount} {folder.fileCount === 1 ? 'document' : 'documents'}
          </span>
          <span className="w-1.5 h-1.5 rounded-full bg-gray-300 dark:bg-gray-700" />
          <span>{formatBytes(folder.sizeBytes)}</span>
        </div>
      </div>
    </motion.button>
  );
}
