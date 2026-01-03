import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

const API_BASE = 'http://localhost:3000/api/v1';

export interface Tomain {
    id: string;
    name: string;
    owner: string;
    health_status: string;
    package_name?: string;
    team_name?: string;
    creator_name?: string;
    created_at: string;
    rate_limit?: any;
    has_public_key?: boolean;
    api_count?: number;
    apis?: {
        name: string;
        method: string;
        params: [string, string][];
        doc?: string;
    }[];
    min_perspective?: string;
    wasm_hashes?: Record<string, string>;
    features?: Record<string, {
        status: string;
        wasm_hash: string;
        branch?: string;
        environments?: string[];
    }>;
}

export interface ConnectionMetadata {
    environment: string;
    database_url: string;
    cache_url: string;
    message_queue: string;
    database: string;
    message_bus: string;
}

export interface CreateTomainRequest {
    name: string;
    owner: string;
}

// 1. Fetch all tomains
export const useTomains = () => {
    return useQuery<Tomain[], Error>({
        queryKey: ['tomains'],
        queryFn: async () => {
            const response = await fetch(`${API_BASE}/tomains`);
            if (!response.ok) {
                throw new Error('Network response was not ok retrieving tomains');
            }
            return response.json();
        }
    });
};

// 1.5 Fetch a single tomain
export const useTomain = (id: string) => {
    return useQuery<Tomain, Error>({
        queryKey: ['tomain', id],
        queryFn: async () => {
            const response = await fetch(`${API_BASE}/tomains/${id}`);
            if (!response.ok) {
                throw new Error(`Network response fallback: ${id}`);
            }
            return response.json();
        },
        enabled: !!id,
    });
};

// 2. Fetch connection metadata for a specific tomain and context color
export const useTomainDetail = (name: string, color: string) => {
    return useQuery<ConnectionMetadata, Error>({
        // Add color to the queryKey so it automatically refetches when the context changes
        queryKey: ['tomainDetail', name, color],
        queryFn: async () => {
            // Don't fetch if there's no name
            if (!name) return null;

            const response = await fetch(`${API_BASE}/tomains/resolve/${name}?color=${color}`);
            if (!response.ok) {
                throw new Error('Network response was not ok resolving tomain capabilities');
            }
            return response.json();
        },
        enabled: !!name,
    });
};

// 3. Create a new tomain and invalidate the cache
export const useCreateTomain = () => {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async (newTomain: CreateTomainRequest) => {
            const response = await fetch(`${API_BASE}/tomains`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Admin-Token': 'axiom-local-dev-token' // Mock admin token for now
                },
                body: JSON.stringify(newTomain),
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`Error creating tomain: ${errorText}`);
            }

            return response.json();
        },
        onSuccess: () => {
            // Invalidate and refetch the tomains list to instantly update the UI Sidebar
            queryClient.invalidateQueries({ queryKey: ['tomains'] });
        },
    });
};

export const useDeleteTomain = () => {
    const queryClient = useQueryClient();

    return useMutation({
        mutationFn: async (id: string) => {
            const response = await fetch(`${API_BASE}/tomains/${id}`, {
                method: 'DELETE',
            });

            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`Error deleting tomain: ${errorText}`);
            }

            return response.text();
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['tomains'] });
        },
    });
};
