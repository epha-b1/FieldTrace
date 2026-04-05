-- Slice 6/7: Supply entries and traceability

CREATE TABLE IF NOT EXISTS supply_entries (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    sku TEXT,
    raw_size TEXT,
    canonical_size TEXT,
    raw_color TEXT,
    canonical_color TEXT,
    price_cents INTEGER,
    discount_cents INTEGER DEFAULT 0,
    stock_status TEXT DEFAULT 'unknown',
    parse_status TEXT NOT NULL DEFAULT 'ok' CHECK(parse_status IN ('ok','needs_review')),
    parse_conflicts TEXT NOT NULL DEFAULT '{}',
    notes TEXT NOT NULL DEFAULT '',
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS traceability_codes (
    id TEXT PRIMARY KEY NOT NULL,
    code TEXT NOT NULL UNIQUE,
    intake_id TEXT REFERENCES intake_records(id),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','published','retracted')),
    version INTEGER NOT NULL DEFAULT 1,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS traceability_events (
    id TEXT PRIMARY KEY NOT NULL,
    code_id TEXT NOT NULL REFERENCES traceability_codes(id),
    event_type TEXT NOT NULL CHECK(event_type IN ('publish','retract')),
    comment TEXT NOT NULL,
    actor_id TEXT NOT NULL REFERENCES users(id),
    version INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS traceability_steps (
    id TEXT PRIMARY KEY NOT NULL,
    code_id TEXT NOT NULL REFERENCES traceability_codes(id),
    step_type TEXT NOT NULL,
    step_label TEXT NOT NULL,
    details TEXT NOT NULL DEFAULT '{}',
    occurred_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_supply_parse_status ON supply_entries(parse_status);
CREATE INDEX IF NOT EXISTS idx_trace_status ON traceability_codes(status);
