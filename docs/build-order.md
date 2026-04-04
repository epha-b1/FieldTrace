# FieldTrace Build Order (Workflow-Compliant)

This plan follows `.tmp/eaglepoint-workflow.md` strictly:
- Docker-first runtime (`docker compose up --build` is source of truth)
- Build one slice at a time
- Do not move to next slice until current slice gate passes
- Every critical rule must have at least one test that fails if rule is removed

Framework baseline for all slices:
- Backend: Axum (Rust)
- Frontend: Leptos (Rust/WASM)
- Persistence: SQLite (`sqlx`)

## Global Rules (Apply to Every Slice)

- Run and verify inside Docker containers, not host-local tooling.
- Keep all environment values inline in `docker-compose.yml` (no `.env` dependency).
- Run tests via `./run_tests.sh` (host script that executes tests in container).
- Every backend endpoint added in a slice must be integrated into the UI in the same slice (no deferred UI wiring).
- Every slice gate must include both backend tests and frontend/UI integration checks.
- For auth/data security rules, prove behavior with 401/403/409 tests.
- For ownership/resource scope, enforce in DB query predicates, not frontend filtering.

## Fullstack Definition of Done (Per Slice)

- API endpoint exists, validated, and documented.
- UI route/page/component calls the endpoint and handles success/error/loading states.
- Role/policy failures are surfaced in UI with clear feedback text.
- Backend tests pass and frontend build/tests pass in Docker.
- Slice gate is not complete unless both layers pass.

## Preflight (Before Slice 1)

- Confirm docs exist and are aligned: `docs/questions.md`, `docs/design.md`, `docs/api-spec.md`, `docs/features.md`.
- Lock down project skeleton: `Dockerfile`, `docker-compose.yml`, `run_tests.sh`, migrations folder, test folders.
- Define error envelope and trace-id response header standard.
- Gate:
  - `docker compose up --build` starts cleanly
  - `/health` reachable from compose healthcheck
  - `run_tests.sh` executes and prints pass/fail summary

## Slice 1 - Foundation

- Implement Axum app bootstrap, router, shared state, config loading.
- Wire SQLite connection + migration runner.
- Add session middleware shell and trace-id middleware.
- Add `GET /health` returning `{"status":"ok"}`.
- Add structured JSON logging + `X-Trace-Id` on every response.
- Implement frontend app shell bootstrap and API client base.
- Wire frontend startup health check path (service status indicator or startup check).
- Tests:
  - Health endpoint contract test
  - Trace-id header presence test
  - Migration bootstrap test
  - Frontend build test (Leptos build succeeds in container)
  - Frontend health integration test (UI can call `/health`)
- Gate:
  - API and DB containers healthy
  - Backend + frontend foundation tests pass in container

## Slice 2 - Auth and Users

- Implement `POST /auth/register` for first-admin bootstrap only.
- Implement login/logout/me/change-password with session auth.
- Enforce password policy (min 12 chars) server-side.
- Implement account lockout in rolling window (timestamp-based, not lifetime counter).
- Implement user CRUD (admin-managed) and role guard checks.
- Implement login/register/logout/profile UI screens and role-aware navigation.
- Integrate all auth/users endpoints into frontend flows.
- Tests:
  - register bootstrap allowed once, blocked after init
  - wrong credentials -> 401
  - wrong role -> 403
  - lockout boundary window tests
  - UI auth flow tests (register/login/logout/session-expiry redirect)
- Gate:
  - Auth backend tests + UI auth tests pass
  - No protected endpoint bypass without session

## Slice 3 - Address Book

- Implement per-user address book CRUD.
- Enforce ZIP+4 format and phone masking behavior.
- Encrypt address/phone fields at rest.
- Enforce object-level authorization in DB queries (`id + owner_id`).
- Implement address book UI (list/create/edit/delete) with masking and validation feedback.
- Tests:
  - user A cannot read/update/delete user B address entries
  - invalid ZIP+4 -> 400
  - encrypted-at-rest roundtrip test
  - UI address book CRUD integration tests
- Gate:
  - Ownership/validation backend tests + UI CRUD tests pass

## Slice 4 - Intake and Inspections

- Implement intake CRUD for animal/supply/donation.
- Implement explicit intake state machine; reject invalid transitions with 409.
- Implement inspections linked to intake with pending/passed/failed lifecycle.
- Expose workspace counters for intake queue, pending inspections, exceptions.
- Implement intake + inspection UI pages and workspace widgets using live API data.
- Tests:
  - valid/invalid state transition tests (409 on invalid)
  - inspection link integrity tests
  - role enforcement tests
  - UI intake/inspection workflow tests
- Gate:
  - State machine tests pass
  - UI displays live backend state (no hardcoded domain responses)

## Slice 5 - Evidence Capture and Chunked Upload

