CREATE TABLE IF NOT EXISTS authorized_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key TEXT NOT NULL,
    key_type VARCHAR(50) NOT NULL,
    comment VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, public_key)
);

CREATE INDEX idx_authorized_keys_user_id ON authorized_keys(user_id);
CREATE INDEX idx_authorized_keys_public_key ON authorized_keys(public_key);
