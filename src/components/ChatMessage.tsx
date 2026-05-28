'use client';

import type { ContentPart, MultimodalMessage, Citation } from '@/lib/multimodal';
import CitationBadge from './CitationBadge';
import StructuredResultsTable, { parseStructuredResultsTableJson } from './StructuredResultsTable';

interface ChatMessageProps {
  message: MultimodalMessage;
  onCitationClick?: (citation: Citation) => void;
}

function renderPart(part: ContentPart, i: number, isUser: boolean) {
  if (part.type === 'text') {
    const structuredTable = isUser ? null : parseStructuredResultsTableJson(part.text);

    if (structuredTable) {
      return <StructuredResultsTable key={i} table={structuredTable} />;
    }

    return <p key={i} className="text-sm leading-relaxed">{part.text}</p>;
  }
  if (part.type === 'image_url') {
    return (
      <div key={i} className="mt-2 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-800">
        <img src={part.image_url.url} alt="Attached" className="max-w-full max-h-64 object-contain" />
      </div>
    );
  }
  if (part.type === 'audio_url') {
    return (
      <div key={i} className="mt-2 p-3 bg-gray-100 dark:bg-gray-800 rounded-lg">
        <audio controls className="w-full h-8">
          <source src={part.audio_url.url} />
        </audio>
      </div>
    );
  }
  return null;
}

export default function ChatMessage({ message, onCitationClick }: ChatMessageProps) {
  const isUser = message.role === 'user';
  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div className={`max-w-[80%] rounded-2xl px-4 py-3 ${isUser ? 'bg-blue-500 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100'}`}>
        {message.content.map((part, i) => renderPart(part, i, isUser))}
        {!isUser && message.citations && message.citations.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-2 pt-2 border-t border-gray-300 dark:border-gray-600">
            {message.citations.map((citation, index) => (
              <CitationBadge
                key={citation.chunkId}
                citation={citation}
                index={index}
                onClick={onCitationClick ?? (() => {})}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
