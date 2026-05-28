"use client";

import React, { useState, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { getSidecarStatus } from "@/lib/aiService";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";

interface LocalOcrFormProps {
  onInjectText: (text: string) => void;
  onLoadingChange?: (loading: boolean) => void;
}

export function LocalOcrForm({ onInjectText, onLoadingChange }: LocalOcrFormProps) {
  const [multimodalActive, setMultimodalActive] = useState<boolean | null>(null);
  const [sidecarRunning, setSidecarRunning] = useState<boolean>(false);
  const [checkingStatus, setCheckingStatus] = useState<boolean>(true);
  
  const [imagePreview, setImagePreview] = useState<string | null>(null);
  const [imageBase64, setImageBase64] = useState<string | null>(null);
  const [extractedText, setExtractedText] = useState<string>("");
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [copied, setCopied] = useState<boolean>(false);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Check sidecar and multimodal capabilities on mount
  useEffect(() => {
    async function checkCapabilities() {
      try {
        const status = await getSidecarStatus();
        setSidecarRunning(status.running);
        setMultimodalActive(!!status.multimodal);
      } catch (err) {
        console.error("Failed to check sidecar capabilities:", err);
        setMultimodalActive(false);
      } finally {
        setCheckingStatus(false);
      }
    }
    
    checkCapabilities();
    // Poll capabilities every 5 seconds to automatically react when user swaps/starts model
    const interval = setInterval(checkCapabilities, 5000);
    return () => clearInterval(interval);
  }, []);

  // Sync processing state with parent
  useEffect(() => {
    onLoadingChange?.(isProcessing);
  }, [isProcessing, onLoadingChange]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      processSelectedFile(file);
    }
  };

  const processSelectedFile = (file: File) => {
    if (!file.type.startsWith("image/")) {
      setErrorMsg("Por favor selecciona un archivo de imagen válido (PNG, JPG, WEBP).");
      return;
    }

    setErrorMsg(null);
    setExtractedText("");

    const reader = new FileReader();
    reader.onload = (event) => {
      const result = event.target?.result as string;
      setImagePreview(result);
      setImageBase64(result);
    };
    reader.onerror = () => {
      setErrorMsg("Error al leer el archivo de imagen.");
    };
    reader.readAsDataURL(file);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    if (isProcessing || checkingStatus || !multimodalActive) return;

    const file = e.dataTransfer.files?.[0];
    if (file) {
      processSelectedFile(file);
    }
  };

  const handleRunOcr = async () => {
    if (!imageBase64) return;

    setIsProcessing(true);
    setErrorMsg(null);

    try {
      const text = await invoke<string>("process_ocr_local", {
        imageBase64: imageBase64,
      });
      setExtractedText(text);
    } catch (err) {
      console.error("OCR Inference error:", err);
      setErrorMsg(typeof err === "string" ? err : String(err));
    } finally {
      setIsProcessing(false);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(extractedText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleInject = () => {
    onInjectText(extractedText);
  };

  const triggerFileSelect = () => {
    fileInputRef.current?.click();
  };

  const clearImage = () => {
    setImagePreview(null);
    setImageBase64(null);
    setExtractedText("");
    setErrorMsg(null);
  };

  // Rendering States
  if (checkingStatus) {
    return (
      <div className="flex flex-col items-center justify-center h-full p-6 text-center space-y-3">
        <svg className="animate-spin h-6 w-6 text-orange-500" fill="none" viewBox="0 0 24 24">
          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        <p className="text-xs text-muted-foreground font-semibold">Comprobando capacidades locales...</p>
      </div>
    );
  }

  if (!sidecarRunning || !multimodalActive) {
    return (
      <div className="p-6 h-full flex flex-col justify-center">
        <motion.div 
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="rounded-2xl border border-red-500/20 bg-red-500/[0.02] p-5 text-center space-y-4 backdrop-blur-sm"
        >
          <div className="w-12 h-12 rounded-full bg-red-100 dark:bg-red-950/20 text-red-600 dark:text-red-400 flex items-center justify-center mx-auto shadow-inner animate-pulse">
            <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div className="space-y-1.5">
            <h3 className="text-sm font-bold text-gray-900 dark:text-gray-100">OCR Local Deshabilitado</h3>
            <p className="text-xs text-muted-foreground leading-relaxed">
              {!sidecarRunning 
                ? "El motor de IA local no está en ejecución. Por favor, inicia el modelo desde el panel del sistema." 
                : "El modelo activo no cuenta con soporte multimodal (visión). Para realizar OCR, activa un modelo multimodal con su proyector (.mmproj)."}
            </p>
          </div>
        </motion.div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <ScrollArea className="flex-1 px-4">
        <div className="space-y-4 py-4">
          <input 
            type="file" 
            ref={fileInputRef} 
            onChange={handleFileChange} 
            accept="image/*" 
            className="hidden" 
          />

          {/* Interactive Drop Zone or Preview */}
          <AnimatePresence mode="wait">
            {!imagePreview ? (
              <motion.div
                key="dropzone"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                onDragOver={handleDragOver}
                onDrop={handleDrop}
                onClick={triggerFileSelect}
                className="relative rounded-2xl border-2 border-dashed border-gray-200/80 dark:border-gray-800 bg-white/40 dark:bg-gray-950/20 hover:bg-white/60 dark:hover:bg-gray-950/40 p-8 text-center cursor-pointer transition-all duration-200 group flex flex-col items-center justify-center min-h-[180px] shadow-sm backdrop-blur-sm"
              >
                <div className="w-12 h-12 rounded-2xl bg-orange-50 dark:bg-orange-950/20 text-orange-500 flex items-center justify-center mb-4 transition-transform group-hover:scale-110 shadow-sm border border-orange-500/10">
                  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M6.827 6.175A2.31 2.31 0 0 1 5.186 7.23c-.38.054-.757.112-1.134.175C2.999 7.58 2.25 8.507 2.25 9.574V18a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9.574c0-1.067-.75-1.994-1.802-2.169a47.865 47.865 0 0 0-1.134-.175 2.31 2.31 0 0 1-1.64-1.055l-.822-1.316a2.192 2.192 0 0 0-1.736-1.039 48.774 48.774 0 0 0-5.232 0 2.192 2.192 0 0 0-1.736 1.039l-.821 1.316Z" />
                    <path strokeLinecap="round" strokeLinejoin="round" d="M16.5 12.75a4.5 4.5 0 1 1-9 0 4.5 4.5 0 0 1 9 0ZM18.75 10.5h.008v.008h-.008V10.5Z" />
                  </svg>
                </div>
                <h3 className="text-xs font-bold text-gray-900 dark:text-gray-100">Cargar Recibo o Documento</h3>
                <p className="text-[10px] text-muted-foreground mt-1 max-w-[200px]">
                  Arrastra tu captura o haz clic para buscar un archivo de imagen.
                </p>
              </motion.div>
            ) : (
              <motion.div
                key="preview"
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.95 }}
                className="relative rounded-2xl overflow-hidden border border-gray-200/80 dark:border-gray-800 bg-white dark:bg-gray-950 p-4 shadow-sm"
              >
                <img 
                  src={imagePreview} 
                  alt="Receipt Preview" 
                  className="max-h-[160px] w-auto mx-auto object-contain rounded-lg shadow-inner bg-muted"
                />
                
                <div className="absolute top-2 right-2 flex gap-1.5">
                  <Button
                    onClick={clearImage}
                    disabled={isProcessing}
                    size="icon"
                    variant="destructive"
                    className="w-7 h-7 rounded-lg shadow-lg hover:scale-105 transition-transform"
                  >
                    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>
                  </Button>
                </div>

                {!extractedText && !isProcessing && (
                  <div className="mt-3 flex justify-center">
                    <Button 
                      onClick={handleRunOcr} 
                      size="sm" 
                      className="w-full bg-gradient-to-r from-orange-500 to-amber-600 text-white font-bold shadow-md hover:scale-[1.01] hover:shadow-lg transition-all"
                    >
                      <svg className="w-4 h-4 mr-1.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                        <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904 9 18.75l-.813-2.846a4.5 4.5 0 0 0-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 0 0 3.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 0 0 3.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 0 0-3.09 3.09Z" />
                      </svg>
                      Digitalizar con Gemma 4
                    </Button>
                  </div>
                )}
              </motion.div>
            )}
          </AnimatePresence>

          {/* Shimmer loading panel */}
          {isProcessing && (
            <div className="rounded-2xl border border-orange-500/20 bg-orange-500/[0.01] p-4 text-center space-y-3 animate-pulse">
              <div className="flex justify-center gap-2">
                <div className="w-2.5 h-2.5 bg-orange-500 rounded-full animate-bounce" />
                <div className="w-2.5 h-2.5 bg-orange-500 rounded-full animate-bounce" style={{ animationDelay: "0.2s" }} />
                <div className="w-2.5 h-2.5 bg-orange-500 rounded-full animate-bounce" style={{ animationDelay: "0.4s" }} />
              </div>
              <p className="text-xs text-orange-600 dark:text-orange-400 font-bold">Procesando OCR Local...</p>
              <p className="text-[10px] text-muted-foreground">La IA está leyendo los caracteres de la imagen.</p>
            </div>
          )}

          {/* Error Message */}
          {errorMsg && (
            <div className="rounded-xl border border-red-500/10 bg-red-500/[0.02] p-3 text-red-600 dark:text-red-400 text-xs font-semibold flex items-center gap-2">
              <svg className="w-4 h-4 shrink-0 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" />
              </svg>
              <span>{errorMsg}</span>
            </div>
          )}

          {/* Result Text Area */}
          {extractedText && (
            <motion.div 
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              className="space-y-2"
            >
              <div className="flex items-center justify-between">
                <label className="text-xs font-bold text-gray-900 dark:text-gray-100 uppercase tracking-wider">Texto Legible Recuperado</label>
                <div className="flex items-center gap-1">
                  <Button 
                    onClick={handleCopy} 
                    size="sm" 
                    variant="ghost" 
                    className="text-[10px] h-7 px-2 font-semibold"
                  >
                    {copied ? "¡Copiado!" : "Copiar"}
                  </Button>
                  <Button 
                    onClick={handleInject} 
                    size="sm" 
                    variant="ghost" 
                    className="text-[10px] h-7 px-2 text-orange-600 hover:text-orange-700 font-bold"
                  >
                    Inyectar en Texto Libre
                  </Button>
                </div>
              </div>
              <Textarea 
                value={extractedText} 
                readOnly 
                className="min-h-[160px] text-xs font-mono font-medium leading-relaxed bg-white/50 dark:bg-gray-950/50 resize-none shadow-inner"
              />
            </motion.div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
