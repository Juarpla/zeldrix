"use client";

import { useEditor, EditorContent, type Editor, Extension } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import Typography from "@tiptap/extension-typography";
import CharacterCount from "@tiptap/extension-character-count";
import Link from "@tiptap/extension-link";
import { useEffect, useState, useRef, useCallback } from "react";
import { transformText, type AIActionType } from "@/lib/aiService";
import "./editor.css";
import "./toolbar.css";
import AIActionsMenu from "./AIActionsMenu";
import { AIShimmer } from "./AIShimmer";

interface TypographyEditorProps {
  content?: string;
  onChange?: (content: string) => void;
  onFocus?: () => void;
  onBlur?: () => void;
  placeholder?: string;
  className?: string;
}

interface FloatingToolbarProps {
  editor: Editor;
}

function ToolbarButton({
  onClick,
  isActive,
  title,
  children,
}: {
  onClick: () => void;
  isActive?: boolean;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onMouseDown={(e) => {
        e.preventDefault();
        onClick();
      }}
      title={title}
      className={`toolbar-button ${isActive ? "active" : ""}`}
    >
      {children}
    </button>
  );
}

function FloatingToolbar({ editor }: FloatingToolbarProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const toolbarRef = useRef<HTMLDivElement>(null);

  const updateToolbar = useCallback(() => {
    const { selection } = editor.state;
    const { from, to } = selection;

    if (from === to) {
      setIsVisible(false);
      return;
    }

    // Get the selection coordinates
    const coords = editor.view.coordsAtPos(from);
    const editorRect = editor.view.dom.getBoundingClientRect();

    const top = coords.bottom - editorRect.top + 8;
    const left = coords.left - editorRect.left;

    setPosition({ top, left });
    setIsVisible(true);
  }, [editor]);

  useEffect(() => {
    editor.on("selectionUpdate", updateToolbar);
    editor.on("blur", () => setIsVisible(false));
    return () => {
      editor.off("selectionUpdate", updateToolbar);
    };
  }, [editor, updateToolbar]);

  if (!isVisible) return null;

  return (
    <div
      ref={toolbarRef}
      className="floating-toolbar"
      style={{ top: position.top, left: position.left }}
    >
      <ToolbarButton
        onClick={() => editor.chain().focus().toggleBold().run()}
        isActive={editor.isActive("bold")}
        title="Negrita (Ctrl+B)"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5">
          <path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
          <path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" />
        </svg>
      </ToolbarButton>

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleItalic().run()}
        isActive={editor.isActive("italic")}
        title="Cursiva (Ctrl+I)"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="19" y1="4" x2="10" y2="4" />
          <line x1="14" y1="20" x2="5" y2="20" />
          <line x1="15" y1="4" x2="9" y2="20" />
        </svg>
      </ToolbarButton>

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleCode().run()}
        isActive={editor.isActive("code")}
        title="Código inline"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <polyline points="16 18 22 12 16 6" />
          <polyline points="8 6 2 12 8 18" />
        </svg>
      </ToolbarButton>

      <div className="toolbar-divider" />

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleHeading({ level: 1 }).run()}
        isActive={editor.isActive("heading", { level: 1 })}
        title="Título 1"
      >
        <span className="heading-label">H1</span>
      </ToolbarButton>

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()}
        isActive={editor.isActive("heading", { level: 2 })}
        title="Título 2"
      >
        <span className="heading-label">H2</span>
      </ToolbarButton>

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleHeading({ level: 3 }).run()}
        isActive={editor.isActive("heading", { level: 3 })}
        title="Título 3"
      >
        <span className="heading-label">H3</span>
      </ToolbarButton>

      <div className="toolbar-divider" />

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleBulletList().run()}
        isActive={editor.isActive("bulletList")}
        title="Lista con puntos"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="8" y1="6" x2="21" y2="6" />
          <line x1="8" y1="12" x2="21" y2="12" />
          <line x1="8" y1="18" x2="21" y2="18" />
          <circle cx="4" cy="6" r="1" fill="currentColor" />
          <circle cx="4" cy="12" r="1" fill="currentColor" />
          <circle cx="4" cy="18" r="1" fill="currentColor" />
        </svg>
      </ToolbarButton>

      <ToolbarButton
        onClick={() => editor.chain().focus().toggleOrderedList().run()}
        isActive={editor.isActive("orderedList")}
        title="Lista numerada"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="10" y1="6" x2="21" y2="6" />
          <line x1="10" y1="12" x2="21" y2="12" />
          <line x1="10" y1="18" x2="21" y2="18" />
          <text x="3" y="7" fontSize="6" fill="currentColor" fontWeight="600">1</text>
          <text x="3" y="13" fontSize="6" fill="currentColor" fontWeight="600">2</text>
          <text x="3" y="19" fontSize="6" fill="currentColor" fontWeight="600">3</text>
        </svg>
      </ToolbarButton>

      <ToolbarButton
        onClick={() => {
          const url = window.prompt("Enter URL:");
          if (url) {
            editor.chain().focus().setLink({ href: url }).run();
          }
        }}
        isActive={editor.isActive("link")}
        title="Insertar enlace (Ctrl+K)"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
          <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
        </svg>
      </ToolbarButton>
    </div>
  );
}

const Document = Extension.create({
  name: "document",
});

