-- Add bindings table for service discovery
CREATE TABLE IF NOT EXISTS bindings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tomain_id TEXT NOT NULL,
    alias TEXT NOT NULL,
    physical_url TEXT NOT NULL,
    environment TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tomain_id, alias, environment)
);
