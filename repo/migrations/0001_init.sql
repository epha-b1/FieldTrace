-- FieldTrace foundation schema (Slice 1)

-- Sessions table shell (fully wired in Slice 2)
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_active TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Facilities reference table
CREATE TABLE IF NOT EXISTS facilities (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default facility for watermarks and traceability codes
INSERT OR IGNORE INTO facilities (id, code, name) VALUES ('default', 'FAC01', 'Main Facility');
