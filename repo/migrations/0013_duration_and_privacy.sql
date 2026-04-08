-- Slice 13: Duration tracking, privacy preferences

-- Add declared duration to upload sessions so the server can enforce
-- duration policy at completion time (fail-safe: reject if not declared).
ALTER TABLE upload_sessions ADD COLUMN duration_seconds INTEGER NOT NULL DEFAULT 0;

-- User-scoped privacy preferences (persisted, user-editable)
CREATE TABLE IF NOT EXISTS privacy_preferences (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id),
    show_email INTEGER NOT NULL DEFAULT 1,
    show_phone INTEGER NOT NULL DEFAULT 0,
    allow_audit_log_export INTEGER NOT NULL DEFAULT 1,
    allow_data_sharing INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Add missing first-class supply fields
ALTER TABLE supply_entries ADD COLUMN media_references TEXT NOT NULL DEFAULT '';
ALTER TABLE supply_entries ADD COLUMN review_summary TEXT NOT NULL DEFAULT '';
