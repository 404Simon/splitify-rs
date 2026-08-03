-- Add an emoji icon to group_map_markers (used as the marker on the map)
ALTER TABLE group_map_markers ADD COLUMN emoji TEXT NOT NULL DEFAULT '📍';
