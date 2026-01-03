-- Add metadata columns to tomains table
ALTER TABLE tomains 
ADD COLUMN IF NOT EXISTS team_prefix TEXT,
ADD COLUMN IF NOT EXISTS package_name TEXT,
ADD COLUMN IF NOT EXISTS creator_name TEXT,
ADD COLUMN IF NOT EXISTS created_by TEXT,
ADD COLUMN IF NOT EXISTS team_members TEXT[];