- Implement media validation (type/size/duration/fingerprint).
- Implement resumable 2MB chunk upload sessions.
- Apply watermark rules: burned-in for photos, metadata/overlay for video/audio.
- Implement evidence linking targets and immutability policy when traceability-linked.
- Add idempotency where duplicate mutating calls are retryable.
- Implement evidence capture/upload/search/link UI flows (including progress and retry states).
- Tests:
  - upload size/format boundaries
  - chunk resume and completion correctness
  - linked evidence delete -> 409
  - idempotency replay behavior
  - UI upload and evidence-link integration tests
- Gate:
  - Evidence backend tests + UI evidence flow tests pass

## Slice 6 - Supply Parsing

- Implement guided parsing schema and local defaults.
- Implement deterministic color map and unit conversions (oz/lb, in/ft).
- Implement conflict detection (`needs_review`) with per-field reasons.
- Implement conflict resolution endpoint.
- Implement supply entry UI with standardized view and conflict resolution screen.
- Tests:
  - color normalization deterministic behavior
  - conversion boundary tests
  - unresolved conflicts flagged correctly
  - UI parsing/conflict resolution tests
- Gate:
  - Parsing backend tests + UI conflict-flow tests pass

## Slice 7 - Traceability

- Implement local code generation with checksum verification.
- Implement publish/retract with mandatory comment.
- Restrict publish/retract to Administrator + Auditor.
- Implement traceability process-step aggregation timeline.
- Preserve audit trail; retraction hides auditor/public-facing views immediately.
- Implement traceability UI (draft/publish/retract/verify) with role-aware controls.
- Tests:
  - checksum tamper test
  - role and comment requirement tests
  - visibility filter tests after retraction
  - UI traceability publish/retract integration tests
- Gate:
  - Traceability backend tests + UI traceability tests pass

## Slice 8 - Check-In

- Implement members registry and barcode/manual check-in.
- Enforce anti-passback 2-minute rule.
- Implement admin override with mandatory reason.
- Keep facial-recognition as UI placeholder only (no biometric processing path).
- Implement check-in UI for barcode/manual entry and policy feedback states.
- Tests:
  - 90s check-in -> 409, 121s -> success
  - override allowed only for admin
  - UI check-in and override-flow tests
- Gate:
  - Check-in backend tests + UI check-in tests pass

## Slice 9 - Dashboard and Reports

- Implement filters (region/time/status/tags) and full-text search.
- Implement metrics: rescue volume, adoption conversion, task completion, donations, inventory on hand.
- Implement CSV export for Administrator + Auditor only.
- Implement dashboard/report UI with filters, search, and CSV export controls.
- Tests:
  - metric formula accuracy tests
  - export access control tests
  - UI filter/search/export integration tests
- Gate:
  - Reporting backend tests + dashboard UI tests pass with correct auth boundaries

## Slice 10 - Admin Config and Observability

- Implement structured logs with trace IDs across requests + jobs.
- Implement job run reports with failure root-cause notes.
- Implement config snapshots (keep last 10) and rollback.
- Implement diagnostic ZIP export and 1-hour cleanup.
- Ensure all background jobs are registered in startup code (not just defined).
- Implement admin UI for config versions, rollback, diagnostics, and job reports.
- Tests:
  - config version cap and rollback tests
  - diagnostic zip lifecycle tests
  - startup wiring checks for every scheduled job
  - UI admin integration tests
- Gate:
  - Observability backend tests + admin UI tests pass

## Slice 11 - Security Hardening and Audit Log

- Complete AES-256-GCM coverage for sensitive fields.
- Implement local keystore and atomic key rotation flow.
- Enforce append-only audit log (no update/delete surface).
- Verify sensitive-field masking in API responses and logs.
- Verify idempotency scope is actor-bound (`method + route + actor_id + key`).
- Implement audit and security-visible UI behavior (masked display, forbidden-action handling).
- Tests:
  - encryption/decryption roundtrip + no plaintext-at-rest checks
  - audit append-only tests
  - cross-user idempotency isolation tests
  - UI authorization/masking regression tests
- Gate:
  - Security suite passes
  - 401/403/409 behavior verified across sensitive routes and UI responses

## Slice 12 - Final Polish

- Align docs with implemented behavior and endpoint list.
- Add/verify OpenAPI or API docs entrypoint if required by delivery rules.
- Finalize README startup/testing instructions for Docker flow.
- Perform cold-start validation with no existing containers/volumes.
- Run complete `./run_tests.sh` from cold start.
- Run full backend + frontend regression pass and UX feedback-state audit.
- Gate:
  - All unit/API tests pass in Docker
  - Frontend integration/regression suite passes in Docker
  - Acceptance checklist fully checked
  - No unresolved requirement gaps in docs
