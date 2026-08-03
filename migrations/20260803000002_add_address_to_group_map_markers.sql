-- Add an address column to group_map_markers for geocoded/entered addresses
ALTER TABLE group_map_markers ADD COLUMN address TEXT;
