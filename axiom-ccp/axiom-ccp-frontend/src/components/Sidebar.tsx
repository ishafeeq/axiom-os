import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useEnvironment } from '../context/EnvironmentContext';
import { useTomains } from '../api/tomainClient';
import { CreateAppModal } from './CreateAppModal';
import { Archive, KeySquare, TerminalSquare, ShieldCheck, PlusCircle, LayoutGrid, ChevronRight, ChevronDown } from 'lucide-react';

export const Sidebar: React.FC = () => {
  const { environment } = useEnvironment();
  const { data: tomains, isLoading, isError } = useTomains();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isExpanded, setIsExpanded] = useState(true);
  const location = useLocation();

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

  const isActive = (path: string) => location.pathname === path;
  const isPackageActive = (id: string) => location.pathname === `/package/${id}`;

  return (
    <>
      <div className={`w-64 border-r border-dark-border theme-sidebar-bg min-h-screen flex flex-col p-4 transition-all duration-700`}>
        <div className="flex items-center gap-3 mb-10 px-2">
           <Link to="/" className="flex items-center gap-3">
              <div className="p-2 bg-dark-surface rounded-lg border border-dark-border shadow-lg">
                 <ShieldCheck size={24} className="theme-accent transition-colors duration-300" />
              </div>
              <div>
                 <h1 className="text-xl font-bold tracking-tight">Axiom Reach</h1>
                 <p className="text-xs text-gray-500 font-mono">Central Control Plane</p>
              </div>
           </Link>
        </div>

        <nav className="flex-1 space-y-4">
          
          <div className="space-y-1">
            <Link 
              to="/" 
              className={`flex items-center gap-3 px-3 py-2 rounded-md transition-all font-semibold border-l-2 group ${
                isActive('/') 
                ? 'text-white bg-dark-surface border-white shadow-[0_0_15px_rgba(255,255,255,0.05)]' 
                : 'text-gray-400 bg-transparent border-transparent hover:bg-dark-surface hover:text-gray-200'
              }`}
            >
              <LayoutGrid size={18} className={`${isActive('/') ? 'theme-accent' : 'text-gray-500'} group-hover:scale-110 transition-transform`} />
              Cluster Overview
            </Link>

            <div>
              <button 
                onClick={() => setIsExpanded(!isExpanded)}
                className={`w-full flex justify-between items-center px-3 py-2 rounded-md transition-all font-semibold border-l-2 group relative ${
                  location.pathname.startsWith('/package')
                  ? 'text-white bg-dark-surface border-white shadow-[0_0_15px_rgba(255,255,255,0.05)]'
                  : 'text-gray-400 bg-transparent border-transparent hover:bg-dark-surface hover:text-gray-200'
                }`}
              >
                <div className="flex items-center gap-3">
                  <Archive size={18} className={`${location.pathname.startsWith('/package') ? 'theme-accent' : 'text-gray-500'} group-hover:scale-110 transition-transform`} />
                  <span>Micro-Services</span>
                  <span className="absolute right-10 bg-dark-bg px-1.5 py-0.5 rounded text-[10px] theme-accent border border-dark-border">{tomains?.length || 0}</span>
                </div>
                <div className="flex items-center gap-2">
                    {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
                </div>
              </button>

              {isExpanded && (
                <div className="mt-2 space-y-1 ml-4 border-l border-dark-border pl-2">
                    <button 
                      onClick={() => setIsModalOpen(true)}
                      className="w-full flex items-center gap-2 px-3 py-2 text-sm rounded transition-all text-gray-400 hover:text-white hover:bg-white/5 border border-dashed border-gray-800 mb-2 group/add"
                    >
                      <PlusCircle size={14} className="theme-accent group-hover/add:scale-110 transition-transform" />
                      <span className="font-bold tracking-tight">Add New Service</span>
                    </button>
                    {isLoading && <div className="px-3 text-xs text-gray-500 italic">Syncing...</div>}
                    {!isLoading && filteredTomains?.map(tomain => {
                      const displayName = tomain.package_name || tomain.name.split('.').pop() || tomain.name;
                      const active = isPackageActive(tomain.id);
                      return (
                        <Link 
                          key={tomain.id} 
                          to={`/package/${tomain.id}`} 
                          className={`flex items-center gap-2 px-3 py-1.5 text-sm rounded transition-all ${
                            active 
                            ? 'text-white bg-dark-surface/50 font-bold' 
                            : 'text-gray-400 hover:text-gray-200 hover:bg-dark-surface'
                          }`}
                        >
                          <div className={`w-1.5 h-1.5 rounded-full ${active ? 'ring-2 ring-white/20' : ''} ${tomain.health_status === 'Healthy' || tomain.health_status === 'Active' ? 'bg-green-500' : 'bg-red-500'}`} />
                          {displayName}
                        </Link>
                      )
                    })}
                </div>
              )}
            </div>

            <Link 
              to="/secrets" 
              className={`flex items-center gap-3 px-3 py-2 rounded-md transition-all font-semibold border-l-2 group ${
                isActive('/secrets') 
                ? 'text-white bg-dark-surface border-white shadow-[0_0_15px_rgba(255,255,255,0.05)]' 
                : 'text-gray-400 bg-transparent border-transparent hover:bg-dark-surface hover:text-gray-200'
              }`}
            >
              <KeySquare size={18} className={`${isActive('/secrets') ? 'theme-accent' : 'text-gray-500'} group-hover:scale-110 transition-transform`} />
              Secrets Management
            </Link>
            
            <Link 
              to="/logs" 
              className={`flex items-center gap-3 px-3 py-2 rounded-md transition-all font-semibold border-l-2 group ${
                isActive('/logs') 
                ? 'text-white bg-dark-surface border-white shadow-[0_0_15px_rgba(255,255,255,0.05)]' 
                : 'text-gray-400 bg-transparent border-transparent hover:bg-dark-surface hover:text-gray-200'
              }`}
            >
              <TerminalSquare size={18} className={`${isActive('/logs') ? 'theme-accent' : 'text-gray-500'} group-hover:scale-110 transition-transform`} />
              Fabric Logs
            </Link>
          </div>
        </nav>

        <div className="mt-auto px-2 py-4">
          <div className="text-xs text-gray-600 font-mono flex flex-col gap-1">
             <span>Registry: <span className={isError ? "text-red-500" : "text-green-500"}>{isError ? 'Offline' : 'Online'}</span></span>
             <span>Mode: {environment} Context</span>
             <span>Version: 0.1.0-alpha</span>
          </div>
        </div>
      </div>

      <CreateAppModal isOpen={isModalOpen} onClose={() => setIsModalOpen(false)} />
    </>
  );
};
