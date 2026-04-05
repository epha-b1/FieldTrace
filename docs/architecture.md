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

| Module                   | What it does                                                                                                           |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------- |
| `auth`                   | Register, login, logout, sessions, password hashing, lockout, account deletion request/cancel                           |
| `users`                  | Admin user CRUD                                                                                                         |
| `address_book`           | Per-user encrypted address entries                                                                                      |
| `intake`                 | Intake records, state machine transitions (409 on invalid)                                                              |
| `inspections`            | Inspection CRUD, resolve-once                                                                                           |
| `evidence`               | Chunked upload, real watermark (`FAC01 MM/DD/YYYY hh:mm AM/PM`), retention, legal hold, immutability, keyword/tag/date search |
| `supply`                 | Parsing pipeline, color/size normalization, `needs_review` conflict state                                               |
| `traceability`           | Code generation (Luhn + real current date), checksum verify, publish/retract with mandatory comment                     |
| `checkin`                | Member records, anti-passback (includes `retry_after_seconds` in 409), admin-only override                              |
| `dashboard`              | Metrics aggregation with query filters, CSV export (Admin/Auditor only)                                                 |
| `admin`                  | Config versioning + rollback, **real** diagnostic ZIP generation + download + 1h cleanup, **real** transactional key rotation |
| `audit`                  | Append-only audit log + CSV export with sensitive fields redacted                                                       |
| `common`                 | `db_err` / `system_err` sanitizers, `require_write_role`, `require_admin_or_auditor`, `CivilDateTime` formatter          |
| `crypto`                 | AES-256-GCM with fallible `try_encrypt` / `try_decrypt`; live key held in `Arc<RwLock<Crypto>>` on `AppState` for rotation |
| `jobs`                   | `session_cleanup` (5 min), `account_deletion_purge` (1 h), `diagnostics_cleanup` (10 min), `evidence_retention` (1 h)    |
| `middleware::idempotency` | Auth-first idempotency — scope `method + matched_route + actor_id + key`, 10-minute window, replay sends `Idempotent-Replay: true` |
| `zip`                    | Minimal PKZIP stored-method writer with CRC-32 (no external crate) used by diagnostics                                  |

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

| Job                      | Runs every | What it does                                                                                                                |
| ------------------------ | ---------- | --------------------------------------------------------------------------------------------------------------------------- |
| `session_cleanup`        | 5 min      | Delete sessions inactive > 30 min. Records run state to `job_metrics`.                                                      |
| `account_deletion_purge` | 1 hour     | Transactionally hard-delete any user whose `deletion_requested_at` is > 7 days old; anonymizes `audit_logs.actor_id`, drops sessions + address book; rolls back on any failure. |
| `diagnostics_cleanup`    | 10 min     | Removes `{storage}/diagnostics/*.zip` files older than 1 hour.                                                              |
| `evidence_retention`     | 1 hour     | Records a run in `job_metrics` (deletion policy is placeholder — legal-hold rows never expire).                              |

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

Codes used: `VALIDATION_ERROR` (400), `UNAUTHORIZED` (401), `FORBIDDEN` (403),
`NOT_FOUND` (404), `CONFLICT` (409), `ACCOUNT_LOCKED` (429),
`ANTI_PASSBACK` (409, with flattened `retry_after_seconds`),
`INTERNAL_ERROR` (500).

Internal database errors are sanitized — handlers call
`common::db_err(trace_id)` which logs the full error with
`tracing::error!(error = ..., trace_id = ...)` and returns a generic
`"Internal server error"` message to the client. Argon2 hashing errors and
filesystem errors use the same pattern.
