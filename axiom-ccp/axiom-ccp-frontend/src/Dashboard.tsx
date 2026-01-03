import React from 'react';
import { Routes, Route, useLocation } from 'react-router-dom';
import { ShieldCheck, UserCircle } from 'lucide-react';
import { ContextSwitcher } from './components/ContextSwitcher';
import { Sidebar } from './components/Sidebar';
import { ServerGrid } from './components/ServerGrid';
import { BindingPanel } from './components/BindingPanel';
import { FabricLog } from './components/FabricLog';
import { PackageDetails } from './components/PackageDetails';
import { FeatureDetails } from './components/FeatureDetails';

export const Dashboard: React.FC = () => {
  const location = useLocation();
  const isExplorer = location.pathname.includes('/explorer');

  return (
    <div className="flex min-h-screen theme-body-bg text-gray-100 font-sans transition-colors duration-500">
      
      {/* Left Sidebar */}
      {!isExplorer && <Sidebar />}

      {/* Main Workspace */}
      <div className="flex-1 flex flex-col overflow-y-auto w-full">
        
        {/* Top Header */}
        {!isExplorer && (
          <header className="flex justify-between items-center px-8 py-6 theme-header-bg sticky top-0 z-10 backdrop-blur-md">
             <div>
                <h1 className="text-3xl font-bold tracking-tight text-white flex items-center gap-3">
                   <ShieldCheck size={28} className="theme-accent" />
                   CCP Dashboard
                </h1>
                <p className="text-gray-400 mt-1">Manage hybrid-cloud deployments and environment capabilities.</p>
             </div>
             
             {/* Global Theme Switcher & Perspective Filter */}
             <div className="flex items-center gap-6">
                <ContextSwitcher />
                <div className="w-px h-8 bg-dark-border mx-2"></div>
                <div className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors cursor-pointer p-2 rounded-full hover:bg-white/5">
                   <UserCircle size={28} className="theme-accent" />
                </div>
             </div>
          </header>
        )}

        <main className="p-8 flex-1 max-w-[1400px] mx-auto w-full">

        {/* Main Content Sections */}
        <div className="flex flex-col gap-10">
            <Routes>
              {/* Default Overview View */}
              <Route path="/" element={
                 <div className="flex flex-col gap-10">
                    <section>
                       <ServerGrid />
                    </section>

                    <section className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                       <div className="flex flex-col h-full">
                          <BindingPanel />
                       </div>
                       <div className="flex flex-col h-[400px]">
                          <FabricLog />
                       </div>
                    </section>
                 </div>
              } />

              {/* Package Details View */}
              <Route path="/package/:packageId" element={<PackageDetails />} />
              
              {/* Feature Details View */}
              <Route path="/package/:packageId/feature/:featureName" element={<FeatureDetails />} />
              
              {/* Dedicated Explorer View (Fullscreen-like) */}
              <Route path="/package/:packageId/explorer" element={<PackageDetails isExplorer={true} />} />

              {/* Secrets Management View */}
              <Route path="/secrets" element={
                <div className="flex flex-col gap-6">
                   <h2 className="text-2xl font-bold tracking-tight">Secrets Management</h2>
                   <div className="p-8 bg-dark-surface border border-dark-border rounded-xl text-gray-500 italic">
                      Secure Vault for Tomain environment variables and API keys is standing by.
                   </div>
                </div>
              } />

              {/* Fabric Logs View */}
              <Route path="/logs" element={<FabricLog />} />
            </Routes>
        </div>
        </main>

        <footer className="p-4 theme-footer-bg text-center text-xs text-gray-600 font-mono tracking-widest border-t border-white/5">
           AXIOM reach — Distributed Capabilities Fabric — v0.1.0-alpha
        </footer>
      </div>

    </div>
  );
};
