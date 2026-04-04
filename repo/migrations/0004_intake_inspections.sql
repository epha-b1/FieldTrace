-- Slice 4: Intake Records and Inspections

CREATE TABLE IF NOT EXISTS intake_records (
    id TEXT PRIMARY KEY NOT NULL,
    facility_id TEXT NOT NULL DEFAULT 'default' REFERENCES facilities(id),
    intake_type TEXT NOT NULL CHECK(intake_type IN ('animal', 'supply', 'donation')),
    status TEXT NOT NULL DEFAULT 'received' CHECK(status IN ('received','in_care','in_stock','adopted','transferred','disposed')),
    details TEXT NOT NULL DEFAULT '{}',
    donor_ref_enc TEXT,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS inspections (
    id TEXT PRIMARY KEY NOT NULL,
    intake_id TEXT NOT NULL REFERENCES intake_records(id),
    inspector_id TEXT NOT NULL REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','passed','failed')),
    outcome_notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_intake_facility ON intake_records(facility_id);
CREATE INDEX IF NOT EXISTS idx_inspections_intake ON inspections(intake_id);
