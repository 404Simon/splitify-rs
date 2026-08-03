-- Create group_map_markers table
-- Stores map markers that group members place on the group's map.
CREATE TABLE group_map_markers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER NOT NULL,
    created_by INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    latitude REAL NOT NULL,
    longitude REAL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_group_map_markers_group_id ON group_map_markers(group_id);
CREATE INDEX idx_group_map_markers_created_by ON group_map_markers(created_by);
