import React from 'react';
import { useEnvironment } from '../context/EnvironmentContext';
import { Shield, Database, Cloud, Zap } from 'lucide-react';

export const ContextSwitcher: React.FC = () => {
  const { environment, setEnvironment } = useEnvironment();

  return (
    <div className="flex space-x-3 bg-dark-bg/60 backdrop-blur-md p-1.5 rounded-xl border border-white/5 shadow-2xl">
      <button
        onClick={() => setEnvironment('DEV')}
        className={`px-5 py-2.5 rounded-lg flex items-center gap-2.5 transition-all duration-500 font-bold text-xs uppercase tracking-widest ${
          environment === 'DEV' 
          ? 'bg-black text-cyan-400 border border-cyan-400 shadow-[0_0_20px_rgba(6,182,212,0.4)] scale-105' 
          : 'hover:bg-white/5 text-gray-500 hover:text-gray-300'
        }`}
      >
        <Database size={14} className={environment === 'DEV' ? 'animate-pulse' : ''} /> DEV
      </button>
      <button
        onClick={() => setEnvironment('QA')}
        className={`px-5 py-2.5 rounded-lg flex items-center gap-2.5 transition-all duration-500 font-bold text-xs uppercase tracking-widest ${
          environment === 'QA' 
          ? 'bg-black text-yellow-500 border border-yellow-500 shadow-[0_0_20px_rgba(234,179,8,0.4)] scale-105' 
          : 'hover:bg-white/5 text-gray-500 hover:text-gray-300'
        }`}
      >
        <Zap size={14} className={environment === 'QA' ? 'animate-pulse' : ''} /> QA
      </button>
      <button
        onClick={() => setEnvironment('STAGING')}
        className={`px-5 py-2.5 rounded-lg flex items-center gap-2.5 transition-all duration-500 font-bold text-xs uppercase tracking-widest ${
          environment === 'STAGING' 
          ? 'bg-black text-purple-500 border border-purple-500 shadow-[0_0_20px_rgba(168,85,247,0.4)] scale-105' 
          : 'hover:bg-white/5 text-gray-500 hover:text-gray-300'
        }`}
      >
        <Cloud size={14} className={environment === 'STAGING' ? 'animate-pulse' : ''} /> STAGING
      </button>
      <button
        onClick={() => setEnvironment('PROD')}
        className={`px-5 py-2.5 rounded-lg flex items-center gap-2.5 transition-all duration-500 font-bold text-xs uppercase tracking-widest ${
          environment === 'PROD' 
          ? 'bg-black text-red-500 border border-red-500 shadow-[0_0_20px_rgba(239,68,68,0.4)] scale-105' 
          : 'hover:bg-white/5 text-gray-500 hover:text-gray-300'
        }`}
      >
        <Shield size={14} className={environment === 'PROD' ? 'animate-pulse' : ''} /> PROD
      </button>
    </div>
  );
};
