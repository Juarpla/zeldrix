'use client';

import { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { IndexedFile } from './types';

interface FileTableProps {
  files: IndexedFile[];
  onReindex: (id: string) => void;
  onDelete: (id: string) => void;
  selectedFolder: string | null;
}

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
}

export default function FileTable({ files, onReindex, onDelete, selectedFolder }: FileTableProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<'all' | 'completed' | 'processing' | 'corrupt'>('all');
  const [formatFilter, setFormatFilter] = useState<'all' | 'pdf' | 'docx' | 'xlsx' | 'txt'>('all');
  const [activeMenuId, setActiveMenuId] = useState<string | null>(null);

  // Filtered files
  const filteredFiles = useMemo(() => {
    return files.filter((file) => {
      const matchesSearch = file.name.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesFolder = selectedFolder ? file.virtualFolder === selectedFolder : true;
      const matchesStatus = statusFilter === 'all' ? true : file.indexingStatus === statusFilter;
      const matchesFormat = formatFilter === 'all' ? true : file.format === formatFilter;

      return matchesSearch && matchesFolder && matchesStatus && matchesFormat;
    });
  }, [files, searchQuery, selectedFolder, statusFilter, formatFilter]);

  // Format Icons for premium file representation
  const renderFileIcon = (format: IndexedFile['format']) => {
    switch (format) {
      case 'pdf':
        return (
          <div className="rounded-lg p-2 bg-red-50 dark:bg-red-950/20 text-red-600 dark:text-red-400 shrink-0">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
            </svg>
          </div>
        );
      case 'docx':
        return (
          <div className="rounded-lg p-2 bg-blue-50 dark:bg-blue-950/20 text-blue-600 dark:text-blue-400 shrink-0">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
            </svg>
          </div>
        );
      case 'xlsx':
        return (
          <div className="rounded-lg p-2 bg-emerald-50 dark:bg-emerald-950/20 text-emerald-600 dark:text-emerald-400 shrink-0">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 12h16.5m-16.5 3.75h16.5M3.75 19.5h16.5M5.625 4.5h12.75c.621 0 1.125.504 1.125 1.125v12.75c0 .621-.504 1.125-1.125 1.125H5.625a1.125 1.125 0 0 1-1.125-1.125V5.625c0-.621.504-1.125 1.125-1.125Z" />
            </svg>
          </div>
        );
      default:
        return (
          <div className="rounded-lg p-2 bg-gray-50 dark:bg-gray-800/40 text-gray-600 dark:text-gray-400 shrink-0">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
            </svg>
          </div>
        );
    }
  };

  // Status Badges rendering
  const renderStatus = (status: IndexedFile['indexingStatus'], errorMessage?: string) => {
    switch (status) {
      case 'completed':
        return (
          <span className="inline-flex items-center gap-1.5 rounded-full bg-emerald-50 dark:bg-emerald-950/20 px-2.5 py-1 text-xs font-bold text-emerald-700 dark:text-emerald-400 border border-emerald-500/10">
            <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
            Indexed
          </span>
        );
      case 'processing':
        return (
          <span className="inline-flex items-center gap-1.5 rounded-full bg-blue-50 dark:bg-blue-950/20 px-2.5 py-1 text-xs font-bold text-blue-700 dark:text-blue-400 border border-blue-500/10 animate-pulse">
            {/* Spinning vector load indicator */}
            <svg className="animate-spin h-3.5 w-3.5 text-blue-500" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            Vectorizing
          </span>
        );
      case 'corrupt':
        return (
          <span 
            title={errorMessage}
            className="inline-flex items-center gap-1.5 rounded-full bg-red-50 dark:bg-red-950/20 px-2.5 py-1 text-xs font-bold text-red-700 dark:text-red-400 border border-red-500/10 group relative cursor-help"
          >
            <span className="h-1.5 w-1.5 rounded-full bg-red-500 animate-ping" />
            Corrupt
            {errorMessage && (
              <span className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 w-48 hidden group-hover:block bg-gray-900 text-white text-[10px] p-2 rounded shadow-lg z-20 text-center font-normal leading-normal">
                {errorMessage}
              </span>
            )}
          </span>
        );
    }
  };

  return (
    <div className="rounded-2xl border border-gray-200/80 dark:border-gray-800 bg-white/70 dark:bg-gray-900/60 p-6 backdrop-blur-md shadow-sm w-full">
      {/* Search and filter action bar */}
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4 pb-6 border-b border-gray-100 dark:border-gray-800">
        <div className="relative flex-1 min-w-[280px]">
          <svg className="absolute left-3 top-3.5 h-4 w-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="m21-21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z" />
          </svg>
          <input
            type="text"
            placeholder="Search index by file name..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 pl-10 pr-4 py-2.5 text-sm font-semibold text-gray-900 dark:text-gray-100 placeholder:text-gray-400 focus:border-blue-500 focus:outline-none transition-all"
          />
        </div>

        {/* Filters */}
        <div className="flex flex-wrap items-center gap-2">
          {/* Format filter */}
          <select
            value={formatFilter}
            onChange={(e: any) => setFormatFilter(e.target.value)}
            className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 px-3.5 py-2.5 text-xs font-bold text-gray-700 dark:text-gray-300 focus:border-blue-500 focus:outline-none shadow-sm"
          >
            <option value="all">All Formats</option>
            <option value="pdf">PDF</option>
            <option value="docx">DOCX</option>
            <option value="xlsx">XLSX</option>
            <option value="txt">TXT</option>
          </select>

          {/* Status filter */}
          <select
            value={statusFilter}
            onChange={(e: any) => setStatusFilter(e.target.value)}
            className="rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 px-3.5 py-2.5 text-xs font-bold text-gray-700 dark:text-gray-300 focus:border-blue-500 focus:outline-none shadow-sm"
          >
            <option value="all">All Statuses</option>
            <option value="completed">Indexed</option>
            <option value="processing">Vectorizing</option>
            <option value="corrupt">Corrupt</option>
          </select>

          {/* Selected virtual folder clear indicator */}
          {selectedFolder && (
            <div className="inline-flex items-center gap-1.5 px-3 py-2 rounded-xl bg-blue-50 dark:bg-blue-950/20 text-blue-700 dark:text-blue-400 border border-blue-500/10 text-xs font-bold shadow-sm">
              Folder: {selectedFolder}
            </div>
          )}
        </div>
      </div>

      {/* Main Files Table */}
      <div className="overflow-x-auto mt-4">
        <table className="min-w-full table-auto text-left border-collapse">
          <thead>
            <tr className="border-b border-gray-100 dark:border-gray-800 text-[10px] font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
              <th className="py-4 px-4">Document Details</th>
              <th className="py-4 px-4">Virtual Folder</th>
              <th className="py-4 px-4">Format</th>
              <th className="py-4 px-4">File Size</th>
              <th className="py-4 px-4">Ingestion Status</th>
              <th className="py-4 px-4 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100/50 dark:divide-gray-800/40 text-sm font-semibold text-gray-700 dark:text-gray-300">
            <AnimatePresence initial={false}>
              {filteredFiles.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-12 text-center text-gray-400 dark:text-gray-500">
                    <svg className="w-12 h-12 mx-auto mb-3 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={1.5}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                    </svg>
                    <p className="text-sm font-bold">No indexed files found matching criteria</p>
                    <p className="text-xs mt-1">Try resetting the search or importing new records.</p>
                  </td>
                </tr>
              ) : (
                filteredFiles.map((file) => (
                  <motion.tr
                    key={file.id}
                    layoutId={file.id}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -4 }}
                    transition={{ duration: 0.2 }}
                    className="hover:bg-gray-50/50 dark:hover:bg-gray-800/10 transition-colors duration-150 group"
                  >
                    {/* Document details name / added date */}
                    <td className="py-4 px-4">
                      <div className="flex items-center gap-3">
                        {renderFileIcon(file.format)}
                        <div className="min-w-0">
                          <p className="font-bold text-gray-900 dark:text-gray-100 hover:text-blue-600 dark:hover:text-blue-400 transition-colors truncate max-w-[200px] md:max-w-xs cursor-pointer">
                            {file.name}
                          </p>
                          <p className="text-[10px] text-gray-400 dark:text-gray-500 font-semibold mt-0.5">
                            Added {new Date(file.addedAt).toLocaleDateString()}
                          </p>
                        </div>
                      </div>
                    </td>

                    {/* Virtual folder */}
                    <td className="py-4 px-4 font-bold text-gray-500 dark:text-gray-400">
                      {file.virtualFolder}
                    </td>

                    {/* Format */}
                    <td className="py-4 px-4">
                      <span className="uppercase text-xs font-extrabold text-gray-400 tracking-wider">
                        {file.format}
                      </span>
                    </td>

                    {/* Size */}
                    <td className="py-4 px-4 font-semibold text-gray-500 dark:text-gray-400">
                      {formatBytes(file.sizeBytes)}
                    </td>

                    {/* Status Badge */}
                    <td className="py-4 px-4">
                      {renderStatus(file.indexingStatus, file.errorMessage)}
                    </td>

                    {/* Inline Actions */}
                    <td className="py-4 px-4 text-right relative">
                      <div className="flex items-center justify-end gap-1.5">
                        <button
                          onClick={() => onReindex(file.id)}
                          disabled={file.indexingStatus === 'processing'}
                          className="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors disabled:opacity-30"
                          title="Re-index file"
                        >
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99" />
                          </svg>
                        </button>
                        <button
                          onClick={() => onDelete(file.id)}
                          className="p-1.5 rounded-lg hover:bg-red-50 dark:hover:bg-red-950/20 text-gray-400 hover:text-red-500 dark:hover:text-red-400 transition-colors"
                          title="Delete file index"
                        >
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
                            <path strokeLinecap="round" strokeLinejoin="round" d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0" />
                          </svg>
                        </button>
                      </div>
                    </td>
                  </motion.tr>
                ))
              )}
            </AnimatePresence>
          </tbody>
        </table>
      </div>
      
      {/* Table Footer Stats Summary */}
      <div className="flex justify-between items-center mt-6 pt-4 border-t border-gray-100 dark:border-gray-800 text-xs font-semibold text-gray-400 dark:text-gray-500">
        <span>Showing {filteredFiles.length} of {files.length} indexed documents</span>
        <span>Corporate Vector Index v2.1.0</span>
      </div>
    </div>
  );
}
