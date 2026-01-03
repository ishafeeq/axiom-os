import React, { useState } from 'react';
import { useCreateTomain } from '../api/tomainClient';
import { X, Server, TerminalSquare } from 'lucide-react';

interface CreateAppModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const CreateAppModal: React.FC<CreateAppModalProps> = ({ isOpen, onClose }) => {
  const [name, setName] = useState('');
  const [owner, setOwner] = useState('');
  const [env] = useState<'RED' | 'BLUE'>('RED');
  const [successCommand, setSuccessCommand] = useState<string | null>(null);

  const createMutation = useCreateTomain();

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !owner) return;

    try {
      await createMutation.mutateAsync({ name, owner });
      setSuccessCommand(`axiom init --tomain ${name} --env ${env}`);
    } catch (err) {
      console.error(err);
    }
  };

  const handleClose = () => {
    setSuccessCommand(null);
    setName('');
    setOwner('');
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-dark-surface border border-dark-border rounded-xl w-full max-w-lg shadow-2xl animate-fade-in overflow-hidden">
        
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-dark-border bg-dark-bg/50">
           <h2 className="text-xl font-bold flex items-center gap-2 text-gray-100">
             <Server className="theme-accent" size={20} /> Provision New Micro-Service
           </h2>
           <button onClick={handleClose} className="text-gray-500 hover:text-gray-300 transition">
             <X size={20} />
           </button>
        </div>

        <div className="p-6">
          {successCommand ? (
            <div className="space-y-6">
               <div className="bg-green-900/20 border border-green-900/50 p-4 rounded-lg text-green-400 text-sm">
                 <span className="font-bold">Success!</span> The Registry has provisioned service <b>{name}</b>.
               </div>
               
               <div>
                  <label className="block text-xs font-bold text-gray-500 uppercase tracking-widest mb-2">Fabric Initialization</label>
                  <div className="bg-dark-bg border border-dark-border p-4 rounded-lg flex items-center gap-3 font-mono text-sm text-gray-300 relative group">
                     <TerminalSquare size={16} className="text-gray-500" />
                     {successCommand}
                  </div>
               </div>

               <button onClick={handleClose} className="w-full py-2.5 bg-dark-bg border border-dark-border hover:bg-dark-border rounded-lg text-sm font-bold transition">
                 Close & Return to Dashboard
               </button>
            </div>
          ) : (
            <form onSubmit={handleSubmit} className="space-y-5">
              
              <div>
                <label className="block text-sm font-semibold text-gray-400 mb-1.5">Micro-Service (M-S) Name</label>
                <input 
                  type="text" 
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. billing-service"
                  className="w-full bg-dark-bg border border-dark-border rounded-lg px-4 py-2.5 text-gray-200 focus:outline-none focus:border-gray-500 transition-colors"
                  required 
                />
              </div>

              <div>
                <label className="block text-sm font-semibold text-gray-400 mb-1.5">Short Description</label>
                <input 
                  type="text" 
                  value={owner}
                  onChange={(e) => setOwner(e.target.value)}
                  placeholder="e.g. Handles payment processing and webhooks"
                  className="w-full bg-dark-bg border border-dark-border rounded-lg px-4 py-2.5 text-gray-200 focus:outline-none focus:border-gray-500 transition-colors"
                  required 
                />
              </div>

              <div className="pt-4 border-t border-dark-border mt-6">
                 <button 
                  type="submit" 
                  disabled={createMutation.isPending}
                  className="w-full py-3 theme-button-active rounded-lg font-bold shadow-lg disabled:opacity-50 transition-all font-mono tracking-widest text-sm"
                 >
                   {createMutation.isPending ? 'PROVISIONING...' : 'INITIATE SERVICE'}
                 </button>
                 {createMutation.isError && (
                   <div className="mt-3 text-red-400 text-xs text-center border border-red-900/30 bg-red-900/10 p-2 rounded">
                     {createMutation.error.message}
                   </div>
                 )}
              </div>

            </form>
          )}
        </div>
      </div>
    </div>
  );
};
