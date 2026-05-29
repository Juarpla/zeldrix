"use client";

import React, { useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { AUTOMATIONS_DATA, AutomationShortcut } from "@/lib/automations-data";
import { AutomationCard } from "./AutomationCard";
import { WorkflowExecution } from "./WorkflowExecution";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  CORPORATE_AUTOMATION_ICONS,
  CustomAutomationIconName,
  CustomAutomationOutputType,
  createCustomAutomationPreset,
  customPresetToShortcut,
  listCustomAutomationPresets,
} from "@/lib/custom-automation-presets-service";
import Link from "next/link";

export function AutomationHub() {
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");
  const [activeShortcut, setActiveShortcut] = useState<AutomationShortcut | null>(null);
  const [customShortcuts, setCustomShortcuts] = useState<AutomationShortcut[]>([]);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [presetTitle, setPresetTitle] = useState("");
  const [presetIconName, setPresetIconName] = useState<CustomAutomationIconName>("translate");
  const [presetBasePrompt, setPresetBasePrompt] = useState("");
  const [presetOutputType, setPresetOutputType] = useState<CustomAutomationOutputType>("text");
  const [presetFormError, setPresetFormError] = useState<string | null>(null);
  const [isSavingPreset, setIsSavingPreset] = useState(false);

  // Local simulated stats state
  const [stats, setStats] = useState({
    executionsCount: 142,
    hoursSaved: 48.5,
    successRate: 99.8,
  });

  const categories = [
    { id: "all", label: "Todos" },
    { id: "email", label: "Correos" },
    { id: "meetings", label: "Reuniones & Minutas" },
    { id: "data-extraction", label: "Extracción de Datos" },
    { id: "custom", label: "Mis Presets" },
  ];

  useEffect(() => {
    listCustomAutomationPresets()
      .then((presets) => setCustomShortcuts(presets.map(customPresetToShortcut)))
      .catch((error) => {
        console.warn("Custom automation presets are not available:", error);
      });
  }, []);

  const shortcuts = useMemo(
    () => [...AUTOMATIONS_DATA, ...customShortcuts],
    [customShortcuts],
  );

  // Filtering logic
  const filteredShortcuts = useMemo(() => {
    return shortcuts.filter((shortcut) => {
      const matchesSearch =
        shortcut.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        shortcut.description.toLowerCase().includes(searchTerm.toLowerCase());
      const matchesCategory =
        selectedCategory === "all" || shortcut.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });
  }, [searchTerm, selectedCategory, shortcuts]);

  const handleCreatePreset = async () => {
    setPresetFormError(null);
    setIsSavingPreset(true);

    try {
      const savedPreset = await createCustomAutomationPreset({
        title: presetTitle,
        icon_name: presetIconName,
        base_prompt: presetBasePrompt,
        output_type: presetOutputType,
      });

      setCustomShortcuts((prev) => [...prev, customPresetToShortcut(savedPreset)]);
      setPresetTitle("");
      setPresetIconName("translate");
      setPresetBasePrompt("");
      setPresetOutputType("text");
      setIsCreateDialogOpen(false);
      setSelectedCategory("custom");
    } catch (error: any) {
      setPresetFormError(typeof error === "string" ? error : error?.message ?? "No se pudo guardar el preset.");
    } finally {
      setIsSavingPreset(false);
    }
  };

  const handleShortcutComplete = () => {
    // Increment local stats when a workflow completes successfully
    setStats((prev) => ({
      ...prev,
      executionsCount: prev.executionsCount + 1,
      hoursSaved: Number((prev.hoursSaved + 0.25).toFixed(2)),
    }));
  };

  return (
    <div className="space-y-8">
      <AnimatePresence mode="wait">
        {!activeShortcut ? (
          <motion.div
            key="grid-view"
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -15 }}
            className="space-y-8"
          >
            {/* Header section with back link */}
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
              <div>
                <h1 className="text-4xl font-extrabold tracking-tight bg-gradient-to-r from-gray-950 to-gray-600 dark:from-white dark:to-gray-400 bg-clip-text text-transparent">
                  Centro de Automatizaciones
                </h1>
                <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 font-semibold">
                  Dispara tareas corporativas pesadas con un solo clic y optimiza tu tiempo diario.
                </p>
              </div>

              <div className="flex flex-wrap items-center gap-3">
                <Button
                  type="button"
                  onClick={() => setIsCreateDialogOpen(true)}
                  className="rounded-xl bg-gray-950 px-4 text-xs font-extrabold text-white hover:bg-gray-800 dark:bg-white dark:text-gray-950 dark:hover:bg-gray-200"
                >
                  Crear Preset
                </Button>
                <Link
                  href="/editor"
                  className="inline-flex items-center gap-2 text-sm font-bold text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" strokeWidth={2.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" />
                  </svg>
                  Volver al Editor de Documentos
                </Link>
              </div>
            </div>

            {/* Quick Stats Grid */}
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
              <div className="bg-white dark:bg-gray-900 border border-gray-150 dark:border-gray-800 rounded-2xl p-5 shadow-sm space-y-1">
                <span className="text-xs font-bold text-gray-400 uppercase tracking-wider">
                  Ejecuciones Totales
                </span>
                <div className="flex items-baseline gap-2">
                  <span className="text-3xl font-extrabold text-gray-900 dark:text-white">
                    {stats.executionsCount}
                  </span>
                  <span className="text-xs text-emerald-500 font-bold">✓ 100% Ok</span>
                </div>
                <p className="text-[11px] text-gray-400">Flujos disparados en este espacio de trabajo</p>
              </div>

              <div className="bg-white dark:bg-gray-900 border border-gray-150 dark:border-gray-800 rounded-2xl p-5 shadow-sm space-y-1">
                <span className="text-xs font-bold text-gray-400 uppercase tracking-wider">
                  Tiempo de Oficina Ahorrado
                </span>
                <div className="flex items-baseline gap-2">
                  <span className="text-3xl font-extrabold text-blue-600 dark:text-blue-400">
                    ~{stats.hoursSaved} hrs
                  </span>
                  <span className="text-xs text-blue-500 font-bold">Productivo</span>
                </div>
                <p className="text-[11px] text-gray-400">Basado en promedio manual de 15 min/tarea</p>
              </div>

              <div className="bg-white dark:bg-gray-900 border border-gray-150 dark:border-gray-800 rounded-2xl p-5 shadow-sm space-y-1">
                <span className="text-xs font-bold text-gray-400 uppercase tracking-wider">
                  Tasa de Precisión IA
                </span>
                <div className="flex items-baseline gap-2">
                  <span className="text-3xl font-extrabold text-emerald-600 dark:text-emerald-400">
                    {stats.successRate}%
                  </span>
                  <span className="text-xs text-emerald-500 font-bold">Excelente</span>
                </div>
                <p className="text-[11px] text-gray-400">Validado mediante cuadres de consistencia</p>
              </div>
            </div>

            {/* Filter and search bar */}
            <div className="flex flex-col md:flex-row gap-4 items-stretch md:items-center justify-between bg-gray-50/50 dark:bg-gray-950 p-4 rounded-2xl border border-gray-150 dark:border-gray-800">
              {/* Category selector pills */}
              <div className="flex flex-wrap gap-2">
                {categories.map((cat) => (
                  <button
                    key={cat.id}
                    onClick={() => setSelectedCategory(cat.id)}
                    className={`px-4 py-2 text-xs font-extrabold rounded-xl transition-all ${
                      selectedCategory === cat.id
                        ? "bg-gray-900 text-white dark:bg-white dark:text-gray-900 shadow-sm"
                        : "bg-white text-gray-600 dark:bg-gray-900 dark:text-gray-400 border border-gray-200 dark:border-gray-800 hover:border-gray-300 dark:hover:border-gray-700"
                    }`}
                  >
                    {cat.label}
                  </button>
                ))}
              </div>

              {/* Search text field */}
              <div className="relative w-full md:w-72">
                <Input
                  type="text"
                  placeholder="Buscar automatizaciones..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-9 bg-white dark:bg-gray-900 text-xs py-2 rounded-xl"
                />
                <svg className="absolute left-3 top-2.5 w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
            </div>

            {/* Shortcuts Grid */}
            {filteredShortcuts.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {filteredShortcuts.map((shortcut) => (
                  <AutomationCard
                    key={shortcut.id}
                    shortcut={shortcut}
                    onSelect={setActiveShortcut}
                  />
                ))}
              </div>
            ) : (
              <div className="text-center py-16 border border-dashed rounded-2xl border-gray-200 dark:border-gray-850 space-y-3">
                <svg className="w-12 h-12 text-gray-300 mx-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M9.879 7.519c1.171-1.025 3.071-1.025 4.242 0 1.172 1.025 1.172 2.687 0 3.712-.203.179-.43.326-.67.442-.745.361-1.45.999-1.45 1.827v.75M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9 5.25h.008v.008H12v-.008z" />
                </svg>
                <div className="space-y-1">
                  <h4 className="font-bold text-gray-950 dark:text-white text-sm">No se encontraron automatizaciones</h4>
                  <p className="text-xs text-gray-500">Prueba con otra palabra clave o limpia el filtro de búsqueda.</p>
                </div>
              </div>
            )}
          </motion.div>
        ) : (
          <motion.div
            key="execution-view"
            initial={{ opacity: 0, scale: 0.99 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.99 }}
          >
            <WorkflowExecution
              shortcut={activeShortcut}
              onBack={() => setActiveShortcut(null)}
              onCompletedAction={handleShortcutComplete}
            />
          </motion.div>
        )}
      </AnimatePresence>

      <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
        <DialogContent className="max-w-2xl rounded-2xl border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
          <DialogHeader>
            <DialogTitle>Crear tarjeta personalizada</DialogTitle>
            <DialogDescription>
              Define un preset corporativo reutilizable. El prompt base queda guardado y no se muestra en la tarjeta.
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-5 py-2">
            {presetFormError && (
              <div className="rounded-xl border border-rose-200 bg-rose-50 p-3 text-xs font-semibold text-rose-700 dark:border-rose-900 dark:bg-rose-950/30 dark:text-rose-300">
                {presetFormError}
              </div>
            )}

            <div className="grid gap-2">
              <label className="text-xs font-extrabold uppercase tracking-wider text-gray-500">
                Título visible
              </label>
              <Input
                value={presetTitle}
                onChange={(event) => setPresetTitle(event.target.value)}
                placeholder="Ej: Traducir a alemán comercial"
                className="rounded-xl"
              />
            </div>

            <div className="grid gap-2">
              <label className="text-xs font-extrabold uppercase tracking-wider text-gray-500">
                Icono corporativo
              </label>
              <div className="grid grid-cols-2 gap-2 sm:grid-cols-5">
                {CORPORATE_AUTOMATION_ICONS.map((icon) => (
                  <button
                    key={icon.id}
                    type="button"
                    onClick={() => setPresetIconName(icon.id)}
                    className={`rounded-xl border px-3 py-2 text-xs font-bold transition-colors ${
                      presetIconName === icon.id
                        ? "border-gray-950 bg-gray-950 text-white dark:border-white dark:bg-white dark:text-gray-950"
                        : "border-gray-200 bg-gray-50 text-gray-600 hover:border-gray-300 dark:border-gray-800 dark:bg-gray-900 dark:text-gray-300"
                    }`}
                  >
                    {icon.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="grid gap-2">
              <label className="text-xs font-extrabold uppercase tracking-wider text-gray-500">
                Prompt base oculto
              </label>
              <Textarea
                value={presetBasePrompt}
                onChange={(event) => setPresetBasePrompt(event.target.value)}
                placeholder='Ej: "Traduce esto al idioma Alemán con tono comercial."'
                className="min-h-[130px] rounded-xl"
              />
            </div>

            <div className="grid gap-2">
              <label className="text-xs font-extrabold uppercase tracking-wider text-gray-500">
                Componente de salida
              </label>
              <div className="grid grid-cols-2 gap-2">
                {(["text", "table"] as const).map((type) => (
                  <button
                    key={type}
                    type="button"
                    onClick={() => setPresetOutputType(type)}
                    className={`rounded-xl border px-4 py-3 text-sm font-bold transition-colors ${
                      presetOutputType === type
                        ? "border-blue-600 bg-blue-600 text-white"
                        : "border-gray-200 bg-gray-50 text-gray-600 hover:border-gray-300 dark:border-gray-800 dark:bg-gray-900 dark:text-gray-300"
                    }`}
                  >
                    {type === "text" ? "Texto" : "Tabla"}
                  </button>
                ))}
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
              Cancelar
            </Button>
            <Button onClick={handleCreatePreset} disabled={isSavingPreset}>
              {isSavingPreset ? "Guardando..." : "Guardar Preset"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
