'use client';

import { useState, useCallback, useMemo } from 'react';
import { motion } from 'framer-motion';
import { IndexedFile, VirtualFolder, KnowledgeBaseStats } from './types';
import FolderCard from './FolderCard';
import { MetricCard, DiskSpaceMetric, AlertPanel } from './MetricCard';
import FileTable from './FileTable';
import ImportSimulator from './ImportSimulator';

const INITIAL_FILES: IndexedFile[] = [
  {
    id: 'file-1',
    name: 'corporate_security_policy_2026.pdf',
    sizeBytes: 15482920, // 14.8 MB
    format: 'pdf',
    virtualFolder: 'Legal & Compliance',
    indexingStatus: 'completed',
    addedAt: '2026-04-12T10:30:00Z',
  },
  {
    id: 'file-2',
    name: 'product_requirements_roadmap.docx',
    sizeBytes: 2548201, // 2.4 MB
    format: 'docx',
    virtualFolder: 'Product Manuals',
    indexingStatus: 'completed',
    addedAt: '2026-05-01T14:15:00Z',
  },
  {
    id: 'file-3',
    name: 'annual_budget_forecast_q4.xlsx',
    sizeBytes: 42938102, // 40.9 MB
    format: 'xlsx',
    virtualFolder: 'Engineering Docs',
    indexingStatus: 'completed',
    addedAt: '2026-05-10T09:00:00Z',
  },
  {
    id: 'file-4',
    name: 'technical_architecture_diagrams.pdf',
    sizeBytes: 94827104, // 90.4 MB
    format: 'pdf',
    virtualFolder: 'Engineering Docs',
    indexingStatus: 'completed',
    addedAt: '2026-05-18T16:45:00Z',
  },
  {
    id: 'file-5',
    name: 'brand_strategy_guidelines.docx',
    sizeBytes: 5291840, // 5.0 MB
    format: 'docx',
    virtualFolder: 'Marketing Strategy',
    indexingStatus: 'completed',
    addedAt: '2026-05-20T11:20:00Z',
  },
  {
    id: 'file-6',
    name: 'damaged_scanned_contract.pdf',
    sizeBytes: 8394820, // 8.0 MB
    format: 'pdf',
    virtualFolder: 'Legal & Compliance',
    indexingStatus: 'corrupt',
    errorMessage: 'OCR Layer Required - PDF contains non-extractable image-only text layers',
    addedAt: '2026-05-25T08:30:00Z',
  },
  {
    id: 'file-7',
    name: 'encrypted_payroll_schema.xlsx',
    sizeBytes: 12048912, // 11.5 MB
    format: 'xlsx',
    virtualFolder: 'Legal & Compliance',
    indexingStatus: 'corrupt',
    errorMessage: 'File decryption failure - Unsupported password-encrypted spreadsheet signature',
    addedAt: '2026-05-26T15:10:00Z',
  },
];

