-- Drop existing media table
DROP TABLE IF EXISTS media CASCADE;

-- Create updated media table
CREATE TABLE media (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type_ VARCHAR(50) NOT NULL,
    url TEXT NOT NULL,
    public_id TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_media_user_id ON media(user_id);
CREATE INDEX idx_media_type ON media(type_); 