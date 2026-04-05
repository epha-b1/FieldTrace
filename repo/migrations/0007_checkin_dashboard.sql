-- Slice 8/9: Check-in, members, tasks, stock movements

CREATE TABLE IF NOT EXISTS members (
    id TEXT PRIMARY KEY NOT NULL,
    member_id TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS checkin_ledger (
    id TEXT PRIMARY KEY NOT NULL,
    member_id TEXT NOT NULL REFERENCES members(id),
    facility_id TEXT NOT NULL DEFAULT 'default',
    checked_in_at TEXT NOT NULL DEFAULT (datetime('now')),
    override_reason TEXT,
    override_by TEXT REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY NOT NULL,
    facility_id TEXT NOT NULL DEFAULT 'default',
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'general',
    status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open','in_progress','completed','canceled')),
    due_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_checkin_member ON checkin_ledger(member_id, facility_id, checked_in_at);
