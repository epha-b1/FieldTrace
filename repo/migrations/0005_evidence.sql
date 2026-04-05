-- Slice 5: Evidence and chunked upload sessions

CREATE TABLE IF NOT EXISTS evidence_records (
    id TEXT PRIMARY KEY NOT NULL,
    filename TEXT NOT NULL,
    media_type TEXT NOT NULL CHECK(media_type IN ('photo','video','audio')),
    size_bytes INTEGER NOT NULL,
    fingerprint TEXT NOT NULL,
    watermark_text TEXT NOT NULL DEFAULT '',
    exif_capture_time TEXT,
    missing_exif INTEGER NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '',
    keyword TEXT NOT NULL DEFAULT '',
    linked INTEGER NOT NULL DEFAULT 0,
    legal_hold INTEGER NOT NULL DEFAULT 0,
    uploaded_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS upload_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    filename TEXT NOT NULL,
    media_type TEXT NOT NULL,
    total_chunks INTEGER NOT NULL,
    received_chunks TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'in_progress',
    uploader_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS evidence_links (
    id TEXT PRIMARY KEY NOT NULL,
    evidence_id TEXT NOT NULL REFERENCES evidence_records(id),
    target_type TEXT NOT NULL CHECK(target_type IN ('intake','inspection','traceability','checkin')),
    target_id TEXT NOT NULL,
    retracted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Idempotency tracking, actor-bound
CREATE TABLE IF NOT EXISTS idempotency_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    method TEXT NOT NULL,
    route TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    key_value TEXT NOT NULL,
    response_body TEXT NOT NULL,
    status_code INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(method, route, actor_id, key_value)
);

CREATE INDEX IF NOT EXISTS idx_evidence_links_ev ON evidence_links(evidence_id);
CREATE INDEX IF NOT EXISTS idx_idempotency_lookup ON idempotency_keys(method, route, actor_id, key_value);
