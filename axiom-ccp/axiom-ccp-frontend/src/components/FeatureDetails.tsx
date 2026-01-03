import React from 'react';
import { useParams, Link } from 'react-router-dom';
import { Clock, BookOpen, GitBranch } from 'lucide-react';

/**
 * Placeholder feature‑detail view.
 * Shows commit ID, commit timestamp, and a short description.
 * In a real implementation this would call a backend endpoint
 * (e.g. GET /api/v1/tomains/{id}/features/{name}) to fetch the data.
 */
export const FeatureDetails: React.FC = () => {
  const { packageId, featureName } = useParams<{
    packageId: string;
    featureName: string;
  }>();

  // Mock data – replace with real API call when available
  const mockFeature = {
    commit_id: '58fab17bf4aa0e8a45ce56821d50d29a914efd10',
    commit_time: '2026-02-26 12:11:30',
    description:
      'Initial implementation of UI validation and OTP endpoint. Includes rate‑limit and auth policies.',
  };

  if (!packageId || !featureName) {
    return (
      <div className="p-10 text-red-500">
        Invalid feature URL.
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6 bg-dark-surface rounded-xl shadow-lg border border-dark-border w-full">
      <Link
        to={`/package/${packageId}`}
        className="inline-flex items-center gap-2 text-sm text-gray-400 hover:text-white mb-6"
      >
        <BookOpen size={16} />
        Back to Service
      </Link>

      <div className="flex items-center gap-4 mb-8">
        <div className="p-3 bg-dark-bg rounded-lg border border-dark-border theme-accent">
            <GitBranch size={24} />
        </div>
        <div>
            <h2 className="text-2xl font-bold flex items-center gap-2 text-white">
                Feature: {featureName}
            </h2>
            <span className="text-xs font-mono text-gray-500 uppercase tracking-widest">{packageId}</span>
        </div>
      </div>

      <div className="space-y-6 text-gray-300">
        <div className="grid grid-cols-2 gap-4">
            <div className="bg-dark-bg p-4 rounded-lg border border-dark-border flex flex-col gap-1">
                <span className="text-[10px] text-gray-500 uppercase font-bold flex items-center gap-1">
                    <Clock size={12} className="text-blue-500"/> Commit ID
                </span>
                <span className="text-sm font-mono text-gray-200 truncate">
                    {mockFeature.commit_id}
                </span>
            </div>
            <div className="bg-dark-bg p-4 rounded-lg border border-dark-border flex flex-col gap-1">
                <span className="text-[10px] text-gray-500 uppercase font-bold flex items-center gap-1">
                    <Clock size={12} className="text-green-500"/> Committed At
                </span>
                <span className="text-sm font-mono text-gray-200 truncate">
                    {mockFeature.commit_time}
                </span>
            </div>
        </div>

        <div className="bg-dark-bg p-5 rounded-lg border border-dark-border mt-4">
          <h3 className="font-semibold text-gray-400 uppercase text-xs tracking-widest mb-3 border-b border-white/5 pb-2">Readme / Description</h3>
          <p className="text-sm leading-relaxed">{mockFeature.description}</p>
        </div>
      </div>
    </div>
  );
};
