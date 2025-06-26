-- Migration: Add 'keys' column to push_subscriptions if it does not exist
ALTER TABLE push_subscriptions ADD COLUMN IF NOT EXISTS keys JSONB NOT NULL DEFAULT '{}'; 