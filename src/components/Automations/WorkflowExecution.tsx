"use client";

import React, { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { AutomationShortcut } from "@/lib/automations-data";
import { Button } from "@/components/ui/button";
import { useRouter } from "next/navigation";

interface WorkflowExecutionProps {
  shortcut: AutomationShortcut;
  onBack: () => void;
  onCompletedAction?: () => void;
}

export function WorkflowExecution({ shortcut, onBack, onCompletedAction }: WorkflowExecutionProps) {
  const router = useRouter();
  const [formInputs, setFormInputs] = useState<Record<string, string>>(() => {
    const defaults: Record<string, string> = {};
    shortcut.inputs.forEach((input) => {
      if (input.defaultValue) {
        defaults[input.id] = input.defaultValue;
      }
    });
    return defaults;
  });

  const [status, setStatus] = useState<"idle" | "running" | "completed">("idle");
  const [currentStepIndex, setCurrentStepIndex] = useState(0);
  const [progressPercent, setProgressPercent] = useState(0);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const activeStep = shortcut.steps[currentStepIndex];

  // Run mock workflow execution
  const runWorkflow = () => {
    setStatus("running");
    setCurrentStepIndex(0);
    setProgressPercent(0);
  };

  useEffect(() => {
    if (status !== "running") return;

    const totalSteps = shortcut.steps.length;
    const totalDurationMs = shortcut.estimatedSeconds * 1000;
    const intervalTime = 50; // Update progress every 50ms
    const totalIterations = totalDurationMs / intervalTime;
    let iteration = 0;

    const interval = setInterval(() => {
      iteration++;
      const nextProgress = Math.min((iteration / totalIterations) * 100, 100);
      setProgressPercent(Math.round(nextProgress));

      // Calculate step transitions
      const stepIndex = Math.min(
        Math.floor((nextProgress / 100) * totalSteps),
        totalSteps - 1
      );
      setCurrentStepIndex(stepIndex);

      if (nextProgress >= 100) {
        clearInterval(interval);
        setTimeout(() => {
          setStatus("completed");
          if (onCompletedAction) onCompletedAction();
        }, 300);
      }
    }, intervalTime);

    return () => clearInterval(interval);
  }, [status, shortcut]);

  const handleInputChange = (id: string, value: string) => {
    setFormInputs((prev) => ({ ...prev, [id]: value }));
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setSelectedFile(e.target.files[0].name);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(shortcut.mockOutput.replace(/<[^>]*>/g, "")); // Copy stripped HTML text
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleOpenInEditor = () => {
    // Redirect to /editor, passing the output content as query parameter
    // The editor can read it and inject it!
    const textOutput = shortcut.mockOutput;
    // Compress or encode to avoid URL limits if too big, or use sessionStorage
    if (typeof window !== "undefined") {
      sessionStorage.setItem("zeldrix_import_editor_text", textOutput);
    }
    router.push("/editor");
  };

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      {/* Top Header Navigation */}
      <div className="flex items-center justify-between pb-4 border-b border-gray-100 dark:border-gray-800">
        <button
          onClick={onBack}
          className="inline-flex items-center gap-2 text-sm font-semibold text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
          </svg>
          Volver al Hub
        </button>
        <h2 className="text-xl font-bold tracking-tight text-gray-900 dark:text-white flex items-center gap-2">
          <span>Automatización Activa: {shortcut.title}</span>
        </h2>
      </div>

      <AnimatePresence mode="wait">
        {status === "idle" && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-850 rounded-2xl p-6 shadow-sm space-y-6"
          >
            <div>
              <h3 className="text-lg font-bold text-gray-900 dark:text-white">Configuración del Flujo de Trabajo</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Verifica o introduce los datos requeridos para ejecutar la automatización corporativa.
              </p>
            </div>

            {/* Render dynamic inputs */}
            <div className="space-y-4">
              {shortcut.inputs.map((input) => (
                <div key={input.id} className="space-y-2">
                  <label className="block text-sm font-bold text-gray-700 dark:text-gray-300">
                    {input.label}
                  </label>
                  {input.type === "textarea" ? (
                    <textarea
                      value={formInputs[input.id] || ""}
                      onChange={(e) => handleInputChange(input.id, e.target.value)}
                      placeholder={input.placeholder}
                      className="w-full min-h-[160px] p-4 rounded-xl border border-gray-250 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-950 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all dark:text-gray-100"
                    />
                  ) : input.type === "file" ? (
                    <div className="flex flex-col items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-800 rounded-xl p-8 bg-gray-50/50 dark:bg-gray-950 transition-colors hover:bg-gray-50 dark:hover:bg-gray-900/50">
                      <input
                        type="file"
                        ref={fileInputRef}
                        onChange={handleFileChange}
                        className="hidden"
                        accept=".pdf,.png,.jpg,.jpeg,.doc,.docx"
                      />
                      <svg className="w-10 h-10 text-gray-400 mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M12 16.5V9.75m0 0l3 3m-3-3l-3 3M6.75 19.5a4.5 4.5 0 0 1-1.41-8.775 5.25 5.25 0 0 1 10.233-2.33 3 3 0 0 1 3.758 3.848A3.752 3.752 0 0 1 18 19.5H6.75z" />
                      </svg>
                      {selectedFile ? (
                        <div className="text-center space-y-1">
                          <p className="text-sm font-bold text-gray-900 dark:text-white">{selectedFile}</p>
                          <p className="text-xs text-gray-400">Archivo listo para procesar</p>
                        </div>
                      ) : (
                        <div className="text-center space-y-1">
                          <Button
                            type="button"
                            variant="secondary"
                            size="sm"
                            onClick={() => fileInputRef.current?.click()}
                          >
                            Seleccionar Archivo
                          </Button>
                          <p className="text-xs text-gray-400">Soporta PDF, DOCX, PNG o JPG hasta 25MB</p>
                        </div>
                      )}
                    </div>
                  ) : (
                    <input
                      type="text"
                      value={formInputs[input.id] || ""}
                      onChange={(e) => handleInputChange(input.id, e.target.value)}
                      placeholder={input.placeholder}
                      className="w-full p-4 rounded-xl border border-gray-250 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-950 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none transition-all dark:text-gray-100"
                    />
                  )}
                </div>
              ))}
            </div>

            {/* Execute actions */}
            <div className="pt-4 border-t border-gray-100 dark:border-gray-800 flex justify-end gap-3">
              <Button variant="outline" onClick={onBack}>
                Cancelar
              </Button>
              <Button
                onClick={runWorkflow}
                disabled={shortcut.inputs.some((i) => i.type === "file") && !selectedFile}
                className="bg-blue-600 hover:bg-blue-700 text-white font-bold px-6 py-2 rounded-xl"
              >
                Disparar Automatización
              </Button>
            </div>
          </motion.div>
        )}

        {status === "running" && (
          <motion.div
            initial={{ opacity: 0, scale: 0.98 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.98 }}
            className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-850 rounded-2xl p-10 shadow-sm text-center space-y-8 flex flex-col items-center justify-center min-h-[400px]"
          >
            {/* Spinning/pulsing animation graphic */}
            <div className="relative w-24 h-24 flex items-center justify-center">
              <motion.div
                animate={{ rotate: 360 }}
                transition={{ repeat: Infinity, duration: 2, ease: "linear" }}
                className="absolute inset-0 rounded-full border-4 border-gray-100 dark:border-gray-800 border-t-blue-600"
              />
              <motion.div
                animate={{ scale: [0.9, 1.1, 0.9] }}
                transition={{ repeat: Infinity, duration: 1.5, ease: "easeInOut" }}
                className="w-12 h-12 bg-blue-500/10 dark:bg-blue-500/5 rounded-full flex items-center justify-center text-blue-600"
              >
                <svg className="w-6 h-6 animate-pulse" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
                </svg>
              </motion.div>
            </div>

            <div className="space-y-3 max-w-md">
              <h3 className="text-xl font-bold text-gray-900 dark:text-white">Procesando Flujo de Trabajo Pesado...</h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                La Inteligencia Artificial está estructurando y analizando tus datos corporativos de manera autónoma.
              </p>
            </div>

            {/* Elegant Steps and Progress Tracker */}
            <div className="w-full max-w-lg space-y-4">
              <div className="relative w-full h-2 bg-gray-100 dark:bg-gray-800 rounded-full overflow-hidden">
                <motion.div
                  className="absolute left-0 top-0 h-full bg-blue-600 rounded-full"
                  style={{ width: `${progressPercent}%` }}
                />
              </div>
              <div className="flex justify-between text-xs font-bold text-gray-400">
                <span>AVANCE: {progressPercent}%</span>
                <span>T. RESTANTE: {Math.max(0, Math.ceil(shortcut.estimatedSeconds * (1 - progressPercent / 100)))}s</span>
              </div>

              {/* Progress Steps list */}
              <div className="border-t border-gray-100 dark:border-gray-800 pt-6 mt-2 grid grid-cols-1 md:grid-cols-3 gap-4 text-left">
                {shortcut.steps.map((step, idx) => {
                  const isDone = idx < currentStepIndex;
                  const isActive = idx === currentStepIndex;
                  return (
                    <div
                      key={idx}
                      className={`p-3 rounded-xl border transition-all duration-300 ${
                        isDone
                          ? "bg-emerald-50/50 dark:bg-emerald-950/20 border-emerald-200 dark:border-emerald-900 text-emerald-900 dark:text-emerald-300"
                          : isActive
                          ? "bg-blue-50/50 dark:bg-blue-950/20 border-blue-200 dark:border-blue-900 text-blue-900 dark:text-blue-300 animate-pulse"
                          : "bg-gray-50/50 dark:bg-gray-950 border-gray-100 dark:border-gray-850 text-gray-400"
                      }`}
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <span className={`w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-extrabold ${
                          isDone
                            ? "bg-emerald-500 text-white"
                            : isActive
                            ? "bg-blue-500 text-white"
                            : "bg-gray-300 dark:bg-gray-700 text-white"
                        }`}>
                          {isDone ? "✓" : idx + 1}
                        </span>
                        <span className="text-xs font-bold tracking-tight">{step.label}</span>
                      </div>
                      <p className="text-[11px] font-medium leading-normal opacity-80">{step.description}</p>
                    </div>
                  );
                })}
              </div>
            </div>
          </motion.div>
        )}

        {status === "completed" && (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="space-y-6"
          >
            {/* Success banner card */}
            <div className="bg-emerald-50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-900 rounded-2xl p-5 flex items-center justify-between gap-4">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-emerald-500 text-white rounded-full flex items-center justify-center font-bold">
                  ✓
                </div>
                <div>
                  <h4 className="font-extrabold text-emerald-900 dark:text-emerald-300">¡Procesamiento Completado de Forma Exitosa!</h4>
                  <p className="text-xs text-emerald-700 dark:text-emerald-400">
                    El resultado fue estructurado mediante los filtros de correspondencia de manera estructurada.
                  </p>
                </div>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={runWorkflow}
                className="text-emerald-700 dark:text-emerald-400 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 font-bold"
              >
                Volver a Ejecutar
              </Button>
            </div>

            {/* Premium Output Display Container */}
            <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-850 rounded-2xl shadow-sm overflow-hidden flex flex-col">
              <div className="px-6 py-4 bg-gray-50/50 dark:bg-gray-950 border-b border-gray-150 dark:border-gray-800 flex items-center justify-between">
                <span className="text-xs font-bold text-gray-500 dark:text-gray-400 uppercase tracking-widest">
                  Resultado de la Automatización
                </span>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleCopy}
                    className="h-8 text-xs font-bold gap-1"
                  >
                    {copied ? (
                      <>
                        <span className="text-emerald-500">✓</span> Copiado
                      </>
                    ) : (
                      <>
                        <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                          <path strokeLinecap="round" strokeLinejoin="round" d="M15.666 3.888A2.25 2.25 0 0 0 13.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 0 1-.75.75H9a.75.75 0 0 1-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 0 1-2.25 2.25H6.75A2.25 2.25 0 0 1 4.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 0 1 1.927-.184" />
                        </svg>
                        Copiar
                      </>
                    )}
                  </Button>

                  <Button
                    onClick={handleOpenInEditor}
                    className="h-8 text-xs font-bold bg-blue-600 hover:bg-blue-700 text-white gap-1"
                  >
                    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
                    </svg>
                    Abrir en Editor
                  </Button>
                </div>
              </div>

              {/* Styled Preview Frame */}
              <div className="p-6 overflow-auto max-h-[450px]">
                <div
                  className="prose dark:prose-invert max-w-none text-sm text-gray-800 dark:text-gray-200 leading-relaxed font-sans"
                  dangerouslySetInnerHTML={{ __html: shortcut.mockOutput }}
                />
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
