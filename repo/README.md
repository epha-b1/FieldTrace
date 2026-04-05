# FieldTrace Rescue & Supply Chain

Offline-first shelter and warehouse management system.

## Stack

- **Backend**: Axum (Rust)
- **Frontend**: Leptos (Rust/WASM, CSR)
- **Database**: SQLite (embedded, single-file)

---

## Test Credentials (quick reference)

The first `/auth/register` call **bootstraps** the initial administrator.
After that, registration is closed and new users must be created by an admin
via `POST /users`.

| Role             | Username    | Password             | Notes                                                                       |
| ---------------- | ----------- | -------------------- | --------------------------------------------------------------------------- |
| Administrator    | `admin`     | `SecurePass12`       | Full access. Create this first with `POST /auth/register`.                  |
| Operations Staff | `staff1`    | `StaffPass1234`      | Create/edit intake, evidence, check-in. No publish/retract, no admin routes. |
| Auditor          | `auditor1`  | `AuditorPass12`      | Read-only everywhere, **plus** publish/retract traceability and report/audit CSV export. |

Bootstrap the admin and log in from the browser at `http://localhost:8080/`,
or with curl:

```bash
# 1. Create the first admin (only works once, before any other user exists)
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"SecurePass12"}'

# 2. Login and keep the session cookie
curl -c /tmp/ck -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"SecurePass12"}'

# 3. Admin creates the other sample users
curl -b /tmp/ck -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"username":"staff1","password":"StaffPass1234","role":"operations_staff"}'

curl -b /tmp/ck -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"username":"auditor1","password":"AuditorPass12","role":"auditor"}'
```

Valid roles: `administrator`, `operations_staff`, `auditor`.

> Passwords are min 12 chars, Argon2id hashed, and accounts lock after 10
> failed attempts in a 15-minute rolling window (`429 ACCOUNT_LOCKED`).
> Sessions expire after 30 minutes of inactivity (HttpOnly cookie).

---

## Quick Start

```bash
docker compose up --build
```

The application will be available at **http://localhost:8080**.

- API: `http://localhost:8080/health`
- UI: `http://localhost:8080/`

## Running Tests

```bash
chmod +x run_tests.sh
./run_tests.sh
```

`run_tests.sh` is idempotent about stack state:

- If the `w2t52` Docker Compose stack is **already up and healthy**, the
  script reuses the running containers (no rebuild, no restart).
- If the stack is **not running**, it transparently runs
  `docker compose -p w2t52 up -d --build`, waits for `/health`, and then
  runs all test suites.

Either way, the database is reset between suites so each suite starts clean.

Test orchestration (8 steps):

```
[Step 1] Check stack status            (reuse or start)
[Step 2] Wait for /health
[Step 3] Slice 1 tests                 bootstrap + health
[Step 4] Slice 2 tests                 auth + users
[Step 5] Slice 3 tests                 address book
[Step 6] Slice 4 tests                 intake + inspections
[Step 7] Slices 4-11 comprehensive
[Step 8] Remediation suite             audit-report fixes (auditor matrix, idempotency, key rotation, diagnostics, …)
```

Each Docker build also runs `cargo test --release -p fieldtrace-backend`
inside the builder stage — any Rust unit test failure (crypto round-trip,
civil-date formatter, traceability checksum, supply parser, error-envelope
flatten, store-ZIP writer) fails the image build before the runtime image
ships.

## Environment Variables

All environment variables are defined inline in `docker-compose.yml`. No `.env` file required.

| Variable | Default | Description |
|---|---|---|
| PORT | 8080 | Server listen port |
| DATABASE_URL | sqlite:///app/storage/app.db | SQLite database path |
| STATIC_DIR | /app/static | Frontend static files directory |
| STORAGE_DIR | /app/storage | Writable directory for diagnostic ZIPs, uploads |
| RUST_LOG | info | Log level filter |
| ENCRYPTION_KEY | (set in compose) | AES-256 encryption key (64 hex chars) |
| ENCRYPTION_KEY_FILE | (unset) | Optional path to a file holding the hex key. If set, takes precedence over `ENCRYPTION_KEY` and is written to on `/admin/security/rotate-key` |

## Role matrix (enforced server-side)

| Action | administrator | operations_staff | auditor |
|---|---|---|---|
| Create/update intake, inspections, evidence, supply, check-in, members | ✔ | ✔ | ✖ 403 |
| Delete own unlinked evidence | ✔ | only uploader | ✖ 403 |
| Publish / retract traceability | ✔ | ✖ 403 | ✔ |
| CSV reports export / audit log export | ✔ | ✖ 403 | ✔ |
| User management, admin config, diagnostics, key rotation | ✔ | ✖ 403 | ✖ 403 |
| Legal hold toggle on evidence | ✔ | ✖ 403 | ✖ 403 |
| Anti-passback override at `/checkin` | ✔ | ✖ 403 | ✖ 403 |

### Account deletion (cooling-off)

- `POST /account/delete` schedules deletion in 7 days. You can still log in.
- `POST /account/cancel-deletion` clears the pending request.
- The `account_deletion_purge` background job runs hourly and removes
  accounts whose `deletion_requested_at` is older than 7 days, transactionally
  dropping their sessions and address book entries and anonymizing audit log
  references.

### Idempotency

Every mutating route (`POST`/`PATCH`/`PUT`/`DELETE`) on the protected router
accepts an optional `Idempotency-Key` header. Scope is
`method + normalized_route + actor_id + key`; the window is 10 minutes.
Retries within the window return the original response body/status and a
`Idempotent-Replay: true` header.

### Encryption key rotation

`POST /admin/security/rotate-key` with `{"new_key_hex": "<64 hex chars>"}`
decrypts every encrypted-at-rest field, re-encrypts with the new key, and
commits in a single SQLite transaction. The in-memory cipher is replaced
only after commit. If `ENCRYPTION_KEY_FILE` is configured, the new key is
atomically written there as well.

### Diagnostic package

`POST /admin/diagnostics/export` builds a real ZIP (PKZIP "stored" method,
no external dependencies) under `/app/storage/diagnostics/{id}.zip`
containing recent structured logs, job metrics, config history, and an
audit summary with sensitive fields redacted. Files older than 1 hour are
removed by the `diagnostics_cleanup` background job. Download via
`GET /admin/diagnostics/download/{id}`.

## Slices Implemented

| Slice | Feature |
|---|---|
| 1 | Foundation — health, trace IDs, SQLite WAL, Docker, static frontend |
| 2 | Auth + Users — Argon2id, lockout, sessions, RBAC |
| 3 | Address Book — AES-256-GCM encryption, ZIP+4 validation, phone masking, object-level auth |
| 4 | Intake + Inspections — state machine, 409 on invalid transitions |
| 5 | Evidence + Chunked Upload — size limits, EXIF flagging, link immutability, legal hold |
| 6 | Supply Parsing — deterministic color/size normalization, `needs_review` state |
| 7 | Traceability — Luhn checksum codes, publish/retract (Admin/Auditor only), offline verify |
| 8 | Check-In — anti-passback 2-min per facility, admin-only override |
| 9 | Dashboard + Reports — metrics summary, CSV export (Admin/Auditor only) |
| 10 | Admin Config — versioning, rollback, diagnostic export, jobs metrics |
| 11 | Security + Audit — append-only audit log, CSV export with [REDACTED] masking |
| 12 | Final Polish — integrated UI, test orchestration |

## Test Summary

**91/91 tests passing** from cold start (`docker compose down -v` → `./run_tests.sh`).