export default function KnowledgeBaseDashboard() {
  const [files, setFiles] = useState<IndexedFile[]>(INITIAL_FILES);
  const [selectedFolderId, setSelectedFolderId] = useState<string | null>(null);
  const [isSimulatingQueue, setIsSimulatingQueue] = useState(false);

  // Re-indexing trigger
  const handleReindex = useCallback((id: string) => {
    setFiles((prev) =>
      prev.map((file) =>
        file.id === id
          ? { ...file, indexingStatus: 'processing', errorMessage: undefined }
          : file
      )
    );

    // Simulate vectorization process
    setTimeout(() => {
      setFiles((prev) =>
        prev.map((file) =>
          file.id === id ? { ...file, indexingStatus: 'completed' } : file
        )
      );
    }, 2500);
  }, []);

  // Delete file index
  const handleDelete = useCallback((id: string) => {
    setFiles((prev) => prev.filter((file) => file.id !== id));
  }, []);

  // Ingestion Simulator - single file
  const handleSimulateImport = useCallback(
    (name: string, format: 'pdf' | 'docx' | 'xlsx' | 'txt', virtualFolder: string, isCorrupt: boolean) => {
      const newId = `sim-${Date.now()}`;
      const randomSize = Math.floor(Math.random() * 20000000) + 1000000; // 1MB to 21MB

      const newFile: IndexedFile = {
        id: newId,
        name,
        sizeBytes: randomSize,
        format,
        virtualFolder,
        indexingStatus: 'processing',
        addedAt: new Date().toISOString(),
      };

      setFiles((prev) => [newFile, ...prev]);

      // Processing stage
      setTimeout(() => {
        setFiles((prev) =>
          prev.map((file) => {
            if (file.id === newId) {
              if (isCorrupt) {
                const reasons = [
                  'OCR Layer Required - PDF contains non-extractable image-only text layers',
                  'File decryption failure - Unsupported password-encrypted spreadsheet signature',
                  'UTF-8 Decoding failure - Invalid block boundary character sequence detected',
                ];
                const selectedReason = reasons[Math.floor(Math.random() * reasons.length)];
                return {
                  ...file,
                  indexingStatus: 'corrupt',
                  errorMessage: selectedReason,
                };
              } else {
                return {
                  ...file,
                  indexingStatus: 'completed',
                };
              }
            }
            return file;
          })
        );
      }, isCorrupt ? 1500 : 3000);
    },
    []
  );

  // Ingestion Simulator - batch
  const handleSimulateBatch = useCallback(() => {
    setIsSimulatingQueue(true);

    const batchFiles = [
      { name: 'architecture_diagrams_v4.pdf', format: 'pdf', folder: 'Engineering Docs', isCorrupt: false, delay: 1500 },
      { name: 'q3_sales_marketing_deck.docx', format: 'docx', folder: 'Marketing Strategy', isCorrupt: false, delay: 2800 },
      { name: 'broken_export_dump.xlsx', format: 'xlsx', folder: 'Product Manuals', isCorrupt: true, delay: 2000 },
      { name: 'readme_deployment.txt', format: 'txt', folder: 'Engineering Docs', isCorrupt: false, delay: 3800 },
    ] as const;

    batchFiles.forEach((item, index) => {
      const newId = `batch-${Date.now()}-${index}`;
      const randomSize = Math.floor(Math.random() * 15000000) + 500000; // 500KB to 15.5MB

      const newFile: IndexedFile = {
        id: newId,
        name: item.name,
        sizeBytes: randomSize,
        format: item.format,
        virtualFolder: item.folder,
        indexingStatus: 'processing',
        addedAt: new Date().toISOString(),
      };

      // Ingest sequentially in UI state
      setTimeout(() => {
        setFiles((prev) => [newFile, ...prev]);

        // Solve vectorization staggered
        setTimeout(() => {
          setFiles((prev) =>
            prev.map((file) => {
              if (file.id === newId) {
                if (item.isCorrupt) {
                  return {
                    ...file,
                    indexingStatus: 'corrupt',
                    errorMessage: 'Ingestion Error - Corrupted structural headers in spreadsheet file archive',
                  };
                } else {
                  return {
                    ...file,
                    indexingStatus: 'completed',
                  };
                }
              }
              return file;
            })
          );
        }, item.delay);
      }, index * 200);
    });

    // Reset simulator loader state after all batches finish queueing
    setTimeout(() => {
      setIsSimulatingQueue(false);
    }, 4500);
  }, []);

  // Compute live statistics based on state
  const stats: KnowledgeBaseStats = useMemo(() => {
    const completedFiles = files.filter((f) => f.indexingStatus === 'completed');
    const failedFiles = files.filter((f) => f.indexingStatus === 'corrupt');
    
    const diskSpaceUsedBytes = completedFiles.reduce((acc, f) => acc + f.sizeBytes, 0);
    const diskSpaceMaxBytes = 107374182400; // 100 GB in bytes

    return {
      totalProcessed: completedFiles.length,
      totalFailed: failedFiles.length,
      diskSpaceUsedBytes,
      diskSpaceMaxBytes,
    };
  }, [files]);

  // Compute folder metrics dynamically
  const folders: VirtualFolder[] = useMemo(() => {
    const folderNames = ['Engineering Docs', 'Legal & Compliance', 'Marketing Strategy', 'Product Manuals'];
    const colors = ['blue', 'emerald', 'purple', 'amber'];

    return folderNames.map((name, i) => {
      const folderFiles = files.filter((f) => f.virtualFolder === name);
      const completedFolderFiles = folderFiles.filter((f) => f.indexingStatus === 'completed');
      
      const fileCount = folderFiles.length;
      const sizeBytes = completedFolderFiles.reduce((acc, f) => acc + f.sizeBytes, 0);

      return {
        id: `folder-${i}`,
        name,
        fileCount,
        sizeBytes,
        color: colors[i % colors.length],
      };
    });
  }, [files]);

  const corruptFiles = useMemo(() => files.filter((f) => f.indexingStatus === 'corrupt'), [files]);

  return (
    <div className="space-y-8">
      {/* Premium dashboard header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-4xl font-extrabold text-gray-900 dark:text-white tracking-tight bg-gradient-to-r from-gray-950 to-gray-600 dark:from-white dark:to-gray-400 bg-clip-text text-transparent">
            Knowledge Base Explorer
          </h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 font-semibold">
            Monitor and manage vector index segments, disk storage, and ingest pipelines.
          </p>
        </div>

        <motion.a
          href="/editor"
          whileHover={{ x: -2 }}
          className="inline-flex items-center gap-2 px-4 py-2 text-sm font-bold text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white transition-colors"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" />
          </svg>
          Back to document editor
        </motion.a>
      </div>

      {/* Expandable Corrupt Files Warnings Banner */}
      <AlertPanel
        corruptFiles={corruptFiles}
        onReindex={handleReindex}
        onDelete={handleDelete}
      />

      {/* Dynamic Statistics Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <MetricCard
          title="Ingested Documents"
          value={stats.totalProcessed}
          description="Vectorized and similarity-searchable"
          trend={{ value: '12%', positive: true }}
          icon={
            <svg className="w-5 h-5 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75M21 12c0 1.268-.63 2.39-1.593 3.068a3.745 3.745 0 0 1-1.043 3.296 3.745 3.745 0 0 1-3.296 1.043A3.745 3.745 0 0 1 12 21c-1.268 0-2.39-.63-3.068-1.593a3.746 3.746 0 0 1-3.296-1.043 3.745 3.745 0 0 1-1.043-3.296A3.745 3.745 0 0 1 3 12c0-1.268.63-2.39 1.593-3.068a3.745 3.745 0 0 1 1.043-3.296 3.746 3.746 0 0 1 3.296-1.043A3.746 3.746 0 0 1 12 3c1.268 0 2.39.63 3.068 1.593a3.746 3.746 0 0 1 3.296 1.043 3.746 3.746 0 0 1 1.043 3.296A3.745 3.745 0 0 1 21 12Z" />
            </svg>
          }
        />

        <DiskSpaceMetric stats={stats} />

        <MetricCard
          title="Active System Alerts"
          value={stats.totalFailed}
          description="Failed documents requiring review"
          trend={{ value: '0%', positive: true }}
          className={stats.totalFailed > 0 ? 'border-red-500/20 bg-red-500/[0.01]' : ''}
          icon={
            <svg className={`w-5 h-5 ${stats.totalFailed > 0 ? 'text-red-500 animate-pulse' : 'text-gray-400'}`} fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
            </svg>
          }
        />
      </div>

      {/* Virtual Folders Grid */}
      <div>
        <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100 tracking-tight mb-4 flex items-center gap-2">
          <span>Virtual Directories</span>
          <span className="w-1.5 h-1.5 rounded-full bg-gray-300 dark:bg-gray-700" />
          <span className="text-xs text-gray-400 dark:text-gray-500 font-semibold uppercase">Category Filter</span>
        </h2>
        
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {folders.map((folder) => (
            <FolderCard
              key={folder.id}
              folder={folder}
              isSelected={selectedFolderId === folder.name}
              onClick={() => {
                setSelectedFolderId((prev) => (prev === folder.name ? null : folder.name));
              }}
            />
          ))}
        </div>
      </div>

      {/* Interactive Files list & Action controls */}
      <div className="grid grid-cols-1 gap-6">
        <FileTable
          files={files}
          onReindex={handleReindex}
          onDelete={handleDelete}
          selectedFolder={selectedFolderId}
        />
      </div>

      {/* Ingestion pipeline simulator controls */}
      <div className="grid grid-cols-1 gap-6">
        <ImportSimulator
          onSimulateImport={handleSimulateImport}
          onSimulateBatch={handleSimulateBatch}
          disabled={isSimulatingQueue}
        />
      </div>
    </div>
  );
}
