import React, { useState } from 'react';
import { Terminal, CheckCircle2 } from 'lucide-react';

interface LogEntry {
  id: number;
  timestamp: string;
  source: string;
  message: string;
  requiresApproval?: boolean;
}

const initialLogs: LogEntry[] = [
  { id: 1, timestamp: '10:45:12 AM', source: 'Fabric', message: 'Analyzing Intent: "Needs a cache storing strings -> HashMaps"' },
  { id: 2, timestamp: '10:45:15 AM', source: 'Fabric Engine', message: 'Suggesting Redis on local port 6379 based on RED Context.', requiresApproval: true }
];

export const FabricLog: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>(initialLogs);
  const [approvedCount, setApprovedCount] = useState(0);

  const handleApprove = (id: number) => {
    setLogs(prev => prev.map(log => log.id === id ? { ...log, requiresApproval: false } : log));
    setApprovedCount(prev => prev + 1);
    
    // Simulate Follow-up log
    setTimeout(() => {
      setLogs(prev => [
        ...prev, 
        { id: Date.now(), timestamp: new Date().toLocaleTimeString(), source: 'Shell Orchestrator', message: 'Binding logical axiom.cache to localhost:6379 established.' }
      ]);
    }, 1000);
  };

  return (
    <div className="bg-dark-bg p-6 rounded-xl border border-dark-border shadow-inner font-mono text-sm h-full flex flex-col">
      <div className="flex items-center gap-3 mb-4 pb-2 border-b border-dark-border">
         <Terminal className="text-gray-400" size={20} />
         <h2 className="text-gray-300 font-bold uppercase tracking-widest text-xs">Fabric Provisioning Log</h2>
      </div>

      <div className="flex-1 space-y-3 overflow-y-auto">
        {logs.map((log) => (
          <div key={log.id} className="flex flex-col sm:flex-row sm:items-start gap-2 animate-fade-in">
             <span className="text-gray-600 shrink-0 w-24">[{log.timestamp}]</span>
             <div className="flex-1">
               <span className="theme-accent mr-2">[{log.source}]</span>
               <span className="text-gray-300">{log.message}</span>
               
               {log.requiresApproval && (
                 <div className="mt-2">
                   <button 
                     onClick={() => handleApprove(log.id)}
                     className="px-3 py-1 bg-dark-surface hover:bg-green-900/30 text-green-500 border border-green-900/50 rounded flex items-center gap-1 transition-colors"
                   >
                     <CheckCircle2 size={14} /> Approve Action
                   </button>
                 </div>
               )}
             </div>
          </div>
        ))}
        {approvedCount > 0 && (
          <div className="text-gray-500 italic mt-6">✓ {approvedCount} Fabric actions approved via CCP.</div>
        )}
      </div>
    </div>
  );
};
