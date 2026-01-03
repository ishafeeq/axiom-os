import React from 'react';
import { Link } from 'react-router-dom';
import { useEnvironment } from '../context/EnvironmentContext';
import { useTomains, useTomainDetail, useDeleteTomain, type Tomain } from '../api/tomainClient';
import { HardDrive, Activity, Database, Network, ChevronRight, Zap, ShieldCheck, ListMusic, Globe, Trash2 } from 'lucide-react';
import { DeleteServiceModal } from './DeleteServiceModal';

export const ServerGrid: React.FC = () => {
  const { environment } = useEnvironment();
  const { data: tomains, isLoading, isError } = useTomains();
  const [selectedId, setSelectedId] = React.useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = React.useState<Tomain | null>(null);

  // FILTERING LOGIC: Hide if not promoted to selected environment (Service OR Feature)
  const filteredTomains = tomains?.filter(t => {
     if (environment === 'DEV') return true;
     
     // Visible if service itself is promoted
     if (t.wasm_hashes?.[environment]) return true;
     
     // Visible if any feature is promoted to this environment
     if (t.features && Object.values(t.features).some(f => f.environments?.includes(environment))) {
        return true;
     }

     return false;
  });

  const deleteMutation = useDeleteTomain();

  const selectedTomain = tomains?.find(t => t.id === (selectedId || (tomains.length > 0 ? tomains[0].id : null)));

  return (
    <div className="space-y-8 animate-fade-in">
      <div className="flex items-center justify-between border-b border-dark-border pb-6">
        <div>
          <div className="flex items-center gap-3">
             <h2 className="text-3xl font-bold tracking-tight text-white">
               {selectedTomain ? (
                 <>
                   {selectedTomain.name.split('.').slice(0, -1).join('.')} 
                   <span className="text-gray-500 font-normal ml-3 text-2xl">|</span>
                   <span className="text-gray-400 font-medium ml-3 text-xl">{selectedTomain.team_name || 'No Team'}</span>
                 </>
               ) : 'Cluster Overview'}
             </h2>
             <select 
               className="bg-dark-surface border border-dark-border text-sm rounded-lg px-2 py-1 focus:ring-1 theme-accent outline-none ml-4"
               onChange={(e) => setSelectedId(e.target.value)}
               value={selectedId || ''}
             >
               {filteredTomains?.map(t => (
                 <option key={t.id} value={t.id}>{t.name}</option>
               ))}
             </select>
          </div>
          <p className="text-gray-500 text-sm mt-2 font-medium tracking-wide">
             CCP <span className="mx-1 text-gray-700">/</span> {selectedTomain?.name} <span className="mx-1 text-gray-700">/</span> Micro-Services
          </p>
        </div>
        <div className="flex items-center gap-2 text-sm font-mono bg-dark-surface px-3 py-1.5 rounded-md border border-dark-border shadow-inner">
          <Activity size={16} className={isError ? "text-red-500" : "text-green-500"} />
          <span>
            {isLoading ? 'Syncing...' : isError ? 'Registry Offline' : `${filteredTomains?.length || 0} ${environment} Visible Services`}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 gap-6">
        {isLoading && <p className="text-gray-500 text-sm font-mono">Loading telemetry from CCP...</p>}
        {isError && <p className="text-red-500 text-sm font-mono">Failed to fetch Tomains.</p>}
        {filteredTomains?.map(tomain => (
          <ServerCard 
            key={tomain.id} 
            tomain={tomain} 
            colorContext={environment || 'GREEN'} 
            onDelete={() => setDeleteTarget(tomain)}
          />
        ))}
      </div>

      {deleteTarget && (
        <DeleteServiceModal 
          isOpen={!!deleteTarget}
          onClose={() => setDeleteTarget(null)}
          serviceName={deleteTarget.name}
          onConfirm={async () => {
             await deleteMutation.mutateAsync(deleteTarget.id);
             if (selectedId === deleteTarget.id) setSelectedId(null);
          }}
        />
      )}
    </div>
  );
};

