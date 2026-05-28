export interface Citation {
  chunkId: string;
  filePath: string;
  fileName: string;
  pageNumber: number | null;
  fragmentText: string;
  similarityScore: number;
}

export type CitationSimilarityTier = 'high' | 'medium' | 'low';

export function extractFileName(filePath: string): string {
  return filePath.split('/').pop() ?? filePath;
}

export function classifySimilarityTier(score: number): CitationSimilarityTier {
  if (score >= 0.85) return 'high';
  if (score >= 0.65) return 'medium';
  return 'low';
}

export function formatSimilarityPercentage(score: number): string {
  return `${Math.round(score * 100)}%`;
}
