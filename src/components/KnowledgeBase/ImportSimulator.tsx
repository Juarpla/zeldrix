'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';

interface ImportSimulatorProps {
  onSimulateImport: (name: string, format: 'pdf' | 'docx' | 'xlsx' | 'txt', virtualFolder: string, isCorrupt: boolean) => void;
  onSimulateBatch: () => void;
  disabled: boolean;
}

const folders = ['Engineering Docs', 'Legal & Compliance', 'Marketing Strategy', 'Product Manuals'];
const validFiles = [
  { name: 'engineering_spec_v3.pdf', format: 'pdf' },
  { name: 'employee_handbook_2026.docx', format: 'docx' },
  { name: 'revenue_projection_q3.xlsx', format: 'xlsx' },
  { name: 'api_endpoints_documentation.txt', format: 'txt' },
  { name: 'system_architecture_diagram.pdf', format: 'pdf' },
  { name: 'brand_guidelines_corporate.docx', format: 'docx' },
] as const;

const corruptFiles = [
  { name: 'damaged_scan_invoice_491.pdf', format: 'pdf', reason: 'OCR Required - No extractable text layer found' },
  { name: 'corrupted_payroll_ledger.xlsx', format: 'xlsx', reason: 'Invalid signature - File format is unsupported or encrypted' },
  { name: 'malformed_config_dump.txt', format: 'txt', reason: 'UTF-8 Decoding failure - Invalid byte sequence detected' },
] as const;

export default function ImportSimulator({ onSimulateImport, onSimulateBatch, disabled }: ImportSimulatorProps) {
  const [selectedFolder, setSelectedFolder] = useState(folders[0]);

  const handleSimulateValid = () => {
    const randomFile = validFiles[Math.floor(Math.random() * validFiles.length)];
    onSimulateImport(randomFile.name, randomFile.format, selectedFolder, false);
  };

  const handleSimulateCorrupt = () => {
    const randomFile = corruptFiles[Math.floor(Math.random() * corruptFiles.length)];
    // We encode a special flag or pass true for corruption
    onSimulateImport(randomFile.name, randomFile.format, selectedFolder, true);
  };

  return (
    <div className="rounded-2xl border border-gray-200/80 dark:border-gray-800 bg-white/70 dark:bg-gray-900/60 p-6 backdrop-blur-md shadow-sm w-full">
      <div className="flex items-center gap-3">
        <div className="rounded-xl p-2.5 bg-blue-100 dark:bg-blue-950/40 text-blue-600 dark:text-blue-400 shrink-0">
          <svg className="w-5 h-5 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 1 1-3 0m3 0a1.5 1.5 0 1 0-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 0 1-3 0m3 0a1.5 1.5 0 0 0-3 0m-9.75 0h9.75" />
          </svg>
        </div>
        <div>
          <h4 className="text-sm font-bold text-gray-900 dark:text-gray-100">
            Ingestion Pipeline Simulator
          </h4>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 font-semibold">
            Test vector ingestion queue states and corruption triggers in real-time.
          </p>
        </div>
      </div>

      <div className="mt-5 grid grid-cols-1 md:grid-cols-4 gap-4 items-end">
        {/* Select Virtual Folder */}
        <div className="md:col-span-2">
          <label className="block text-xs font-bold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-2">
            Target Virtual Folder
          </label>
          <select
            value={selectedFolder}
            onChange={(e) => setSelectedFolder(e.target.value)}
            disabled={disabled}
            className="w-full rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 px-4 py-2.5 text-sm font-semibold text-gray-900 dark:text-gray-100 shadow-sm focus:border-blue-500 focus:outline-none disabled:opacity-50"
          >
            {folders.map((folder) => (
              <option key={folder} value={folder}>
                {folder}
              </option>
            ))}
          </select>
        </div>

        {/* Buttons Grid */}
        <div className="md:col-span-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={handleSimulateValid}
            disabled={disabled}
            className="inline-flex items-center justify-center gap-1.5 rounded-xl bg-blue-600 hover:bg-blue-700 text-white text-xs font-bold px-4 py-2.5 shadow-sm transition-colors disabled:opacity-50"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
            </svg>
            Add Valid Doc
          </motion.button>

          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={handleSimulateCorrupt}
            disabled={disabled}
            className="inline-flex items-center justify-center gap-1.5 rounded-xl bg-amber-500 hover:bg-amber-600 text-white text-xs font-bold px-4 py-2.5 shadow-sm transition-colors disabled:opacity-50"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
            </svg>
            Add Corrupt
          </motion.button>

          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={onSimulateBatch}
            disabled={disabled}
            className="inline-flex items-center justify-center gap-1.5 rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-900 text-xs font-bold px-4 py-2.5 shadow-sm transition-colors disabled:opacity-50"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v5.25c0 .621-.504 1.125-1.125 1.125h-2.25A1.125 1.125 0 0 1 3 18.375v-5.25ZM9 9.75c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v8.625c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V9.75ZM15 5.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v12.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V5.625Z" />
            </svg>
            Ingest Batch
          </motion.button>
        </div>
      </div>
    </div>
  );
}
