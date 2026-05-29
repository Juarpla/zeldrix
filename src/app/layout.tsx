import type { Metadata } from "next";
import "./globals.css";
import Sidebar from "@/components/Sidebar";

export const metadata: Metadata = {
  title: "Zeldrix",
  description: "Tauri + Next.js Local AI Suite",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning className="dark">
      <body className="antialiased bg-gray-950 text-white min-h-screen">
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
      </body>
    </html>
  );
}
