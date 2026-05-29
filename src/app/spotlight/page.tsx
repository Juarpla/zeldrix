"use client";

import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { transformText, AIActionType } from "@/lib/aiService";

export default function SpotlightPage() {
  const [query, setQuery] = useState("");
  const [inputText, setInputText] = useState("");
  const [result, setResult] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [activeTab, setActiveTab] = useState<AIActionType>("translate");
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Focus textarea on mount and window focus
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus();
    }
  }, []);

  // Handle global escape key to hide the window
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        try {
          const win = getCurrentWindow();
          await win.hide();
        } catch (err) {
          console.error("Failed to hide window on Escape:", err);
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleTransform = async () => {
    if (!inputText.trim()) return;
    setLoading(true);
    setError("");
    setResult("");

    try {
      const res = await transformText(activeTab, inputText);
      if (res.success && res.result) {
        setResult(res.result);
      } else {
        setError(res.error || "Error al transformar texto");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleCopy = () => {
    if (!result) return;
    navigator.clipboard.writeText(result);
  };

  return (
    <main className="w-full h-full min-h-screen p-4 flex flex-col items-center justify-center bg-transparent">
      {/* Premium Glassmorphic Raycast Container */}
      <div className="w-[640px] h-[340px] rounded-2xl border border-white/10 bg-gray-950/80 backdrop-blur-xl shadow-2xl shadow-black/80 flex flex-col overflow-hidden animate-in fade-in zoom-in duration-200">
        
        {/* Modern Command Search / Input area */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-white/5 bg-white/[0.02]">
          <div className="w-6 h-6 rounded-lg bg-gradient-to-tr from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/20">
            <svg className="w-3.5 h-3.5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 21l8.982-11.795L17.082 3 8.1 14.795l1.713 1.109z" />
            </svg>
          </div>
          <div className="flex-1 flex items-center gap-2">
            <span className="text-xs font-semibold tracking-wider text-indigo-400 select-none uppercase">
              ZELDRIX SPOTLIGHT
            </span>
            <span className="text-[10px] text-white/20 select-none">|</span>
            <span className="text-xs font-medium text-white/50 select-none">
              Presiona <kbd className="px-1.5 py-0.5 rounded bg-white/10 text-[9px] text-white/80 font-mono">ESC</kbd> para salir
            </span>
          </div>
        </div>

        {/* Dynamic Workspace */}
        <div className="flex-1 flex min-h-0">
          {/* Menu Options (Sidebar inside Spotlight) */}
          <div className="w-44 border-r border-white/5 bg-black/20 p-2 flex flex-col gap-1">
            <span className="px-3 py-1.5 text-[10px] font-bold text-white/30 uppercase tracking-wider">
              Acciones de IA
            </span>
            {[
              { id: "translate", name: "Traducir a Inglés", desc: "Translate to English" },
              { id: "style", name: "Corregir Estilo", desc: "Grammar & Flow" },
              { id: "formal", name: "Convertir a Formal", desc: "Professional tone" },
              { id: "summarize", name: "Resumir Texto", desc: "Executive summary" },
            ].map((tab) => (
              <button
                key={tab.id}
                onClick={() => {
                  setActiveTab(tab.id as AIActionType);
                  if (inputRef.current) inputRef.current.focus();
                }}
                className={`w-full text-left px-3 py-2 rounded-xl text-xs transition-all duration-200 flex flex-col gap-0.5 ${
                  activeTab === tab.id
                    ? "bg-gradient-to-r from-indigo-600/30 to-purple-600/30 border border-white/10 text-white font-medium shadow-md shadow-indigo-600/5"
                    : "text-white/50 border border-transparent hover:bg-white/[0.02] hover:text-white/80"
                }`}
              >
                <span>{tab.name}</span>
                <span className="text-[9px] opacity-60 font-normal">{tab.desc}</span>
              </button>
            ))}
          </div>

          {/* Core Input/Output Canvas */}
          <div className="flex-1 flex flex-col min-h-0 bg-black/10">
            {/* Input area */}
            <div className="flex-1 min-h-0 p-3">
              <textarea
                ref={inputRef}
                value={inputText}
                onChange={(e) => setInputText(e.target.value)}
                placeholder="Escribe o pega tu texto aquí..."
                className="w-full h-full bg-transparent text-sm text-white/90 placeholder-white/30 resize-none outline-none border-none focus:ring-0 focus:outline-none scrollbar-thin"
              />
            </div>

            {/* Results or Status Panel */}
            {loading && (
              <div className="h-28 border-t border-white/5 bg-black/40 p-3 flex flex-col items-center justify-center gap-2">
                <div className="w-5 h-5 rounded-full border-2 border-indigo-500 border-t-transparent animate-spin" />
                <span className="text-xs text-white/50 animate-pulse font-medium">Procesando con IA Local...</span>
              </div>
            )}

            {error && (
              <div className="h-28 border-t border-white/5 bg-red-950/20 p-3 overflow-y-auto text-xs text-red-400/90 font-medium">
                {error}
              </div>
            )}

            {result && !loading && !error && (
              <div className="h-28 border-t border-white/5 bg-white/[0.02] p-3 flex flex-col justify-between group">
                <div className="flex-1 overflow-y-auto text-xs text-white/85 pr-1 leading-relaxed select-text font-medium">
                  {result}
                </div>
                <div className="flex justify-end gap-2 pt-2 border-t border-white/5 mt-2">
                  <button
                    onClick={handleCopy}
                    className="px-2.5 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-[10px] text-white/70 hover:text-white border border-white/5 transition-all cursor-pointer font-semibold"
                  >
                    Copiar resultado
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Sleek bottom status bar */}
        <div className="h-9 border-t border-white/5 px-4 bg-black/40 flex items-center justify-between text-[11px] text-white/40">
          <div className="flex items-center gap-1.5 select-none font-semibold">
            <span className="w-1.5 h-1.5 rounded-full bg-indigo-500 animate-pulse" />
            <span>Listo para asistir</span>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleTransform}
              disabled={loading || !inputText.trim()}
              className={`px-3 py-1 rounded-lg text-xs font-bold transition-all duration-200 ${
                inputText.trim() && !loading
                  ? "bg-gradient-to-r from-indigo-500 to-purple-600 hover:from-indigo-600 hover:to-purple-700 text-white cursor-pointer shadow-lg shadow-indigo-500/10 active:scale-95"
                  : "bg-white/5 text-white/20 border border-white/5 cursor-not-allowed"
              }`}
            >
              Procesar (Enter)
            </button>
          </div>
        </div>
      </div>
    </main>
  );
}
