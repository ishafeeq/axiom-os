import React, { createContext, useContext, useState, useEffect } from 'react';

type ColorContextType = 'DEV' | 'QA' | 'STAGING' | 'PROD';

interface ActionContextValue {
  environment: ColorContextType;
  setEnvironment: (env: ColorContextType) => void;
}

const EnvironmentContext = createContext<ActionContextValue | undefined>(undefined);

export const EnvironmentProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [environment, setEnvironment] = useState<ColorContextType>('DEV');

  // Inject themes into the DOM body dynamically
  useEffect(() => {
    document.body.className = `env-${environment.toLowerCase()} theme-body-bg text-gray-100 min-h-screen transition-all duration-700`;
  }, [environment]);

  return (
    <EnvironmentContext.Provider value={{ environment, setEnvironment }}>
      {children}
    </EnvironmentContext.Provider>
  );
};

export const useEnvironment = () => {
  const context = useContext(EnvironmentContext);
  if (!context) throw new Error('useEnvironment must be used within an EnvironmentProvider');
  return context;
};
