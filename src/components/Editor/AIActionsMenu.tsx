"use client";

import { useEffect, useState, useRef, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Editor } from "@tiptap/react";
import type { AIActionType } from "@/lib/aiService";
import "./ai-menu.css";

interface AIActionsMenuProps {
  editor: Editor;
  onAction?: (action: AIActionType, selectedText: string) => void;
  isLoading?: boolean;
}

interface AIAction {
  id: AIActionType;
  label: string;
  icon: React.ReactNode;
}

const AI_ACTIONS: AIAction[] = [
  {
    id: "formal" as AIActionType,
    label: "Más formal",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 2L2 7l10 5 10-5-10-5z" />
        <path d="M2 17l10 5 10-5" />
        <path d="M2 12l10 5 10-5" />
      </svg>
    ),
  },
  {
    id: "style" as AIActionType,
    label: "Corregir estilo",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M12 20h9" />
        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
      </svg>
    ),
  },
  {
    id: "translate" as AIActionType,
    label: "Traducir",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M5 8l6 6" />
        <path d="M4 14l6-6 2-3" />
        <path d="M2 5h12" />
        <path d="M7 2h1" />
        <path d="M22 22l-5-10-5 10" />
        <path d="M14 18h6" />
      </svg>
    ),
  },
  {
    id: "summarize" as AIActionType,
    label: "Resumir",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <path d="M14 2v6h6" />
        <line x1="16" y1="13" x2="8" y2="13" />
        <line x1="16" y1="17" x2="8" y2="17" />
        <line x1="10" y1="9" x2="8" y2="9" />
      </svg>
    ),
  },
];

export default function AIActionsMenu({
  editor,
  onAction,
  isLoading = false,
}: AIActionsMenuProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const [selectedText, setSelectedText] = useState("");
  const menuRef = useRef<HTMLDivElement>(null);

  const updateMenuPosition = useCallback(() => {
    const { selection } = editor.state;
    const { from, to } = selection;

    if (from === to) {
      setIsVisible(false);
      return;
    }

    const text = editor.state.doc.textBetween(from, to, " ");
    if (!text.trim()) {
      setIsVisible(false);
      return;
    }

    setSelectedText(text);

    // Calculate position based on selection coordinates
    const coords = editor.view.coordsAtPos(from);
    const endCoords = editor.view.coordsAtPos(to);
    const editorRect = editor.view.dom.getBoundingClientRect();

    // Position menu below the selection, centered
    const top = endCoords.bottom - editorRect.top + 12;
    const left = (coords.left + endCoords.left) / 2 - editorRect.left;

    setPosition({ top, left });
    setIsVisible(true);
  }, [editor]);

  useEffect(() => {
    const handleSelectionUpdate = () => {
      updateMenuPosition();
    };

    editor.on("selectionUpdate", handleSelectionUpdate);

    return () => {
      editor.off("selectionUpdate", handleSelectionUpdate);
    };
  }, [editor, updateMenuPosition]);

  // Hide menu on click outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        // Check if click is on editor content
        const editorElement = editor.view.dom;
        if (!editorElement.contains(event.target as Node)) {
          setIsVisible(false);
        }
      }
    };

    const handleEditorBlur = () => {
      // Delay to allow click events on menu buttons
      setTimeout(() => {
        const { selection } = editor.state;
        if (selection.from === selection.to) {
          setIsVisible(false);
        }
      }, 150);
    };

    document.addEventListener("mousedown", handleClickOutside);
    editor.on("blur", handleEditorBlur);

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      editor.off("blur", handleEditorBlur);
    };
  }, [editor]);

  const handleAction = useCallback(
    async (actionId: AIActionType) => {
      if (!selectedText || isLoading) return;
      onAction?.(actionId, selectedText);
    },
    [selectedText, onAction, isLoading]
  );

  return (
    <AnimatePresence>
      {isVisible && (
        <motion.div
          ref={menuRef}
          className="ai-actions-menu"
          style={{
            top: position.top,
            left: position.left,
            transform: "translateX(-50%)",
          }}
          initial={{ opacity: 0, scale: 0.9, y: 8 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: 8 }}
          transition={{
            duration: 0.2,
            ease: [0.4, 0, 0.2, 1],
          }}
          onMouseDown={(e) => e.preventDefault()}
        >
          {isLoading && (
            <div className="ai-action-button loading" style={{ padding: "6px 16px" }}>
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                style={{ animation: "ai-spin 1s linear infinite" }}
              >
                <path d="M12 2v4m0 12v4M4.93 4.93l2.83 2.83m8.48 8.48l2.83 2.83M2 12h4m12 0h4M4.93 19.07l2.83-2.83m8.48-8.48l2.83-2.83" />
              </svg>
              <span>Procesando...</span>
            </div>
          )}
          {!isLoading &&
            AI_ACTIONS.map((action, index) => (
              <motion.button
                key={action.id}
                className="ai-action-button"
                onClick={() => handleAction(action.id)}
                initial={{ opacity: 0, y: 4 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{
                  delay: index * 0.05,
                  duration: 0.15,
                }}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
              >
                {action.icon}
                <span>{action.label}</span>
              </motion.button>
            ))}
        </motion.div>
      )}
    </AnimatePresence>
  );
}