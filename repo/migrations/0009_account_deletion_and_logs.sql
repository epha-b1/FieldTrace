-- Slice 11/12 remediation: account deletion cooling-off + structured log writes

-- Cooling-off timestamp on users (7-day purge window)
ALTER TABLE users ADD COLUMN deletion_requested_at TEXT;

-- Structured log writes (previously table existed but nothing wrote to it)
-- We just need to guarantee the schema matches what backend writes; schema
-- already exists from 0008_admin_audit.sql. This file is kept additive-only.

-- Index to speed up daily purge scans.
CREATE INDEX IF NOT EXISTS idx_users_deletion_requested ON users(deletion_requested_at);
