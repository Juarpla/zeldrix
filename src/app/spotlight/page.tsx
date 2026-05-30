"use client";

import { useState, useEffect, useRef, type FormEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";

const aiModels = ["Zeldrix Local", "GPT-4.1", "Claude Sonnet", "Llama 3.2"];

const streamAnswerSegments = [
  "Entendido. ",
  "Estoy preparando ",
  "una respuesta breve ",
  "y directa ",
  "para mantener ",
  "el flujo compacto ",
  "sin romper ",
  "la ventana flotante.",
];

export default function SpotlightPage() {
  const [quickReplyText, setQuickReplyText] = useState("");
  const [activeModel, setActiveModel] = useState(aiModels[0]);
  const [submittedQuery, setSubmittedQuery] = useState("");
  const [streamedAnswer, setStreamedAnswer] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const streamTimerRef = useRef<number | null>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    return () => {
      if (streamTimerRef.current) {
        window.clearInterval(streamTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";

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

    return () => {
      document.body.style.overflow = previousOverflow;
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  function stopCurrentStream() {
    if (!streamTimerRef.current) {
      return;
    }

    window.clearInterval(streamTimerRef.current);
    streamTimerRef.current = null;
  }

  function handleQuickReplySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const query = quickReplyText.trim();

    if (!query) {
      return;
    }

    stopCurrentStream();
    setSubmittedQuery(query);
    setQuickReplyText("");
    setStreamedAnswer("");
    setIsStreaming(true);
    inputRef.current?.focus();

    let nextSegmentIndex = 0;

    streamTimerRef.current = window.setInterval(() => {
      setStreamedAnswer((currentAnswer) => {
        const nextAnswer = currentAnswer + streamAnswerSegments[nextSegmentIndex];
        nextSegmentIndex += 1;

        if (nextSegmentIndex >= streamAnswerSegments.length) {
          stopCurrentStream();
          setIsStreaming(false);
        }

        return nextAnswer;
      });
    }, 95);
  }

  return (
    <main className="flex h-screen w-screen items-start justify-center overflow-hidden bg-transparent px-6 pt-[34vh]">
      <motion.section
        layout
        aria-label="Quick reply input"
        transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
        className="group relative flex w-full max-w-3xl origin-top flex-col overflow-hidden rounded-[2rem] border border-slate-950/10 bg-gradient-to-br from-white/95 via-slate-50/92 to-zinc-100/90 shadow-[0_34px_80px_rgba(15,23,42,0.28),0_14px_36px_rgba(15,23,42,0.18),inset_0_1px_0_rgba(255,255,255,0.88)] backdrop-blur-2xl transition-transform duration-300 ease-out hover:-translate-y-0.5 dark:border-white/12 dark:from-zinc-950/92 dark:via-slate-950/90 dark:to-neutral-900/92 dark:shadow-[0_38px_90px_rgba(0,0,0,0.78),0_18px_44px_rgba(0,0,0,0.58),inset_0_1px_0_rgba(255,255,255,0.12)]"
      >
        <div className="pointer-events-none absolute inset-x-8 top-0 h-px bg-gradient-to-r from-transparent via-white/90 to-transparent dark:via-white/25" />
        <div className="pointer-events-none absolute -bottom-10 left-10 h-20 w-72 rounded-full bg-sky-500/10 blur-3xl dark:bg-cyan-400/10" />

        <form
          onSubmit={handleQuickReplySubmit}
          className="relative z-10 flex h-20 w-full items-center gap-3 px-4"
        >
          <input
            ref={inputRef}
            aria-label="Quick reply text"
            value={quickReplyText}
            onChange={(event) => setQuickReplyText(event.target.value)}
            placeholder="Escribe una respuesta rápida..."
            className="min-w-0 flex-1 border-none bg-transparent px-4 py-0 text-[1.05rem] font-medium text-slate-950 shadow-none outline-none transition-colors placeholder:text-slate-400 focus:ring-0 dark:text-zinc-50 dark:placeholder:text-zinc-500"
          />

          <label className="sr-only" htmlFor="active-ai-model">
            Active AI model
          </label>
          <select
            id="active-ai-model"
            aria-label="Active AI model"
            value={activeModel}
            onChange={(event) => setActiveModel(event.target.value)}
            className="h-11 max-w-44 shrink-0 cursor-pointer rounded-full border border-slate-950/10 bg-white/55 px-3 text-xs font-semibold uppercase tracking-[0.16em] text-slate-600 shadow-[inset_0_1px_0_rgba(255,255,255,0.7)] outline-none transition-colors hover:bg-white/80 focus:ring-2 focus:ring-slate-400/35 dark:border-white/10 dark:bg-white/[0.06] dark:text-zinc-300 dark:hover:bg-white/[0.09] dark:focus:ring-cyan-300/25"
          >
            {aiModels.map((model) => (
              <option key={model} value={model}>
                {model}
              </option>
            ))}
          </select>
        </form>

        <AnimatePresence initial={false}>
          {submittedQuery && (
            <motion.div
              key="stream-view"
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ duration: 0.28, ease: [0.22, 1, 0.36, 1] }}
              className="relative z-10 overflow-hidden"
            >
              <div className="mx-6 border-t border-slate-950/10 px-2 pb-6 pt-4 dark:border-white/10">
                <p className="mb-3 max-w-full truncate text-xs font-semibold uppercase tracking-[0.18em] text-slate-500 dark:text-zinc-500">
                  {submittedQuery}
                </p>
                <p
                  aria-live="polite"
                  className="min-h-14 text-[0.98rem] leading-7 text-slate-800 dark:text-zinc-200"
                >
                  {streamedAnswer}
                  {isStreaming && (
                    <motion.span
                      aria-hidden="true"
                      animate={{ opacity: [0.25, 1, 0.25] }}
                      transition={{ duration: 0.9, repeat: Infinity }}
                      className="ml-1 inline-block h-4 w-1 translate-y-0.5 rounded-full bg-slate-700 dark:bg-zinc-200"
                    />
                  )}
                </p>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.section>
    </main>
  );
}
