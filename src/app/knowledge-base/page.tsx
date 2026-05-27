import KnowledgeBaseDashboard from '@/components/KnowledgeBase/KnowledgeBaseDashboard';

export const metadata = {
  title: 'Knowledge Base - Zeldrix',
  description: 'Manage virtual folders, disk storage, and vector ingestion pipeline segments.',
};

export default function KnowledgeBasePage() {
  return (
    <main className="min-h-screen bg-gradient-to-b from-gray-50/50 to-gray-100/30 dark:from-gray-950 dark:to-gray-900/40 px-6 py-10 md:px-12 md:py-16">
      <div className="max-w-7xl mx-auto">
        <KnowledgeBaseDashboard />
      </div>
    </main>
  );
}
