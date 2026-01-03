import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { EnvironmentProvider } from './context/EnvironmentContext';
import { Dashboard } from './Dashboard';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false, 
      staleTime: 5000, 
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <EnvironmentProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/*" element={<Dashboard />} />
          </Routes>
        </BrowserRouter>
      </EnvironmentProvider>
    </QueryClientProvider>
  );
}

export default App;
