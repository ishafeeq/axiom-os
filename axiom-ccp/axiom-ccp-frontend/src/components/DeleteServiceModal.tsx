import React, { useState } from 'react';
import { X, AlertTriangle, Trash2 } from 'lucide-react';

interface DeleteServiceModalProps {
  isOpen: boolean;
  onClose: () => void;
  serviceName: string;
  onConfirm: () => Promise<void>;
}

export const DeleteServiceModal: React.FC<DeleteServiceModalProps> = ({ 
  isOpen, 
  onClose, 
  serviceName,
  onConfirm 
}) => {
  const [confirmName, setConfirmName] = useState('');
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleConfirm = async () => {
    if (confirmName !== serviceName) {
      setError('Service name does not match.');
      return;
    }

    setIsDeleting(true);
    setError(null);
    try {
      await onConfirm();
      onClose();
    } catch (err: any) {
      setError(err.message || 'Failed to delete service.');
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-md z-[60] flex items-center justify-center p-4">
      <div className="bg-dark-surface border border-red-900/30 rounded-xl w-full max-w-md shadow-2xl animate-fade-in overflow-hidden">
        
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-dark-border bg-red-900/10">
           <h2 className="text-xl font-bold flex items-center gap-2 text-red-500">
             <AlertTriangle size={22} /> Dangerous Action
           </h2>
           <button onClick={onClose} className="text-gray-500 hover:text-gray-300 transition">
             <X size={20} />
           </button>
        </div>

        <div className="p-6 space-y-4">
           <p className="text-gray-300 text-sm leading-relaxed">
             You are about to permanently delete the micro-service <span className="text-white font-bold font-mono">"{serviceName}"</span> from the high-availability registry. This action cannot be undone.
           </p>

           <div className="bg-dark-bg p-4 rounded-lg border border-dark-border">
              <label className="block text-xs font-bold text-gray-500 uppercase tracking-widest mb-2">
                Type service name to confirm
              </label>
              <input 
                type="text" 
                value={confirmName}
                onChange={(e) => {
                  setConfirmName(e.target.value);
                  setError(null);
                }}
                placeholder={serviceName}
                className="w-full bg-dark-surface border border-dark-border rounded-lg px-3 py-2 text-gray-200 focus:outline-none focus:border-red-500 transition-colors font-mono text-sm"
              />
           </div>

           {error && (
             <div className="text-red-500 text-xs mt-2 border border-red-900/30 bg-red-900/5 p-2 rounded">
               {error}
             </div>
           )}

           <div className="flex gap-3 pt-4">
              <button 
                onClick={onClose}
                className="flex-1 py-2.5 bg-dark-bg border border-dark-border hover:bg-dark-border rounded-lg text-sm font-bold transition text-gray-400"
              >
                Cancel
              </button>
              <button 
                onClick={handleConfirm}
                disabled={confirmName !== serviceName || isDeleting}
                className={`flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-bold transition-all ${
                  confirmName === serviceName && !isDeleting
                  ? 'bg-red-600 hover:bg-red-500 text-white shadow-lg shadow-red-900/20'
                  : 'bg-dark-bg text-gray-700 border border-dark-border cursor-not-allowed'
                }`}
              >
                {isDeleting ? 'DELETING...' : <><Trash2 size={16}/> Delete Forever</>}
              </button>
           </div>
        </div>
      </div>
    </div>
  );
};
