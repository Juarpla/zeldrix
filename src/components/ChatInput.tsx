'use client';

import { useState, useRef, type FormEvent } from 'react';
import MediaDropZone from './MediaDropZone';
import MediaPreview from './MediaPreview';
import type { MediaFile } from '@/lib/multimodal';

interface ChatInputProps {
  onSend: (text: string, files: MediaFile[]) => void;
  disabled?: boolean;
}

export default function ChatInput({ onSend, disabled }: ChatInputProps) {
  const [text, setText] = useState('');
  const [attachedFiles, setAttachedFiles] = useState<MediaFile[]>([]);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleFilesDropped = (files: MediaFile[]) => {
    setAttachedFiles(prev => [...prev, ...files]);
  };

  const handleRemoveFile = (id: string) => {
    setAttachedFiles(prev => prev.filter(f => f.id !== id));
  };

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    const trimmedText = text.trim();
    if (!trimmedText && attachedFiles.length === 0) return;
    onSend(trimmedText, attachedFiles);
    setText('');
    setAttachedFiles([]);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as unknown as FormEvent);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      {/* Attached files preview */}
      {attachedFiles.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {attachedFiles.map(file => (
            <MediaPreview key={file.id} file={file} onRemove={handleRemoveFile} />
          ))}
        </div>
      )}

      {/* Drop zone */}
      <MediaDropZone onFilesDropped={handleFilesDropped} />

      {/* Text input */}
      <div className="flex gap-2">
        <textarea
          ref={textareaRef}
          value={text}
          onChange={e => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type your message... (Shift+Enter for new line)"
          disabled={disabled}
          className="flex-1 resize-none rounded-lg border border-gray-300 dark:border-gray-600 px-4 py-3 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
          rows={2}
        />
        <button
          type="submit"
          disabled={disabled || (!text.trim() && attachedFiles.length === 0)}
          className="px-6 py-3 rounded-lg bg-blue-500 text-white font-medium hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          Send
        </button>
      </div>
    </form>
  );
}
