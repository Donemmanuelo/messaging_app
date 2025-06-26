-- Migration: Create statuses table for stories/status feature
CREATE TABLE statuses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    media_url TEXT,
    text TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    expires_at TIMESTAMP
); 