'use client';

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import ChatInput from './ChatInput';
import ChatMessage from './ChatMessage';
import CitationDrawer from './CitationDrawer';
import { getSidecarStatus } from '@/lib/aiService';
import type { MediaFile, MultimodalMessage, ContentPart } from '@/lib/multimodal';
import { mediaFileToContentPart } from '@/lib/multimodal';
import type { Citation } from '@/lib/citation-types';

interface RetrievalResult {
  id: string;
  text: string;
  file_path: string;
  similarity: number;
  page_number: number | null;
}

export default function MultimodalChat() {
  const [messages, setMessages] = useState<MultimodalMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeCitation, setActiveCitation] = useState<Citation | null>(null);
  const [sidecarRunning, setSidecarRunning] = useState(false);
  const [checkingSidecar, setCheckingSidecar] = useState(true);

  useEffect(() => {
    async function checkStatus() {
      try {
        const status = await getSidecarStatus();
        setSidecarRunning(status.running);
      } catch {
        setSidecarRunning(false);
      } finally {
        setCheckingSidecar(false);
      }
    }

    checkStatus();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleSend = useCallback(async (text: string, files: MediaFile[]) => {
    // Build user message with text and attached files
    const userContent: ContentPart[] = [];
    if (text) {
      userContent.push({ type: 'text', text });
    }
    for (const file of files) {
      userContent.push(mediaFileToContentPart(file));
    }

    if (userContent.length === 0) return;

    const userMessage: MultimodalMessage = {
      role: 'user',
      content: userContent,
    };

    // Add user message to state
    setMessages(prev => [...prev, userMessage]);
    setError(null);
    setIsLoading(true);

    try {
      // Step 1: Retrieve context from vector DB for semantic traceability (RAG)
      let retrievedContext: RetrievalResult[] = [];
      let citations: Citation[] = [];

      if (text) {
        try {
          retrievedContext = await invoke<RetrievalResult[]>('retrieve_relevant_context', {
            query: text,
            limit: 5,
          });

          citations = retrievedContext.map((res) => ({
            chunkId: res.id,
            filePath: res.file_path,
            fileName: res.file_path.split('/').pop() ?? res.file_path,
            pageNumber: res.page_number,
            fragmentText: res.text,
            similarityScore: res.similarity,
          }));
        } catch (retrievalError) {
          console.error('RAG Retrieval failed, continuing with direct generation:', retrievalError);
        }
      }

      // Step 2: Inject retrieved context into Gemma 4 query (Prompt Packing Pipeline)
      let userContentForBackend = [...userContent];
      if (text && retrievedContext.length > 0) {
        try {
          const packed = await invoke<{ formatted_prompt: string }>('format_inference_prompt', {
            query: text,
            documents: retrievedContext,
            maxTokens: 4000,
          });

          userContentForBackend = [
            { type: 'text', text: packed.formatted_prompt },
            ...userContent.filter((part) => part.type !== 'text'),
          ];
        } catch (packerError) {
          console.error('Prompt packing failed, using raw query text:', packerError);
        }
      }

      // Send to backend - concatenate all messages for context
      const userMessageForBackend: MultimodalMessage = {
        role: 'user',
        content: userContentForBackend,
      };
      const allMessages = [...messages, userMessageForBackend];

      const response = await invoke<string>('chat_complete_multimodal', {
        messages: allMessages,
      });

      // Add assistant response carrying citation metadata
      const assistantMessage: MultimodalMessage = {
        role: 'assistant',
        content: [{ type: 'text', text: response }],
        citations: citations.length > 0 ? citations : undefined,
      };
      setMessages(prev => [...prev, assistantMessage]);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Chat error:', err);
    } finally {
      setIsLoading(false);
    }
  }, [messages]);

  return (
    <div className="flex flex-col h-full relative overflow-hidden">
      {/* Header */}
      <div className="flex-shrink-0 px-6 py-4 border-b border-gray-200 dark:border-gray-700">
        <h1 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
          Multimodal Chat
        </h1>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Drag & drop images or audio to analyze with Gemma 4
        </p>
      </div>

      {/* Sidecar Status Banner */}
      {!sidecarRunning && !checkingSidecar && (
        <div className="flex-shrink-0 mx-6 mt-3 rounded-xl border border-amber-500/20 bg-amber-500/[0.04] px-4 py-3 flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-amber-100 dark:bg-amber-950/30 text-amber-600 dark:text-amber-400 flex items-center justify-center shrink-0 animate-pulse">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" />
            </svg>
          </div>
          <div>
            <p className="text-xs font-semibold text-amber-700 dark:text-amber-300">AI engine not running</p>
            <p className="text-[10px] text-amber-600/70 dark:text-amber-400/60">Start the model from the system panel to enable chat.</p>
          </div>
        </div>
      )}

      {checkingSidecar && (
        <div className="flex-shrink-0 mx-6 mt-3 rounded-xl border border-gray-200/50 dark:border-gray-700/50 bg-gray-50/50 dark:bg-gray-800/30 px-4 py-3 flex items-center gap-3">
          <svg className="animate-spin h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
          <p className="text-xs text-gray-500 dark:text-gray-400">Checking AI engine status…</p>
        </div>
      )}

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
        {messages.length === 0 && !isLoading && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-500 dark:text-gray-400">
              <svg className="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
              <p>Start a conversation by sending a message</p>
              <p className="text-sm mt-1">You can attach images or audio files</p>
            </div>
          </div>
        )}

        {messages.map((message, index) => (
          <ChatMessage
            key={index}
            message={message}
            onCitationClick={setActiveCitation}
          />
        ))}

        {isLoading && (
          <div className="flex justify-start">
            <div className="bg-gray-200 dark:bg-gray-700 rounded-2xl px-4 py-3">
              <div className="flex gap-2">
                <div className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" />
                <div className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
                <div className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style={{ animationDelay: '0.4s' }} />
              </div>
            </div>
          </div>
        )}

        {error && (
          <div className="flex justify-start">
            <div className="bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded-2xl px-4 py-3">
              <p className="text-sm">Error: {error}</p>
            </div>
          </div>
        )}
      </div>

      {/* Input */}
      <div className="flex-shrink-0 px-6 py-4 border-t border-gray-200 dark:border-gray-700">
        <ChatInput onSend={handleSend} disabled={isLoading || !sidecarRunning || checkingSidecar} />
      </div>

      {/* Slide-out Citation Panel */}
      <CitationDrawer
        citation={activeCitation}
        onClose={() => setActiveCitation(null)}
      />
    </div>
  );
}