const ServerCard: React.FC<{ tomain: Tomain, colorContext: string, onDelete: () => void }> = ({ tomain, colorContext, onDelete }) => {
  const { data: detail, isLoading } = useTomainDetail(tomain.name, colorContext);

  const isHealthy = tomain.health_status === 'Healthy' || tomain.health_status === 'Active';
  const isInactive = tomain.health_status === 'Inactive' || tomain.health_status === 'Stopped';

  return (
    <div className={`rounded-xl border-5 ${isHealthy ? 'border-green-500/50 hover:border-green-700' : 'border-red-500/50 hover:border-red-700'} bg-dark-surface transition-all duration-300 flex flex-col shadow-lg group relative overflow-hidden h-full ${isHealthy ? 'shadow-[0_0_15px_rgba(34,197,94,0.1)]' : 'shadow-[0_0_15px_rgba(239,68,68,0.1)]'}`}>
      {/* Background Health Glow */}
      <div className={`absolute top-0 left-0 w-full h-[92px] ${isHealthy ? 'bg-green-600' : 'bg-red-600'} opacity-[0.05]`} />

      {/* Clickable Header Section */}
      <Link 
        to={`/package/${tomain.id}`} 
        className="flex justify-between items-start p-5 mb-0 hover:bg-dark-bg/20 transition-colors border-b border-transparent hover:border-dark-border group/header relative z-10"
      >
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-md theme-bg-subtle theme-accent transition-transform duration-500 group-hover/header:rotate-12`}>
             <HardDrive size={20} />
          </div>
          <div>
            <h3 className="font-bold text-gray-100 group-hover/header:text-white transition-colors">
              {tomain.package_name || tomain.name.split('.').pop()}
            </h3>
            <div className="flex gap-2 items-center mt-1">
              <span className={`text-[10px] px-2 py-0.5 rounded-full border ${isHealthy ? 'text-green-500 border-green-900/50' : 'text-red-500 border-red-900/50'} uppercase tracking-wider font-bold`}>
                {tomain.health_status}
              </span>
              <span className="text-[10px] text-gray-500 font-mono tracking-tighter">
                {tomain.team_name}
              </span>
            </div>
          </div>
        </div>
        <div className="p-2 text-gray-400 group-hover/header:text-white group-hover/header:translate-x-1 transition-all">
          <ChevronRight size={18} />
        </div>
      </Link>

      <div className="p-5 flex flex-col gap-4 relative z-10">
        <div className="grid grid-cols-2 gap-3">
           {/* Rate Limit */}
           <div className="bg-dark-bg p-3 rounded-lg border border-dark-border flex flex-col gap-1">
              <span className="text-[10px] text-gray-500 uppercase font-bold flex items-center gap-1">
                 <Zap size={10} className="text-yellow-500"/> Rate Limit
              </span>
              <span className="text-sm font-mono text-gray-200">
                 {tomain.rate_limit && tomain.rate_limit !== 'null' ? tomain.rate_limit : 'Default (1k)'}
              </span>
           </div>

           {/* Security */}
           <div className="bg-dark-bg p-3 rounded-lg border border-dark-border flex flex-col gap-1">
              <span className="text-[10px] text-gray-500 uppercase font-bold flex items-center gap-1">
                 <ShieldCheck size={10} className="text-blue-500"/> Auth Guard
              </span>
              <span className="text-sm font-mono text-gray-200">
                 {tomain.has_public_key ? 'JWT Validated' : 'Public'}
              </span>
           </div>

           {/* API Count */}
           <div className="bg-dark-bg p-3 rounded-lg border border-dark-border flex flex-col gap-1">
              <span className="text-[10px] text-gray-500 uppercase font-bold flex items-center gap-1">
                 <ListMusic size={10} className="text-green-500"/> API Count
              </span>
              <span className="text-sm font-mono text-gray-200">
                 {tomain.api_count} Endpoints
              </span>
           </div>

           {/* Context */}
           <div className="bg-dark-bg p-3 rounded-lg border border-dark-border flex flex-col gap-1">
              <span className="text-[10px] text-gray-500 uppercase font-bold flex items-center gap-1">
                 <Globe size={10} className="text-purple-500"/> Context
              </span>
              <span className="text-sm font-mono text-gray-200">
                 {colorContext}
              </span>
           </div>
        </div>

        {/* Delete Action (Bottom Bar) */}
        <div className="mt-auto pt-4 border-t border-dark-border flex justify-between items-center">
           <div className="text-[10px] text-gray-500 italic">
              ID: {tomain.id}
           </div>
           <button 
             disabled={!isInactive}
             onClick={(e) => { e.stopPropagation(); onDelete(); }}
             className={`flex items-center gap-2 px-3 py-1.5 rounded text-xs font-bold transition-all ${
               isInactive 
               ? 'bg-red-900/10 text-red-500 border border-red-900/50 hover:bg-red-500 hover:text-white' 
               : 'bg-dark-bg text-gray-700 border border-dark-border cursor-not-allowed opacity-50'
             }`}
           >
             <Trash2 size={12} /> Delete Service
           </button>
        </div>

        {/* Database/Queue Detail (Subtle) */}
        <div className="mt-2 pt-2 border-t border-dark-border space-y-1.5">
           {!isLoading && detail && (
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center gap-2 text-[10px] text-gray-500 uppercase font-bold">
                  <Database size={10} /> {detail.database}
                </div>
                <div className="flex items-center gap-2 text-[10px] text-gray-500 uppercase font-bold">
                  <Network size={10} /> {detail.message_bus}
                </div>
              </div>
           )}
        </div>
      </div>
    </div>
  );
};
