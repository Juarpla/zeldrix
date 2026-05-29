import DownloadManager from "@/components/DownloadManager";

export const metadata = {
  title: "Gestor de Modelos - Zeldrix",
  description: "Descarga y gestiona modelos de lenguaje local para Zeldrix.",
};

export default function ModelsPage() {
  return (
    <main className="min-h-screen bg-gray-900 text-white">
      <DownloadManager />
    </main>
  );
}
