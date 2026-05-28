'use client';

import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Citation } from '@/lib/citation-types';
import { formatSimilarityPercentage, classifySimilarityTier } from '@/lib/citation-types';

interface CitationDrawerProps {
  citation: Citation | null;
  onClose: () => void;
}

interface FragmentState {
  isLoading: boolean;
  fragmentText: string;
  error: string | null;
}

const SIMILARITY_TIER_BADGE_STYLES = {
  high: 'citation-drawer__score-badge--high',
  medium: 'citation-drawer__score-badge--medium',
  low: 'citation-drawer__score-badge--low',
};

function buildPageReference(pageNumber: number | null): string | null {
  if (pageNumber === null) return null;
  return `Página ${pageNumber}`;
}

export default function CitationDrawer({ citation, onClose }: CitationDrawerProps) {
  const drawerRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const [fragmentState, setFragmentState] = useState<FragmentState>({
    isLoading: false,
    fragmentText: '',
    error: null,
  });

  useEffect(() => {
    if (!citation) return;

    setFragmentState({ isLoading: true, fragmentText: '', error: null });

    invoke<{ text: string }>('get_citation_fragment', { chunkId: citation.chunkId })
      .then((result) => {
        setFragmentState({ isLoading: false, fragmentText: result.text, error: null });
      })
      .catch((err: unknown) => {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setFragmentState({
          isLoading: false,
          fragmentText: citation.fragmentText,
          error: `No se pudo verificar el fragmento actualizado: ${errorMessage}`,
        });
      });

    closeButtonRef.current?.focus();
  }, [citation]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape' && citation) {
        onClose();
      }
    }

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [citation, onClose]);

  useEffect(() => {
    if (!citation || !drawerRef.current) return;

    const focusableSelectors = [
      'button',
      'a[href]',
      'input',
      'textarea',
      '[tabindex]:not([tabindex="-1"])',
    ].join(',');

    const drawer = drawerRef.current;
    const focusableElements = Array.from(drawer.querySelectorAll<HTMLElement>(focusableSelectors));

    function trapFocus(event: KeyboardEvent) {
      if (event.key !== 'Tab' || focusableElements.length === 0) return;
      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      if (event.shiftKey && document.activeElement === firstElement) {
        event.preventDefault();
        lastElement.focus();
      } else if (!event.shiftKey && document.activeElement === lastElement) {
        event.preventDefault();
        firstElement.focus();
      }
    }

    drawer.addEventListener('keydown', trapFocus);
    return () => drawer.removeEventListener('keydown', trapFocus);
  }, [citation]);

  const isOpen = citation !== null;
  const pageReference = citation ? buildPageReference(citation.pageNumber) : null;
  const scoreTier = citation ? classifySimilarityTier(citation.similarityScore) : 'low';
  const scoreBadgeClass = SIMILARITY_TIER_BADGE_STYLES[scoreTier];

  return (
    <>
      {isOpen && (
        <div
          className="citation-drawer__overlay"
          aria-hidden="true"
          onClick={onClose}
        />
      )}
      <div
        ref={drawerRef}
        id="citation-drawer"
        className={`citation-drawer ${isOpen ? 'citation-drawer--open' : ''}`}
        role="dialog"
        aria-modal="true"
        aria-label="Fragmento de documento fuente"
        aria-hidden={!isOpen}
      >
        <div className="citation-drawer__header">
          <div className="citation-drawer__title-group">
            <span className="citation-drawer__icon" aria-hidden="true">📄</span>
            <h2 className="citation-drawer__title">
              {citation?.fileName ?? 'Fuente documental'}
            </h2>
          </div>
          <button
            ref={closeButtonRef}
            id="citation-drawer-close"
            className="citation-drawer__close"
            onClick={onClose}
            aria-label="Cerrar panel de citación"
          >
            ✕
          </button>
        </div>

        {citation && (
          <div className="citation-drawer__body">
            <div className="citation-drawer__meta">
              <p className="citation-drawer__filepath" title={citation.filePath}>
                {citation.filePath}
              </p>
              <div className="citation-drawer__meta-row">
                {pageReference && (
                  <span className="citation-drawer__page-badge">
                    📖 {pageReference}
                  </span>
                )}
                <span className={`citation-drawer__score-badge ${scoreBadgeClass}`}>
                  {formatSimilarityPercentage(citation.similarityScore)} de similitud
                </span>
              </div>
            </div>

            <div className="citation-drawer__fragment-section">
              <p className="citation-drawer__fragment-label">Fragmento original:</p>
              {fragmentState.isLoading ? (
                <div className="citation-drawer__loading">
                  <span className="citation-drawer__spinner" aria-hidden="true" />
                  <span>Cargando fragmento…</span>
                </div>
              ) : (
                <pre className="citation-drawer__fragment-text">
                  {fragmentState.fragmentText || citation.fragmentText}
                </pre>
              )}
              {fragmentState.error && (
                <p className="citation-drawer__error">{fragmentState.error}</p>
              )}
            </div>
          </div>
        )}
      </div>
    </>
  );
}
