-- Rename team_prefix to team_name safely
DO $$ 
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tomains' AND column_name = 'team_prefix') THEN
        ALTER TABLE tomains RENAME COLUMN team_prefix TO team_name;
    END IF;
END $$;
