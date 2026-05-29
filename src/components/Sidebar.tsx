"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { getSidecarStatus } from "@/lib/aiService";

interface NavigationItem {
  name: string;
  href: string;
  icon: React.ReactNode;
}

export default function Sidebar() {
  const pathname = usePathname();
  const router = useRouter();
  const [status, setStatus] = useState<{
    running: boolean;
    model: string | null;
    multimodal: boolean;
  }>({
    running: false,
    model: null,
    multimodal: false,
  });

  const [checking, setChecking] = useState(true);

  useEffect(() => {
    async function checkStatus() {
      try {
        const res = await getSidecarStatus();
        setStatus({
          running: res.running,
          model: res.model ? extractModelName(res.model) : null,
          multimodal: !!res.multimodal,
        });
      } catch (err) {
        setStatus({ running: false, model: null, multimodal: false });
      } finally {
        setChecking(false);
      }
    }

    checkStatus();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  function extractModelName(path: string): string {
    const filename = path.split(/[/\\]/).pop() || "";
    const stem = filename.replace(".gguf", "");
    if (stem.includes("gemma-4-E2B-it-IQ4_XS")) return "Gemma 4 2B IT Q4_XS";
    if (stem.includes("gemma-4-E4B-it-IQ4_XS")) return "Gemma 4 4B IT Q4_XS";
    if (stem.includes("gemma-4-E2B-it-Q6_K")) return "Gemma 4 2B IT Q6_K";
    if (stem.includes("gemma-4-E4B-it-Q6_K")) return "Gemma 4 4B IT Q6_K";
    if (stem.includes("gemma-4-E2B-it-Q8_0")) return "Gemma 4 2B IT Q8_0";
    if (stem.includes("gemma-4-E4B-it-Q8_0")) return "Gemma 4 4B IT Q8_0";
    if (stem.includes("gemma-4-E2B-it-BF16")) return "Gemma 4 2B IT BF16";
    if (stem.includes("gemma-4-E4B-it-BF16")) return "Gemma 4 4B IT BF16";
    if (stem.includes("Qwen3.6-27B-UD-IQ2_XXS")) return "Qwen 3.6 27B UD IQ2_XXS";
    return stem || "Modelo Local";
  }

  const items: NavigationItem[] = [
    {
      name: "Conversación",
      href: "/chat",
      icon: (
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 1 1 0-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 1 0 0-2.684 3 3 0 0 0 0 2.684Zm0 9.316a3 3 0 1 0 0-2.684 3 3 0 0 0 0 2.684Z" />
        </svg>
      ),
    },
    {
      name: "Redacción",
      href: "/editor",
      icon: (
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
        </svg>
      ),
    },
    {
      name: "Plantillas",
      href: "/templates",
      icon: (
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
        </svg>
      ),
    },
    {
      name: "Base Documental",
      href: "/knowledge-base",
      icon: (
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 12.75V12A9 9 0 0 1 12 3v0a9 9 0 0 1 9 9v0.75m-18 0h18M2.25 12.75a8.966 8.966 0 0 1 1.985-5.514L12 12.75m0 0l6.81-5.514a8.966 8.966 0 0 1 1.985 5.514m-18 0h18" />
        </svg>
      ),
    },
    {
      name: "Automatizaciones",
      href: "/automations",
      icon: (
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
        </svg>
      ),
    },
    {
      name: "Modelos",
      href: "/models",
      icon: (
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
          <path strokeLinecap="round" strokeLinejoin="round" d="M9 3.75H6.912a2.25 2.25 0 00-2.15 1.588L2.35 13.177a2.25 2.25 0 00-.1.661V18a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18v-4.162c0-.224-.034-.447-.1-.661L19.24 5.338a2.25 2.25 0 00-2.15-1.588H15M9 3.75a2.25 2.25 0 014.5 0M9 3.75h4.5m0 0H15m-6 3h6m-3 9v-3m0 0l-3 3m3-3l3 3" />
        </svg>
      ),
    },
  ];

  const isActive = (href: string) => {
    if (href === "/chat" && pathname === "/") return true;
    return pathname.startsWith(href);
  };

  return (
    <>
      {/* Desktop Left Sidebar */}
      <aside className="hidden md:flex flex-col w-64 h-screen fixed left-0 top-0 border-r border-gray-800 bg-gray-950 text-gray-200 z-30">
        {/* Brand Header */}
        <div className="flex items-center gap-3 px-6 py-6 border-b border-gray-900">
          <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/20">
            <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M9.813 15.904L9 21l8.982-11.795L17.082 3 8.1 14.795l1.713 1.109z" />
            </svg>
          </div>
          <div>
            <h1 className="text-base font-bold bg-gradient-to-r from-white via-gray-100 to-gray-400 bg-clip-text text-transparent">
              ZELDRIX
            </h1>
            <p className="text-[10px] text-gray-500 font-semibold tracking-wider uppercase">
              Local AI Engine
            </p>
          </div>
        </div>

        {/* Navigation Items */}
        <nav className="flex-1 px-4 py-6 space-y-1.5 overflow-y-auto">
          {items.map((item) => (
            <Link
              key={item.name}
              href={item.href}
              className={`flex items-center gap-3.5 px-4 py-3 rounded-xl text-sm font-medium transition-all duration-200 group ${
                isActive(item.href)
                  ? "bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-md shadow-indigo-600/10 font-semibold"
                  : "text-gray-400 hover:bg-gray-900/60 hover:text-gray-100"
              }`}
            >
              <span className={`transition-transform duration-200 group-hover:scale-105 ${
                isActive(item.href) ? "text-white" : "text-gray-500 group-hover:text-gray-300"
              }`}>
                {item.icon}
              </span>
              <span>{item.name}</span>
            </Link>
          ))}
        </nav>

        {/* AI Engine Status Card */}
        <div className="p-4 border-t border-gray-900 bg-gray-950/60">
          <div className="rounded-xl border border-gray-900 bg-gray-900/30 p-3.5 flex flex-col gap-3">
            <div className="flex items-center justify-between">
              <span className="text-[10px] text-gray-500 font-bold uppercase tracking-wider">
                Motor de IA
              </span>
              <div className="flex items-center gap-1.5">
                <span className={`w-2.5 h-2.5 rounded-full relative flex ${
                  status.running ? "text-emerald-500" : "text-amber-500"
                }`}>
                  <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75 ${
                    status.running ? "bg-emerald-500" : "bg-amber-500"
                  }`} />
                  <span className={`relative inline-flex rounded-full h-2.5 w-2.5 ${
                    status.running ? "bg-emerald-500" : "bg-amber-500"
                  }`} />
                </span>
                <span className={`text-xs font-semibold ${
                  status.running ? "text-emerald-400" : "text-amber-400"
                }`}>
                  {status.running ? "Activo" : "Apagado"}
                </span>
              </div>
            </div>

            {status.running && status.model ? (
              <div className="bg-black/20 rounded-lg px-2.5 py-2 border border-gray-900">
                <p className="text-[10px] text-gray-500 font-medium">Modelo cargado</p>
                <p className="text-xs font-semibold text-gray-300 truncate mt-0.5">
                  {status.model}
                </p>
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                <p className="text-[11px] text-gray-400 leading-relaxed">
                  El servidor local está inactivo. Carga un modelo para habilitar la IA.
                </p>
                <button
                  onClick={() => router.push("/models")}
                  className="w-full py-1.5 rounded-lg bg-gray-900 hover:bg-gray-800 border border-gray-800 text-[11px] font-bold text-gray-200 hover:text-white transition-colors duration-200 cursor-pointer"
                >
                  Gestionar Modelos
                </button>
              </div>
            )}
          </div>
        </div>
      </aside>

      {/* Mobile Bottom Navigation Bar */}
      <nav className="flex md:hidden fixed bottom-0 left-0 right-0 h-16 border-t border-gray-900 bg-gray-950/95 backdrop-blur-md z-30 justify-around items-center px-2">
        {items.map((item) => (
          <Link
            key={item.name}
            href={item.href}
            className={`flex flex-col items-center justify-center gap-1 flex-1 py-1 ${
              isActive(item.href) ? "text-indigo-400 font-semibold" : "text-gray-500"
            }`}
          >
            <span className="transition-transform duration-200">{item.icon}</span>
            <span className="text-[10px] truncate max-w-full px-1">{item.name}</span>
          </Link>
        ))}
      </nav>
    </>
  );
}
