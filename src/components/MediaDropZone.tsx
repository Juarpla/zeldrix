'use client';

import { useCallback, useState } from 'react';
import { processFiles, SUPPORTED_TYPES } from '@/lib/multimodal';
import type { MediaFile } from '@/lib/multimodal';

interface MediaDropZoneProps {
  onFilesDropped: (files: MediaFile[]) => void;
  children?: React.ReactNode;
}

export default function MediaDropZone({ onFilesDropped, children }: MediaDropZoneProps) {
  const [isDragOver, setIsDragOver] = useState(false);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);

      const files = e.dataTransfer.files;
      if (files.length > 0) {
        const processedFiles = await processFiles(files);
        if (processedFiles.length > 0) {
          onFilesDropped(processedFiles);
        }
      }
    },
    [onFilesDropped]
  );

  const handleFileSelect = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (files && files.length > 0) {
        const processedFiles = await processFiles(files);
        if (processedFiles.length > 0) {
          onFilesDropped(processedFiles);
        }
      }
      // Reset input so the same file can be selected again
      e.target.value = '';
    },
    [onFilesDropped]
  );

  return (
    <div
      className={`relative border-2 border-dashed rounded-lg p-4 transition-colors ${
        isDragOver
          ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
          : 'border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500'
      }`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <input
        type="file"
        id="file-input"
        className="hidden"
        accept={SUPPORTED_TYPES.join(',')}
        multiple
        onChange={handleFileSelect}
      />
      <label
        htmlFor="file-input"
        className="flex flex-col items-center justify-center cursor-pointer text-gray-500 dark:text-gray-400"
      >
        {children || (
          <>
            <svg
              className="w-8 h-8 mb-2"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
              />
            </svg>
            <span className="text-sm">
              Drop image or audio files here, or click to select
            </span>
            <span className="text-xs mt-1 text-gray-400 dark:text-gray-500">
              PNG, JPEG, GIF, WEBP, WAV, MP3, OGG, FLAC
            </span>
          </>
        )}
      </label>
    </div>
  );
}