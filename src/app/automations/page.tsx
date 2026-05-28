import { AutomationHub } from "@/components/Automations/AutomationHub";

export const metadata = {
  title: "Automatización de Tareas - Zeldrix",
  description: "Hub de automatizaciones repetitivas y flujos de trabajo de oficina pesados con IA.",
};

export default function AutomationsPage() {
  return (
    <main className="min-h-screen bg-gradient-to-b from-gray-50/50 to-gray-100/30 dark:from-gray-950 dark:to-gray-900/40 px-6 py-10 md:px-12 md:py-16">
      <div className="max-w-7xl mx-auto">
        <AutomationHub />
      </div>
    </main>
  );
}
