-- Create invites table
CREATE TABLE IF NOT EXISTS invites (
    uuid TEXT PRIMARY KEY NOT NULL,
    group_id INTEGER NOT NULL,
    name TEXT,
    is_reusable INTEGER NOT NULL DEFAULT 0,
    duration_days INTEGER NOT NULL DEFAULT 1 CHECK(duration_days >= 1 AND duration_days <= 30),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

-- Index for faster lookups
CREATE INDEX idx_invites_group_id ON invites(group_id);
CREATE INDEX idx_invites_created_at ON invites(created_at);
