-- Make media_url nullable in messages table
ALTER TABLE messages ALTER COLUMN media_url DROP NOT NULL; 