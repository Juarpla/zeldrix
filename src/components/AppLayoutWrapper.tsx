"use client";

import { useEffect } from "react";
import { usePathname } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "@/components/Sidebar";

export default function AppLayoutWrapper({
  children,
}: {
  children: React.ReactNode;
}) {
  const pathname = usePathname();
  const isSpotlight = pathname === "/spotlight";

  // Global key combination to toggle spotlight window
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      // Toggle with Alt+Space or Ctrl+Space or Cmd+K
      if (
        (e.altKey && e.code === "Space") ||
        (e.ctrlKey && e.code === "Space") ||
        ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k")
      ) {
        e.preventDefault();
        try {
          await invoke("toggle_spotlight");
        } catch (err) {
          console.error("Failed to toggle spotlight window:", err);
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  if (isSpotlight) {
    // Elegant, completely transparent, borderless container for spotlight view
    return (
      <div className="w-screen h-screen overflow-hidden bg-transparent select-none">
        {children}
      </div>
    );
  }

  return (
    <div className="flex min-h-screen h-screen overflow-hidden">
      {/* Global Premium Left Sidebar / Mobile Bottom Bar */}
      <Sidebar />

      {/* Main Layout Area */}
      <div className="flex-1 flex flex-col min-w-0 md:pl-64 pb-16 md:pb-0 h-full overflow-hidden bg-gray-950">
        <div className="flex-1 h-full overflow-y-auto min-w-0">
          {children}
        </div>
      </div>
    </div>
  );
}
