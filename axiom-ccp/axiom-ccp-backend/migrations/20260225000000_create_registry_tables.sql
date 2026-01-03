-- Up Migration
CREATE TABLE IF NOT EXISTS tomains (
    id UUID PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    owner TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS resources (
    id UUID PRIMARY KEY,
    tomain_id UUID NOT NULL REFERENCES tomains(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL,
    env_color TEXT NOT NULL,
    connection_string TEXT NOT NULL
);

-- Index for quick resolution by tomain and color
CREATE INDEX IF NOT EXISTS idx_resources_tomain_color ON resources(tomain_id, env_color);
