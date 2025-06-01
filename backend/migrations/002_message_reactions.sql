-- Add search_vector column to messages table
ALTER TABLE messages ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    to_tsvector('english', coalesce(content, '')) ||
    to_tsvector('english', coalesce(media_type::text, ''))
) STORED;

-- Create index for search_vector
CREATE INDEX idx_messages_search ON messages USING GIN (search_vector);

-- Create trigger function to update search_vector
CREATE OR REPLACE FUNCTION messages_search_vector_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        to_tsvector('english', coalesce(NEW.content, '')) ||
        to_tsvector('english', coalesce(NEW.media_type::text, ''));
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Create trigger to update search_vector on insert or update
CREATE TRIGGER messages_search_vector_update
    BEFORE INSERT OR UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION messages_search_vector_update(); 