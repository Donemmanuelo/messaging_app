-- Migration: Create status_views table for tracking seen statuses
CREATE TABLE status_views (
    id SERIAL PRIMARY KEY,
    status_id UUID REFERENCES statuses(id),
    user_id UUID REFERENCES users(id),
    viewed_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(status_id, user_id)
); 