-- Create status updates table
CREATE TABLE status_updates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    content TEXT,
    media_url TEXT,
    media_type VARCHAR(20) CHECK (media_type IN ('image', 'video')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE,
    background_color VARCHAR(7),
    font_style VARCHAR(50)
);

-- Create status views table to track who viewed the status
CREATE TABLE status_views (
    status_id UUID REFERENCES status_updates(id) ON DELETE CASCADE,
    viewer_id UUID REFERENCES users(id) ON DELETE CASCADE,
    viewed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (status_id, viewer_id)
);

-- Create indexes for better performance
CREATE INDEX idx_status_updates_user_id ON status_updates(user_id);
CREATE INDEX idx_status_updates_created_at ON status_updates(created_at);
CREATE INDEX idx_status_updates_expires_at ON status_updates(expires_at);
CREATE INDEX idx_status_views_viewer_id ON status_views(viewer_id);
