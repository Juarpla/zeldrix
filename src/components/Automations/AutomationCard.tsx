"use client";

import React from "react";
import { motion } from "framer-motion";
import { AutomationShortcut } from "@/lib/automations-data";
import { Badge } from "@/components/ui/badge";

interface AutomationCardProps {
  shortcut: AutomationShortcut;
  onSelect: (shortcut: AutomationShortcut) => void;
}

const CATEGORY_COLORS: Record<string, { bg: string; text: string; border: string; glow: string }> = {
  email: {
    bg: "bg-blue-500/10 dark:bg-blue-500/5",
    text: "text-blue-600 dark:text-blue-400",
    border: "border-blue-500/20 dark:border-blue-500/10",
    glow: "group-hover:shadow-[0_0_20px_rgba(59,130,246,0.15)]",
  },
  meetings: {
    bg: "bg-amber-500/10 dark:bg-amber-500/5",
    text: "text-amber-600 dark:text-amber-400",
    border: "border-amber-500/20 dark:border-amber-500/10",
    glow: "group-hover:shadow-[0_0_20px_rgba(245,158,11,0.15)]",
  },
  "data-extraction": {
    bg: "bg-emerald-500/10 dark:bg-emerald-500/5",
    text: "text-emerald-600 dark:text-emerald-400",
    border: "border-emerald-500/20 dark:border-emerald-500/10",
    glow: "group-hover:shadow-[0_0_20px_rgba(16,185,129,0.15)]",
  },
  documents: {
    bg: "bg-purple-500/10 dark:bg-purple-500/5",
    text: "text-purple-600 dark:text-purple-400",
    border: "border-purple-500/20 dark:border-purple-500/10",
    glow: "group-hover:shadow-[0_0_20px_rgba(139,92,246,0.15)]",
  },
  custom: {
    bg: "bg-slate-500/10 dark:bg-slate-500/5",
    text: "text-slate-700 dark:text-slate-300",
    border: "border-slate-500/20 dark:border-slate-500/10",
    glow: "group-hover:shadow-[0_0_20px_rgba(71,85,105,0.15)]",
  },
};

const DIFFICULTY_LABELS: Record<string, { label: string; color: string }> = {
  light: { label: "Ligero", color: "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400" },
  medium: { label: "Moderado", color: "bg-amber-500/10 text-amber-600 dark:text-amber-400" },
  heavy: { label: "Pesado", color: "bg-red-500/10 text-red-600 dark:text-red-400 animate-pulse" },
};

