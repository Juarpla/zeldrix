import type { Citation } from './citation-types';

export type { Citation };
export type ContentPartType = 'text' | 'image_url' | 'audio_url';

export interface TextContentPart {
  type: 'text';
  text: string;
}

export interface ImageUrlContentPart {
  type: 'image_url';
  image_url: {
    url: string;
  };
}

export interface AudioUrlContentPart {
  type: 'audio_url';
  audio_url: {
    url: string;
  };
}

export type ContentPart = TextContentPart | ImageUrlContentPart | AudioUrlContentPart;

export interface MultimodalMessage {
  role: 'user' | 'assistant';
  content: ContentPart[];
  citations?: Citation[];
}

export interface MediaFile {
  id: string;
  type: 'image' | 'audio';
  name: string;
  size: number;
  mimeType: string;
  dataUrl: string; // base64 data URL
}

// MIME types supported for multimodal input
export const SUPPORTED_IMAGE_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
export const SUPPORTED_AUDIO_TYPES = ['audio/wav', 'audio/mpeg', 'audio/ogg', 'audio/flac'];
export const SUPPORTED_TYPES = [...SUPPORTED_IMAGE_TYPES, ...SUPPORTED_AUDIO_TYPES];

// Check if a MIME type is supported
export function isSupportedType(mimeType: string): boolean {
  return SUPPORTED_TYPES.includes(mimeType);
}

// Check if a MIME type is an image
export function isImageType(mimeType: string): boolean {
  return SUPPORTED_IMAGE_TYPES.includes(mimeType);
}

// Check if a MIME type is audio
export function isAudioType(mimeType: string): boolean {
  return SUPPORTED_AUDIO_TYPES.includes(mimeType);
}

// Read a file and convert it to a base64 data URL
export function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      if (typeof reader.result === 'string') {
        resolve(reader.result);
      } else {
        reject(new Error('Failed to read file as data URL'));
      }
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

// Process a file list into MediaFile objects
export async function processFiles(files: FileList | File[]): Promise<MediaFile[]> {
  const fileArray = Array.from(files);
  const results: MediaFile[] = [];

  for (const file of fileArray) {
    if (!isSupportedType(file.type)) {
      console.warn(`Unsupported file type: ${file.type}`);
      continue;
    }

    try {
      const dataUrl = await readFileAsDataUrl(file);
      results.push({
        id: crypto.randomUUID(),
        type: isImageType(file.type) ? 'image' : 'audio',
        name: file.name,
        size: file.size,
        mimeType: file.type,
        dataUrl,
      });
    } catch (error) {
      console.error(`Failed to process file ${file.name}:`, error);
    }
  }

  return results;
}

// Convert MediaFile to ContentPart
export function mediaFileToContentPart(file: MediaFile): ContentPart {
  if (file.type === 'image') {
    return {
      type: 'image_url',
      image_url: {
        url: file.dataUrl,
      },
    };
  } else {
    return {
      type: 'audio_url',
      audio_url: {
        url: file.dataUrl,
      },
    };
  }
}

// Format file size for display
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// Format duration for audio (assuming seconds)
export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}