export default function TypographyEditor({
  content = "",
  onChange,
  onFocus,
  onBlur,
  placeholder = "Escribe algo...",
  className = "",
}: TypographyEditorProps) {
  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: {
          levels: [1, 2, 3],
        },
        bulletList: {
          HTMLAttributes: {
            class: "list-disc",
          },
        },
        orderedList: {
          HTMLAttributes: {
            class: "list-decimal",
          },
        },
        codeBlock: {
          HTMLAttributes: {
            class: "code-block",
          },
        },
      }),
      Placeholder.configure({
        placeholder: ({ node }) => {
          if (node.type.name === "heading") {
            return "Título...";
          }
          return placeholder;
        },
        showOnlyWhenEditable: true,
        showOnlyCurrent: true,
      }),
      Typography,
      CharacterCount,
      Link.configure({
        openOnClick: false,
        HTMLAttributes: {
          class: "editor-link",
        },
      }),
      Document,
      AIShimmer,
    ],
    content,
    editorProps: {
      attributes: {
        class: "editor-content",
      },
    },
    onUpdate: ({ editor }) => {
      onChange?.(editor.getHTML());
    },
    onFocus: () => {
      onFocus?.();
    },
    onBlur: () => {
      onBlur?.();
    },
  });

  // Handle keyboard shortcuts
  useEffect(() => {
    if (!editor) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl/Cmd + K for link
      if ((event.metaKey || event.ctrlKey) && event.key === "k") {
        event.preventDefault();
        const url = window.prompt("Enter URL:");
        if (url) {
          editor.chain().focus().setLink({ href: url }).run();
        }
      }
    };

    const editorElement = editor.view.dom;
    editorElement.addEventListener("keydown", handleKeyDown);

    return () => {
      editorElement.removeEventListener("keydown", handleKeyDown);
    };
  }, [editor]);

  const wordCount = editor?.storage.characterCount.words() ?? 0;
  const charCount = editor?.storage.characterCount.characters() ?? 0;
  const [isAILoading, setIsAILoading] = useState(false);
  const [aiError, setAiError] = useState<string | null>(null);

  const handleAIAction = useCallback(
    async (action: AIActionType, selectedText: string) => {
      if (!editor) return;

      const { from, to } = editor.state.selection;

      // 1. Immediately apply the shimmer mark on the selection during loading
      editor.chain().focus().setTextSelection({ from, to }).setMark("aiShimmer").run();

      setIsAILoading(true);
      setAiError(null);

      try {
        const response = await transformText(action, selectedText);

        if (response.success && response.result) {
          // 2. Delete the selected text
          editor.chain().focus().deleteRange({ from, to }).run();

          // 3. Simular renderizado progresivo de tokens (typewriter)
          const resultText = response.result;
          // Split by words/whitespace to stream as word tokens
          const tokens = resultText.split(/(\s+)/);
          let currentPos = from;
          let tokenIndex = 0;

          const streamInterval = setInterval(() => {
            if (!editor) {
              clearInterval(streamInterval);
              setIsAILoading(false);
              return;
            }

            if (tokenIndex < tokens.length) {
              const token = tokens[tokenIndex];
              if (token) {
                // Insert token
                editor.chain().insertContentAt(currentPos, token).run();
                
                // Keep the shimmer mark active on the inserted range so far
                editor.chain().setTextSelection({ from, to: currentPos + token.length }).setMark("aiShimmer").run();
                
                currentPos += token.length;
              }
              tokenIndex++;
            } else {
              clearInterval(streamInterval);
              // 4. Remove shimmer mark once the last token is rendered
              editor.chain().focus().setTextSelection({ from, to: currentPos }).unsetMark("aiShimmer").run();
              // Reset selection/cursor to the end
              editor.chain().setTextSelection(currentPos).run();
              setIsAILoading(false);
            }
          }, 35); // 35ms per word/whitespace token for a smooth visual effect

        } else if (response.error) {
          // Remove shimmer mark on error
          editor.chain().focus().setTextSelection({ from, to }).unsetMark("aiShimmer").run();
          setAiError(response.error);
          setIsAILoading(false);
          setTimeout(() => setAiError(null), 3000);
        }
      } catch (error) {
        // Remove shimmer mark on error
        editor.chain().focus().setTextSelection({ from, to }).unsetMark("aiShimmer").run();
        const errorMessage =
          error instanceof Error ? error.message : "Error desconocido";
        setAiError(errorMessage);
        setIsAILoading(false);
        setTimeout(() => setAiError(null), 3000);
      }
    },
    [editor]
  );

  if (!editor) {
    return null;
  }

  return (
    <div className={`typography-editor ${className}`}>
      <div className="editor-container">
        <FloatingToolbar editor={editor} />
        <AIActionsMenu
          editor={editor}
          onAction={handleAIAction}
          isLoading={isAILoading}
        />
        <div className="editor-wrapper">
          <EditorContent editor={editor} />
        </div>
        {aiError && (
          <div className="ai-error-toast">
            <span>{aiError}</span>
            <button onClick={() => setAiError(null)} aria-label="Cerrar">
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        )}
      </div>
      <div className="editor-footer">
        <span className="word-count">
          {wordCount.toLocaleString()} {wordCount === 1 ? "palabra" : "palabras"}
        </span>
        <span className="separator">·</span>
        <span className="char-count">
          {charCount.toLocaleString()} {charCount === 1 ? "carácter" : "caracteres"}
        </span>
      </div>
    </div>
  );
}