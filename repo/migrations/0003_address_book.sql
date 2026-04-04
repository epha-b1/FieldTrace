-- Slice 3: Address Book
CREATE TABLE IF NOT EXISTS address_book (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    street_enc TEXT NOT NULL,
    city_enc TEXT NOT NULL,
    state_enc TEXT NOT NULL,
    zip_plus4 TEXT NOT NULL,
    phone_enc TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_address_book_user ON address_book(user_id);
