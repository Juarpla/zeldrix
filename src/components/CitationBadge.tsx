'use client';

import type { Citation, CitationSimilarityTier } from '@/lib/citation-types';
import { classifySimilarityTier, formatSimilarityPercentage } from '@/lib/citation-types';

interface CitationBadgeProps {
  citation: Citation;
  index: number;
  onClick: (citation: Citation) => void;
}

const SIMILARITY_TIER_STYLES: Record<CitationSimilarityTier, string> = {
  high: 'citation-badge--high',
  medium: 'citation-badge--medium',
  low: 'citation-badge--low',
};

function buildBadgeLabel(citation: Citation): string {
  if (citation.pageNumber !== null) {
    return `${citation.fileName} – Pág. ${citation.pageNumber}`;
  }
  return citation.fileName;
}

function buildTooltipText(citation: Citation): string {
  const similarity = formatSimilarityPercentage(citation.similarityScore);
  return `${citation.filePath}\nSimilaridad: ${similarity}`;
}

export default function CitationBadge({ citation, index, onClick }: CitationBadgeProps) {
  const tier = classifySimilarityTier(citation.similarityScore);
  const tierClass = SIMILARITY_TIER_STYLES[tier];
  const label = buildBadgeLabel(citation);
  const tooltip = buildTooltipText(citation);

  function handleKeyDown(event: React.KeyboardEvent<HTMLSpanElement>) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onClick(citation);
    }
  }

  return (
    <span
      id={`citation-badge-${citation.chunkId}-${index}`}
      className={`citation-badge ${tierClass}`}
      role="button"
      tabIndex={0}
      title={tooltip}
      aria-label={`Ver fuente: ${label}`}
      onClick={() => onClick(citation)}
      onKeyDown={handleKeyDown}
    >
      <span className="citation-badge__icon" aria-hidden="true">📄</span>
      <span className="citation-badge__label">{label}</span>
    </span>
  );
}
