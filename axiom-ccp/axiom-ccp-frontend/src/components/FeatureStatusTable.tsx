import { useEnvironment } from '../context/EnvironmentContext';
import { useTomains } from '../api/tomainClient';
import { Zap, ArrowRight } from 'lucide-react';

export const FeatureStatusTable: React.FC<{ tomainId: string }> = ({ tomainId }) => {
  const { environment } = useEnvironment();
  const { data: tomains } = useTomains();
  const tomain = tomains?.find(t => t.id === tomainId);

  if (!tomain) return null;

  const envLabel = environment.toLowerCase();
  const allFeatures = tomain.features ? Object.entries(tomain.features) : [];
  
  const envOrder = ['DEV', 'QA', 'STAGING', 'PROD'];
  
  // FILTERING LOGIC: Only show the feature in the highest environment it has been promoted to.
  const features = allFeatures.filter(([_, detail]) => {
     const featureEnvs = detail.environments && detail.environments.length > 0 ? detail.environments : ['DEV'];
     
     // Find the highest environment level this feature has reached
     const highestEnvIdx = Math.max(...featureEnvs.map((e: string) => envOrder.indexOf(e)));
     
     // Check if the current context environment matches that highest level
     return envOrder.indexOf(environment) === highestEnvIdx;
  });
  
  return (
    <div className="bg-dark-surface border border-dark-border rounded-xl overflow-hidden shadow-lg p-4 mb-6">
      <h3 className="text-xs font-bold uppercase tracking-widest text-gray-500 mb-4 flex items-center gap-2">
        <Zap size={14} className="text-yellow-500" /> Ongoing features in {envLabel.charAt(0).toUpperCase() + envLabel.slice(1)}
      </h3>
      
      <div className="space-y-3">
        {features.length === 0 ? (
          <div className="py-8 text-center text-sm text-gray-500 italic bg-dark-bg/20 rounded-lg border border-dashed border-dark-border">
            No {envLabel} feature found for this service.
          </div>
        ) : (
          features.map(([name, detail]: [string, any]) => (
            <div key={name} className="flex items-center justify-between p-3 bg-dark-bg/40 border border-dark-border rounded-lg group hover:border-gray-600 transition-all">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-dark-surface rounded border border-dark-border group-hover:theme-border transition-colors">
                  <Zap size={14} className="theme-accent" />
                </div>
                <div>
                  <div className="text-sm font-bold text-gray-200">{name}</div>
                  <div className="text-[10px] font-mono text-gray-500">
                      Branch: {detail.branch || 'main'}
                      {detail.commits_ahead !== undefined && (
                         <span className="ml-2 px-1.5 py-0.5 rounded bg-gray-800 border border-gray-700 text-gray-300">
                             {detail.commits_ahead} commits
                         </span>
                      )}
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-4">
                <div className="flex flex-col items-end">
                  <span className="text-[10px] font-bold uppercase tracking-tighter text-green-500">{detail.status}</span>
                  <span className="text-[9px] font-mono text-gray-600">{detail.wasm_hash?.substring(0, 8)}</span>
                </div>
                <div className="p-1.5 rounded-full bg-dark-surface border border-dark-border text-gray-400 group-hover:text-white transition-colors">
                  <ArrowRight size={14} />
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
