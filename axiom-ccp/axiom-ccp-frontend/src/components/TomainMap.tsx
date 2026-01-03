import React, { useEffect, useState } from 'react';
import { useEnvironment } from '../context/EnvironmentContext';
import { Network, Server } from 'lucide-react';

interface TomainMetadata {
  environment: string;
  database_url: string;
  cache_url: string;
  message_queue: string;
}

export const TomainMap: React.FC = () => {
  const { environment } = useEnvironment();
  const [metadata, setMetadata] = useState<TomainMetadata | null>(null);
  const [loading, setLoading] = useState(false);

  // Mock fetching connection metadata from the CCP Backend
  useEffect(() => {
    setLoading(true);
    fetch(`http://localhost:3000/api/v1/tomains/last-mile-tech/scheduling/fleet-mgmt/resolve?color=${environment}`)
      .then(res => res.json())
      .then(data => {
        setMetadata(data);
        setLoading(false);
      })
      .catch(err => {
        console.error("Failed to fetch tomain data:", err);
        setLoading(false);
      });
  }, [environment]);

  // Color theme classes mapped from the selected context
  const getEnvStyles = (env: string) => {
    switch (env) {
      case 'DEV': return 'text-cyan-400 bg-cyan-900/20 border-cyan-900 shadow-[0_0_15px_rgba(6,182,212,0.2)]';
      case 'QA': return 'text-yellow-500 bg-yellow-900/20 border-yellow-900 shadow-[0_0_15px_rgba(234,179,8,0.2)]';
      case 'STAGING': return 'text-purple-500 bg-purple-900/20 border-purple-900 shadow-[0_0_15px_rgba(168,85,247,0.2)]';
      case 'PROD': return 'text-red-500 bg-red-900/20 border-red-900 shadow-[0_0_15px_rgba(239,68,68,0.2)]';
      default: return 'text-cyan-400 bg-cyan-900/20 border-cyan-900';
    }
  };

  const getEnvBadge = (env: string) => {
    switch (env) {
      case 'DEV': return 'bg-cyan-400 text-cyan-900';
      case 'QA': return 'bg-yellow-500 text-yellow-900';
      case 'STAGING': return 'bg-purple-500 text-purple-900';
      case 'PROD': return 'bg-red-500 text-red-900';
      default: return 'bg-gray-500 text-gray-900';
    }
  };

  const currentStyles = getEnvStyles(environment);

  return (
    <div className={`p-6 rounded-lg border shadow-xl ${currentStyles} transition-all duration-700`}>
       <div className="flex items-center gap-3 mb-6 border-b border-white/5 pb-4">
        <Network size={24} className="theme-accent" />
        <h2 className="text-xl font-bold">The Tomain Map</h2>
       </div>

       <div className="space-y-4 font-mono text-sm">
         <div className="flex items-center gap-2">
            <span className="text-gray-500">Namespace:</span>
            <span className="bg-dark-surface px-2 py-1 rounded">last-mile-tech ➔ scheduling ➔ fleet-mgmt</span>
         </div>

         {loading ? (
             <div className="animate-pulse text-gray-500">Resolving CCP Bindings...</div>
         ) : metadata ? (
             <div className="bg-dark-surface p-4 rounded border border-dark-border space-y-3 shadow-inner">
                 <div className="flex justify-between items-center border-b border-dark-border pb-2">
                     <span className="text-gray-400">Context Resolution</span>
                     <span className={`px-2 py-0.5 rounded text-[10px] font-black tracking-tighter uppercase leading-none ${getEnvBadge(environment)}`}>
                        {metadata.environment}
                     </span>
                 </div>
                 
                 <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
                    <div className="p-3 bg-dark-bg rounded border border-dark-border">
                        <div className="text-gray-500 text-xs mb-1">Database Intent</div>
                        <div className="truncate theme-accent">{metadata.database_url}</div>
                    </div>
                    <div className="p-3 bg-dark-bg rounded border border-dark-border">
                        <div className="text-gray-500 text-xs mb-1">Cache Intent</div>
                        <div className="truncate theme-accent">{metadata.cache_url}</div>
                    </div>
                    <div className="p-3 bg-dark-bg rounded border border-dark-border">
                        <div className="text-gray-500 text-xs mb-1">Queue Intent</div>
                        <div className="truncate theme-accent">{metadata.message_queue}</div>
                    </div>
                 </div>
             </div>
         ) : (
             <div className="text-red-500">Failed to connect to CCP Backend on port 3000. Is it running?</div>
         )}
       </div>

       <div className="mt-6 flex justify-end">
          <button className={`flex items-center gap-2 px-4 py-2 rounded bg-dark-surface hover:bg-dark-border border border-dark-border transition-colors text-sm`}>
            <Server size={14} className="theme-accent" /> Hot-swap Wasm Module
          </button>
       </div>
    </div>
  );
};
