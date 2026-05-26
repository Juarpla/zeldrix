import { TemplateCatalog } from "@/components/templates";

export default function TemplatesPage() {
  return (
    <main className="min-h-screen bg-gradient-to-b from-background to-muted/20">
      <div className="container max-w-6xl mx-auto px-4 py-8">
        <TemplateCatalog />
      </div>
    </main>
  );
}