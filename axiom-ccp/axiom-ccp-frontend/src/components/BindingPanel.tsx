import React, { useState } from 'react';
import { Settings, Link2, ArrowRight, X, Terminal } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';

interface PhysicalBinding {
  id: string;
  tomain_id: string;
  alias: string;
  physical_url: string;
  environment: string;
}

export const BindingPanel: React.FC = () => {
  const [showCliDialog, setShowCliDialog] = useState(false);

  const { data: bindings, isLoading, error } = useQuery<PhysicalBinding[]>({
    queryKey: ['bindings'],
    queryFn: async () => {
      const res = await fetch('http://localhost:3000/api/v1/bindings');
      if (!res.ok) throw new Error('Failed to fetch bindings');
      return res.json();
    }
  });

  return (
    <div className="bg-dark-surface p-6 rounded-xl border border-dark-border shadow-md h-full flex flex-col relative">
       <div className="flex items-center gap-3 mb-6 border-b border-dark-border pb-4">
          <Settings className="theme-accent transition-colors" size={24} />
          <div className="flex-1">
            <h2 className="text-xl font-bold tracking-tight">Egress Registry (Pillar #9)</h2>
            <p className="text-xs text-gray-500">Live service-binding mappings for outbound calls</p>
          </div>
       </div>

       <div className="space-y-4 flex-1 overflow-y-auto">
         {isLoading && <div className="text-center p-4 text-gray-400">Loading bindings...</div>}
         {error && <div className="text-center p-4 text-red-400">Failed to load bindings</div>}
         
         {!isLoading && bindings?.length === 0 && (
            <div className="text-center p-8 border border-dashed border-dark-border rounded-lg text-gray-500 italic">
               No bindings registered yet. Run `ax bind` to add service mappings.
            </div>
         )}

         {bindings?.map((b) => (
            <div key={b.id} className="flex flex-col p-4 bg-dark-bg border border-dark-border rounded-lg group hover:border-theme-accent transition-all duration-300">
               <div className="flex items-center justify-between mb-3 text-xs uppercase tracking-widest font-bold text-gray-500">
                  <span>Context: {b.environment}</span>
                  <span className="theme-accent-text">Active Egress Guard</span>
               </div>
               
               <div className="flex items-center gap-4 bg-dark-surface p-3 rounded-lg border border-dark-border">
                  <div className="flex flex-col items-center flex-1">
                    <span className="text-[10px] text-gray-500 mb-1">Local App</span>
                    <div className="bg-gray-800 px-3 py-1 rounded text-xs font-mono truncate max-w-[120px] shadow-inner text-gray-300">
                        {b.tomain_id.split('.').pop()}
                    </div>
                  </div>

                  <ArrowRight size={16} className="text-gray-600 shrink-0" />

                  <div className="flex flex-col items-center flex-1">
                    <span className="text-[10px] text-gray-500 mb-1">Alias</span>
                    <div className="bg-theme-bg-subtle border border-theme-accent/30 px-3 py-1 rounded text-xs theme-accent font-bold flex items-center gap-2 shadow-sm">
                        <Link2 size={12} />
                        {b.alias}
                    </div>
                  </div>

                  <ArrowRight size={16} className="text-gray-600 shrink-0" />

                  <div className="flex flex-col items-center flex-[1.5]">
                    <span className="text-[10px] text-gray-500 mb-1">Actual Service</span>
                    <div className="bg-gray-900 border border-gray-700 px-3 py-1 rounded text-xs font-mono text-green-400 truncate w-full flex justify-center shadow-inner">
                        {b.physical_url}
                    </div>
                  </div>
               </div>
            </div>
         ))}
       </div>

       <button 
         onClick={() => setShowCliDialog(true)}
         className="w-full mt-6 py-3.5 bg-theme-accent/10 border border-theme-accent/50 rounded-lg text-sm font-bold text-white hover:bg-theme-accent/20 hover:border-theme-accent transition-all flex justify-center items-center gap-2 group shadow-[0_0_15px_rgba(255,255,255,0.05)]"
       >
         <span className="group-hover:scale-110 transition-transform theme-accent">+</span> 
         Register Binding via CLI
       </button>

       {/* CLI Command Modal Overlay */}
       {showCliDialog && (
          <div className="absolute inset-0 bg-black/80 backdrop-blur-sm z-50 rounded-xl flex items-center justify-center p-6 animate-in fade-in duration-200">
             <div className="bg-dark-surface border border-theme-accent w-full rounded-2xl shadow-2xl overflow-hidden relative">
                <button 
                  onClick={() => setShowCliDialog(false)}
                  className="absolute top-4 right-4 text-gray-500 hover:text-white transition-colors"
                >
                  <X size={20} />
                </button>
                
                <div className="p-6 border-b border-dark-border bg-theme-accent/5">
                   <h3 className="text-lg font-bold flex items-center gap-3">
                      <Terminal className="theme-accent" size={20} />
                      CLI Binding Command
                   </h3>
                   <p className="text-xs text-gray-400 mt-1">Run this command from inside your local Vault project directory.</p>
                </div>
                
                <div className="p-6 space-y-4">
                   <div className="bg-black border border-dark-border p-4 rounded-lg flex items-center justify-between group/code cursor-copy shadow-inner relative" onClick={() => navigator.clipboard.writeText('ax bind <alias> <actual-physical-url>')}>
                       <code className="text-sm font-mono text-green-400 font-bold block overflow-hidden">
                           $&gt; ax bind &lt;alias&gt; &lt;actual-physical-url&gt;
                       </code>
                       <span className="text-[10px] text-gray-500 group-hover/code:text-white transition-colors absolute right-4 bg-black pl-2">Click to copy</span>
                   </div>
                   
                   <div className="mt-4 pt-4 border-t border-dark-border text-xs text-gray-500 space-y-2">
                      <p><span className="text-gray-300 font-semibold">&lt;alias&gt;</span>: The internal name your wasm code uses to call this service.</p>
                      <p><span className="text-gray-300 font-semibold">&lt;physical-url&gt;</span>: The real external URL mapped to this alias.</p>
                   </div>
                </div>
             </div>
          </div>
       )}
    </div>
  );
};
