CREATE TABLE active_sessions (
    id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID PRIMARY KEY,
    expires_at TIMESTAMP NOT NULL
);