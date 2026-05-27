export interface IndexedFile {
  id: string;
  name: string;
  sizeBytes: number;
  format: 'pdf' | 'docx' | 'xlsx' | 'txt';
  virtualFolder: string;
  indexingStatus: 'completed' | 'processing' | 'corrupt';
  errorMessage?: string;
  addedAt: string;
}

export interface VirtualFolder {
  id: string;
  name: string;
  fileCount: number;
  sizeBytes: number;
  color: string; // Tailwind color class prefix (e.g. 'blue', 'emerald')
}

export interface KnowledgeBaseStats {
  totalProcessed: number;
  totalFailed: number;
  diskSpaceUsedBytes: number;
  diskSpaceMaxBytes: number;
}
