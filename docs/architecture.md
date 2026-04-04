# FieldTrace — Architecture

## Stack

| Layer            | Technology                 |
| ---------------- | -------------------------- |
| Backend          | Axum (Rust)                |
| Frontend         | Leptos (Rust/WASM)         |
| Database         | SQLite (single file, sqlx) |
| Password hashing | Argon2id                   |
| Field encryption | AES-256-GCM                |
| Logging          | tracing (structured JSON)  |
| Background jobs  | Tokio tasks                |

Runs on a single machine. No internet required. No external services.

## Runtime and Deployment

- Docker-only runtime: the app is considered valid only when it runs via `docker compose up`.
- No host-local dependency assumptions (no local DB/bootstrap tools required).
- `docker-compose.yml` must define explicit ports, inline environment variables, and healthchecks.
- API container healthcheck targets `GET /health`; startup depends on healthy DB.

---

## Module Breakdown

| Module         | What it does                                                                          |
| -------------- | ------------------------------------------------------------------------------------- |
| `auth`         | Bootstrap registration, login, logout, session management, password hashing, lockout |
| `users`        | User CRUD, profile, address book, account deletion                                    |
| `intake`       | Intake records, status transitions                                                    |
| `inspections`  | Inspection CRUD, outcomes                                                             |
| `evidence`     | Chunked upload, watermark, retention, legal hold, immutability                        |
| `supply`       | Parsing pipeline, color/size normalization, conflict detection                        |
| `traceability` | Code generation, checksum, publish/retract, versioned events                          |
| `checkin`      | Member records, check-in, anti-passback                                               |
| `dashboard`    | Metrics aggregation, CSV export                                                       |
| `admin`        | Config versioning, rollback, diagnostic ZIP, key rotation                             |
| `audit`        | Append-only audit log                                                                 |
| `jobs`         | Background tasks (session cleanup, evidence retention, account deletion, ZIP cleanup) |
| `common`       | Session extractor, role guard, trace ID middleware, encryption service, error types   |

---

## Database Tables

```
users               — credentials, role, deletion_requested_at
sessions            — session token, last_active
address_book        — per-user shipping destinations (encrypted)
facilities          — facility code used in watermarks and traceability codes
intake_records      — animal/supply/donation intakes
inspections         — linked to intake_records
evidence_records    — uploaded media metadata, watermark, retention
evidence_links      — links evidence to intake/inspection/traceability/checkin
upload_sessions     — tracks chunked upload progress
supply_entries      — parsed supply data, conflict state
traceability_codes  — generated codes, publish/retract status
traceability_events — versioned publish/retract history with mandatory comment
checkin_ledger      — member check-in events
members             — barcode ID + name
config_versions     — last 10 config snapshots
structured_logs     — job and request logs
job_metrics         — background job run history
audit_logs          — append-only admin action trail
```

---

## Request Flow

```
Browser (Leptos)
    │
    ▼
Axum HTTP Server :8080
    ├── Trace ID middleware     → attaches UUID to every request and log line
    ├── Session/Auth middleware → validates cookie, checks 30-min inactivity, 401 if expired
    ├── Idempotency middleware  → runs after auth on protected mutating endpoints
    ├── Role guard              → checks session.role against required role, 403 if wrong
    └── Handler → Service → SQLite
```

---

## Background Jobs

| Job                    | Runs every | What it does                                           |
| ---------------------- | ---------- | ------------------------------------------------------ |
| Session cleanup        | 5 min      | Delete sessions inactive > 30 min                      |
| Account deletion       | 1 day      | Hard-delete accounts past 7-day cooling-off            |
| Evidence retention     | 1 day      | Delete unlinked, non-legal-hold evidence past 365 days |
| Diagnostic ZIP cleanup | 1 hour     | Delete generated ZIPs older than 1 hour                |

---

## Security Design

- Passwords: Argon2id, minimum 12 characters
- Account lockout: rolling-window failure check (timestamped failures, not lifetime counter)
- Sessions: random UUID in HttpOnly cookie, 30-min inactivity expiry
- Object-level authorization: ownership enforced in DB query predicates (`WHERE id=? AND owner_id=?`)
- Sensitive fields: AES-256-GCM encrypted (phone, address, donor_ref)
- Keystore: 256-bit key in `data/keystore.bin`, outside the database
- Key rotation: re-encrypts all sensitive fields in a single SQLite transaction
- Masking: only last 4 digits shown on screen
- Idempotency scope: dedup key bound to `method + route + actor_id` within configured window
- Audit log: INSERT only — no DELETE or UPDATE endpoint exists
- Trace IDs: on every request, every log line, every X-Trace-Id response header

---

## Error Response Format

Every error returns the same shape:

```json
{
  "status": 400,
  "code": "VALIDATION_ERROR",
  "message": "human readable description",
  "trace_id": "uuid"
}
```

Codes used: `VALIDATION_ERROR` (400), `UNAUTHORIZED` (401), `FORBIDDEN` (403), `NOT_FOUND` (404), `CONFLICT` (409), `INTERNAL_ERROR` (500)
