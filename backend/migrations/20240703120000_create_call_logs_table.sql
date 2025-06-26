-- Migration: Create call_logs table for call history
CREATE TABLE call_logs (
    id SERIAL PRIMARY KEY,
    caller_id UUID REFERENCES users(id),
    callee_id UUID REFERENCES users(id),
    call_type TEXT NOT NULL CHECK (call_type IN ('audio', 'video')),
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMP,
    status TEXT NOT NULL CHECK (status IN ('missed', 'answered', 'declined', 'outgoing')),
    group_id UUID REFERENCES groups(id)
); 