export function AutomationCard({ shortcut, onSelect }: AutomationCardProps) {
  const catColor = CATEGORY_COLORS[shortcut.category] || CATEGORY_COLORS.email;
  const diff = DIFFICULTY_LABELS[shortcut.difficulty] || DIFFICULTY_LABELS.light;

  const renderIcon = () => {
    const className = `w-6 h-6 ${catColor.text} transition-transform duration-300 group-hover:scale-110`;
    switch (shortcut.iconName) {
      case "email":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M21.75 6.75v10.5a2.25 2.25 0 0 1-2.25 2.25h-15a2.25 2.25 0 0 1-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0 0 19.5 4.5h-15a2.25 2.25 0 0 0-2.25 2.25m19.5 0v.243a2.25 2.25 0 0 1-1.07 1.916l-7.5 4.615a2.25 2.25 0 0 1-2.36 0L3.32 8.91a2.25 2.25 0 0 1-1.07-1.916V6.75" />
          </svg>
        );
      case "minutes":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 1 1 .512 1.347l-.041.02a.75.75 0 0 1-.512-1.347zM11.25 15.25l.041-.02a.75.75 0 1 1 .512 1.347l-.041.02a.75.75 0 0 1-.512-1.347zM11.25 7.25l.041-.02a.75.75 0 1 1 .512 1.347l-.041.02a.75.75 0 0 1-.512-1.347zM15 11.25l.041-.02a.75.75 0 1 1 .512 1.347l-.041.02a.75.75 0 0 1-.512-1.347zM15 15.25l.041-.02a.75.75 0 1 1 .512 1.347l-.041.02a.75.75 0 0 1-.512-1.347zM15 7.25l.041-.02a.75.75 0 1 1 .512 1.347l-.041.02a.75.75 0 0 1-.512-1.347zM7.5 7.25h.008v.008H7.5V7.25zm0 4h.008v.008H7.5v-.008zm0 4h.008v.008H7.5v-.008z" />
            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9z" />
          </svg>
        );
      case "pdf":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9zM9 13h6M9 17h6" />
          </svg>
        );
      case "reply":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M9 15 3 9m0 0 6-6M3 9h12a6 6 0 0 1 0 12h-3" />
          </svg>
        );
      case "translate":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M10.5 21l5.25-11.25L21 21m-8.25-4.5h6M3 5.25h12M6.75 3v2.25m4.5 0c-.563 2.288-2.063 4.223-4.028 5.25m0 0A8.966 8.966 0 013 8.25m4.222 2.25a8.96 8.96 0 004.028 1.5" />
          </svg>
        );
      case "table":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 5.25h16.5m-16.5 4.5h16.5m-16.5 4.5h16.5m-16.5 4.5h16.5M8.25 5.25v13.5m7.5-13.5v13.5" />
          </svg>
        );
      case "briefcase":
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M20.25 14.15v4.1A2.25 2.25 0 0118 20.5H6a2.25 2.25 0 01-2.25-2.25v-4.1m16.5 0a2.18 2.18 0 00.75-1.65V8.25A2.25 2.25 0 0018.75 6h-13.5A2.25 2.25 0 003 8.25v4.25c0 .64.274 1.216.75 1.65m16.5 0a10.5 10.5 0 01-8.25 3.85 10.5 10.5 0 01-8.25-3.85M9.75 6V4.5A1.5 1.5 0 0111.25 3h1.5a1.5 1.5 0 011.5 1.5V6" />
          </svg>
        );
      default:
        return (
          <svg className={className} fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
          </svg>
        );
    }
  };

  return (
    <motion.div
      onClick={() => onSelect(shortcut)}
      whileHover={{ y: -6, scale: 1.025 }}
      whileTap={{ scale: 0.98 }}
      className={`group relative flex flex-col justify-between overflow-hidden rounded-2xl border bg-white dark:bg-gray-900 p-6 transition-all duration-300 cursor-pointer shadow-sm hover:border-gray-300 dark:hover:border-gray-700 ${catColor.glow}`}
    >
      {/* Background dynamic glow pattern */}
      <div className="absolute top-0 right-0 w-32 h-32 rounded-full blur-3xl opacity-10 group-hover:opacity-20 transition-opacity duration-300 bg-current -mr-10 -mt-10" style={{ color: "currentColor" }} />

      <div className="space-y-4">
        {/* Header Icon + Difficulty Badge */}
        <div className="flex items-center justify-between">
          <div className={`p-3 rounded-xl ${catColor.bg} ${catColor.border} border`}>
            {renderIcon()}
          </div>
          <Badge variant="secondary" className={`font-semibold tracking-wide ${diff.color}`}>
            {diff.label}
          </Badge>
        </div>

        {/* Text */}
        <div className="space-y-2">
          <h3 className="text-lg font-bold text-gray-900 dark:text-white group-hover:text-gray-950 dark:group-hover:text-white transition-colors">
            {shortcut.title}
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 line-clamp-3 leading-relaxed">
            {shortcut.description}
          </p>
        </div>
      </div>

      {/* Action trigger footer */}
      <div className="mt-6 pt-4 border-t border-gray-100 dark:border-gray-800 flex items-center justify-between">
        <span className="text-xs text-gray-400 font-semibold flex items-center gap-1">
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          ~{shortcut.estimatedSeconds} seg.
        </span>
        <span className={`text-sm font-bold flex items-center gap-1 transition-all duration-300 group-hover:translate-x-1 ${catColor.text}`}>
          Ejecutar
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
          </svg>
        </span>
      </div>
    </motion.div>
  );
}
