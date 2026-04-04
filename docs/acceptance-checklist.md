# FieldTrace — Acceptance Checklist

Use this before submission to verify the project passes the grading rubric.

---

## 1. Can It Run?

- [ ] `docker compose up --build` starts without errors
- [ ] SQLite database and keystore created automatically on first run
- [ ] `GET /health` returns 200
- [ ] Leptos UI loads at configured mapped port from `docker-compose.yml`
- [ ] `./run_tests.sh` (host) produces a clear pass/fail result by executing tests inside container
- [ ] App starts from a clean environment without modifying any code

---

## 1.1 Docker Baseline

- [ ] `docker-compose.yml` uses inline environment variables (no `env_file`)
- [ ] API service defines healthcheck against `/health`
- [ ] DB service defines healthcheck
- [ ] API service uses `restart: unless-stopped`
- [ ] All required ports are explicitly declared

---

## 2. All Features Implemented?

Go through `docs/features.md` and confirm every checkbox is ticked.

---

## 3. Code Structure

- [ ] Each domain has its own module (auth, intake, evidence, supply, traceability, checkin, dashboard, admin, audit)
- [ ] No single file contains all the logic
- [ ] Common utilities (session, role guard, encryption, error types) are in a shared module
- [ ] Background jobs are separate from request handlers
- [ ] Migrations are separate SQL files, not inline schema creation

---

## 4. Engineering Standards

- [ ] All errors return `{status, code, message, trace_id}` — no raw panics or stack traces exposed
- [ ] Structured JSON logging via tracing crate — no println! in production code
- [ ] Every request has a trace ID in logs and X-Trace-Id response header
- [ ] All write endpoints validate input server-side
- [ ] Sensitive values (passwords, keys) never appear in logs or API responses
- [ ] Config loaded from environment variables, not hardcoded

---

## 5. Security — Check These First

- [ ] Session cookie is HttpOnly
- [ ] Session expires after 30 minutes of inactivity
- [ ] Wrong role returns 403, not 404 or 200
- [ ] Operations Staff cannot publish/retract traceability → 403
- [ ] Operations Staff cannot access audit log → 403
- [ ] Operations Staff cannot export CSV → 403
- [ ] Auditor cannot POST to any endpoint → 403
- [ ] User can only access their own address book entries → 403 for others
- [ ] Linked evidence cannot be deleted → 409
- [ ] Anti-passback override only by Administrator → 403 for others
- [ ] Audit log has no DELETE or UPDATE endpoint
- [ ] Audit log export masks sensitive fields as [REDACTED]
- [ ] Passwords not returned in any API response
- [ ] Key rotation re-encrypts all sensitive fields in a single transaction

---

## 6. Tests

- [ ] Unit tests exist for: password hashing, lockout rolling window, session expiry, checksum generation, color normalization, size conversion, conflict detection, anti-passback window, watermark format, encrypt/decrypt, idempotency scope
- [ ] Integration tests exist for: register bootstrap policy, login, wrong password, expired session, wrong role, all 403 cases listed above, evidence upload flow, linked evidence delete, supply conflict/resolve, traceability publish/retract, check-in anti-passback, account deletion flow, config rollback, audit log export masking, object-level auth isolation
- [ ] `./run_tests.sh` runs all tests inside Docker containers

---

## 7. UI (Leptos Frontend)

- [ ] Home workspace shows intake queue, pending inspections, exceptions
- [ ] In-app feedback shown: "saved locally", "needs review", "blocked by policy"
- [ ] Watermark overlay visible on video/audio evidence
- [ ] Phone numbers masked (last 4 digits only)
- [ ] Facial recognition toggle present but disabled with no biometric processing
- [ ] Session expiry redirects to login; draft form data restored after re-login
- [ ] Different sections visually distinct
- [ ] Consistent fonts, spacing, and layout

---

## 8. Business Logic Edge Cases

- [ ] Traceability code checksum: altering one character makes verify return invalid
- [ ] Anti-passback: 90 seconds after check-in → 409; 121 seconds after → 201
- [ ] Lockout window: attempts outside the rolling window do not trigger lockout
- [ ] Supply color "teal" with no mapping → needs_review, not silently dropped
- [ ] Evidence linked to traceability → delete returns 409, not 404
- [ ] Config save when already at 10 versions → oldest deleted, count stays at 10
- [ ] Diagnostic ZIP not downloaded within 1 hour → auto-deleted
- [ ] Account deletion during cooling-off → user can still log in and cancel
- [ ] Argon2 password with exactly 12 characters → accepted; 11 characters → 400
- [ ] Same idempotency key used by different users does not replay cross-user response
