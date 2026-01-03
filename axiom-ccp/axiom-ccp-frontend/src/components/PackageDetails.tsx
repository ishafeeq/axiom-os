import React from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { ArrowLeft, ExternalLink, ShieldCheck, Terminal, BookOpen, Clock, Activity, Zap, RefreshCcw, Lock, UserCircle } from 'lucide-react';
import { FeatureStatusTable } from './FeatureStatusTable';
import { ContextSwitcher } from './ContextSwitcher';
import { useTomain } from '../api/tomainClient';
import { useEnvironment } from '../context/EnvironmentContext';

export const PackageDetails: React.FC<{ isExplorer?: boolean }> = ({ isExplorer }) => {
  const { environment } = useEnvironment();
  const { packageId } = useParams<{ packageId: string }>();
  const navigate = useNavigate();
  const { data: tomain, isLoading } = useTomain(packageId || '');

  if (!packageId) return <div className="p-10 text-red-500">Invalid Package ID</div>;

  const hasServiceInEnv = environment === 'DEV' || !!(tomain?.wasm_hashes && tomain.wasm_hashes[environment]);

  return (
    <div className="flex flex-col h-full gap-8 animate-in fade-in duration-700">
      <div className="flex items-center justify-between border-b border-white/5 pb-6">
        <div className="flex items-center gap-5">
          <button 
            onClick={() => navigate('/')}
            className="p-2.5 bg-dark-surface rounded-xl border border-dark-border hover:border-gray-500 transition-all text-gray-400 hover:text-white group"
          >
            <ArrowLeft size={20} className="group-hover:-translate-x-1 transition-transform" />
          </button>
          <div>
            <div className="flex items-center gap-3">
               <h2 className="text-3xl font-black tracking-tighter flex items-center gap-3">
                 <ShieldCheck size={28} className="theme-accent" />
                 {tomain?.package_name || packageId.split('.').pop()}
               </h2>
               <span className="bg-dark-surface px-2 py-0.5 rounded border border-dark-border text-[10px] font-mono text-gray-500 uppercase tracking-widest">
                  {packageId}
               </span>
            </div>
            <p className="text-gray-500 text-sm mt-1 font-medium">Live Perspective: <span className="text-gray-300">Wasm Kernel v{tomain?.created_at.slice(0, 10) || '0.1.0'}</span></p>
          </div>
        </div>
        
        <div className="flex items-center gap-4">
            <div className="hidden md:flex flex-col items-end mr-4">
                <span className="text-[10px] text-gray-600 uppercase font-bold tracking-widest">Health Status</span>
                <span className={`text-xs font-bold ${tomain?.health_status === 'Healthy' || tomain?.health_status === 'Active' ? 'text-green-500' : 'text-red-500'}`}>
                   ● {tomain?.health_status || 'Checking...'}
                </span>
            </div>
            {isExplorer ? (
              <div className="flex items-center gap-6">
                 <ContextSwitcher />
                 <div className="w-px h-8 bg-dark-border mx-2"></div>
                 <div className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors cursor-pointer p-2 rounded-full hover:bg-white/5">
                    <UserCircle size={28} className="theme-accent" />
                 </div>
              </div>
            ) : (
              <Link 
                to={`/package/${packageId}/explorer`} 
                target="_blank" 
                className="flex items-center gap-2 text-xs font-bold uppercase tracking-widest bg-dark-surface px-5 py-2.5 rounded-lg border border-dark-border hover:border-gray-500 transition-all text-gray-300 hover:text-white shadow-lg"
              >
                <ExternalLink size={14} />
                Full Explorer
              </Link>
            )}
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-8">
          {/* Feature Status Table */}
          <section>
            <div className="flex items-center gap-2 mb-4">
                <Activity size={18} className="theme-accent" />
                <h3 className="text-sm font-bold uppercase tracking-widest text-gray-400">Environment Perspective</h3>
            </div>
            <FeatureStatusTable tomainId={packageId} />
          </section>

          {/* API Documentation / Reflection */}
          {hasServiceInEnv ? (
            <section className="flex flex-col gap-4">
              <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                      <BookOpen size={18} className="theme-accent" />
                      <h3 className="text-sm font-bold uppercase tracking-widest text-gray-400">Documentation & Reflection</h3>
                  </div>
                  <span className="text-[10px] font-mono text-gray-600">PORT 9000 ➔ 3000</span>
              </div>
              <div className="h-[650px] border border-dark-border rounded-2xl overflow-hidden bg-black/40 shadow-inner relative group/docs">
                  <iframe 
                    src={`http://localhost:3000/api/v1/docs/${packageId}`}
                    className="w-full h-full border-0 grayscale-[0.2] hover:grayscale-0 transition-all duration-500"
                    title={`Axiom API Explorer - ${packageId}`}
                  />
                  {!isLoading && !tomain?.apis?.length && (
                      <div className="absolute inset-0 pointer-events-none flex flex-col items-center justify-center bg-dark-bg/80 backdrop-blur-sm opacity-0 group-docs:hover:opacity-100 transition-opacity">
                          <Terminal size={48} className="text-gray-700 mb-4" />
                          <p className="text-gray-500 font-mono text-sm max-w-xs text-center">
                             Waiting for kernel reflection signal from Shell...
                          </p>
                      </div>
                  )}
              </div>
            </section>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 border border-dashed border-dark-border rounded-2xl bg-dark-bg/20">
                <ShieldCheck size={48} className="text-gray-600 mb-4" />
                <h3 className="text-lg font-bold text-gray-400">No Active Service in {environment}</h3>
                <p className="text-sm text-gray-500 mt-2 max-w-sm text-center">Deploy or promote this service to the {environment} environment to unlock API reflection and service specifications.</p>
            </div>
          )}
        </div>

        <div className="space-y-8">
            {/* Quick API Stats */}
            {hasServiceInEnv && (
                <div className="bg-dark-surface border border-dark-border rounded-2xl p-6 shadow-xl space-y-6">
                    <div className="flex items-center gap-3 border-b border-white/5 pb-4">
                        <Terminal size={20} className="theme-accent" />
                        <h3 className="font-bold">Service Specifications</h3>
                        <span className="ml-auto bg-dark-bg px-2 py-0.5 rounded text-xs font-mono">{tomain?.apis?.length || 0}</span>
                    </div>
                    
                    <div className="space-y-3">
                        {isLoading ? (
                            <div className="space-y-2 animate-pulse">
                                {[1,2,3].map(i => <div key={i} className="h-10 bg-white/5 rounded-lg" />)}
                            </div>
                        ) : (tomain?.apis || []).map((api: any, idx: number) => (
                            <div key={idx} className="flex flex-col items-start p-3 bg-dark-bg rounded-xl border border-white/5 hover:border-theme-accent transition-colors group/api">
                                <div className="flex w-full items-center justify-between">
                                    <div className="flex flex-col">
                                        <span className="text-xs font-mono font-bold group-hover/api:text-white transition-colors capitalize">{api.name}</span>
                                        <span className="text-[10px] text-gray-600 uppercase font-black">{api.method}</span>
                                    </div>
                                    <div className="text-[10px] font-mono text-gray-500 group-hover/api:text-theme-accent transition-colors">
                                        {api.params.length} params
                                    </div>
                                </div>
                                <div className="mt-3 grid grid-cols-2 gap-y-2 gap-x-1 text-[11px] text-gray-400 w-full border-t border-white/5 pt-2">
                                    <div className="flex items-center gap-1.5"><Zap size={11} className="text-yellow-500" /> Rate-limit: <span className="text-gray-300 font-mono pl-1">{api.rate_limit ?? 'Default (1k)'}</span></div>
                                    <div className="flex items-center gap-1.5"><ShieldCheck size={11} className="text-red-500" /> Breaker: <span className="text-gray-300 font-mono pl-1">{api.circuit_breaker ?? 'None'}</span></div>
                                    <div className="flex items-center gap-1.5"><RefreshCcw size={11} className="text-blue-500" /> Retry: <span className="text-gray-300 font-mono pl-1">{api.retry_policy ?? 'None'}</span></div>
                                    <div className="flex items-center gap-1.5"><Lock size={11} className="text-green-500" /> Auth: <span className="text-gray-300 font-mono pl-1">{api.auth ?? 'None'}</span></div>
                                </div>
                            </div>
                        ))}
                        {!isLoading && (!tomain?.apis || tomain.apis.length === 0) && (
                            <div className="py-10 text-center text-gray-600 italic text-sm">
                               No exported APIs found in Wit manifest.
                            </div>
                        )}
                    </div>
                    
                    <div className="pt-4 border-t border-white/5">
                        <button className="w-full py-2.5 rounded-xl bg-white/5 hover:bg-white/10 border border-dark-border transition-all text-xs font-bold uppercase tracking-widest text-gray-400 hover:text-white">
                            Purge Edge Cache
                        </button>
                    </div>
                </div>
            )}

            {/* Event Log or metadata */}
            <div className="bg-dark-bg border border-dark-border rounded-2xl p-6 space-y-4">
                <div className="flex items-center gap-3">
                    <Clock size={16} className="text-gray-500" />
                    <h3 className="text-xs font-bold uppercase tracking-widest text-gray-500">Service Timeline</h3>
                </div>
                <div className="space-y-3">
                    <div className="flex gap-3">
                        <div className="w-1.5 h-1.5 rounded-full bg-green-500 mt-1" />
                        <div className="flex flex-col">
                            <span className="text-[11px] font-bold">Successfully Bound (@edge-01)</span>
                            <span className="text-[10px] text-gray-600">Just now</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
      </div>
    </div>
  );
};
