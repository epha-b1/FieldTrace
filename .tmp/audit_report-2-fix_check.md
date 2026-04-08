# Audit Report 2 Fix Check (Static Recheck)

## 1. Verdict
- **Overall conclusion: Partial Pass**
- Prior High issues from `audit_report-2.md` are now addressed by static evidence.
- Boundary: runtime behavior is still not executed in this audit (manual/runtime confirmation still required).

## 2. Scope and Boundary
- Rechecked changed areas only: evidence upload integrity/duration, traceability steps visibility, privacy preferences, supply fields, cookie hardening, tests/docs.
- Files reviewed include:
  - `repo/src/backend/modules/evidence/handlers.rs`
  - `repo/src/backend/modules/traceability/handlers.rs`
  - `repo/src/backend/modules/profile/handlers.rs`
  - `repo/src/backend/modules/supply/handlers.rs`
  - `repo/src/backend/modules/auth/handlers.rs`
  - `repo/src/frontend/pages/profile.rs`
  - `repo/src/frontend/pages/supply.rs`
  - `repo/src/frontend/api/client.rs`
  - `repo/migrations/0013_duration_and_privacy.sql`
  - `repo/API_tests/audit_fixes_test.sh`
  - `repo/README.md`
  - `repo/docs/api-spec.md`
- Not executed: project runtime/tests/docker (static-only boundary).

## 3. Fix-by-Fix Recheck

### A) Fingerprint integrity validation (previous High)
- **Result: Addressed**
- Server now computes SHA-256 from assembled chunks and rejects mismatch with conflict.
- Evidence:
  - `repo/src/backend/modules/evidence/handlers.rs:441`
  - `repo/src/backend/modules/evidence/handlers.rs:466`
  - `repo/src/backend/modules/evidence/handlers.rs:469`
  - `repo/src/backend/modules/evidence/handlers.rs:472`
- Test evidence:
  - `repo/API_tests/audit_fixes_test.sh:81`
  - `repo/API_tests/audit_fixes_test.sh:95`

### B) Duration enforcement bypass risk (previous High)
- **Result: Addressed (static evidence)**
- Duration now derived from uploaded file bytes; unverifiable video/audio are rejected fail-safe.
- Evidence:
  - extraction entry: `repo/src/backend/modules/evidence/handlers.rs:144`
  - mp4 parser: `repo/src/backend/modules/evidence/handlers.rs:156`
  - wav parser: `repo/src/backend/modules/evidence/handlers.rs:235`
  - enforced at completion: `repo/src/backend/modules/evidence/handlers.rs:478`
  - fail-safe reject on unverifiable duration: `repo/src/backend/modules/evidence/handlers.rs:507`
- Test evidence:
  - verified <= limit and > limit: `repo/API_tests/audit_fixes_test.sh:199`, `repo/API_tests/audit_fixes_test.sh:205`, `repo/API_tests/audit_fixes_test.sh:211`, `repo/API_tests/audit_fixes_test.sh:217`
  - fail-safe unverifiable: `repo/API_tests/audit_fixes_test.sh:223`, `repo/API_tests/audit_fixes_test.sh:229`
  - client-lie scenario: `repo/API_tests/audit_fixes_test.sh:235`

### C) Traceability steps visibility gap (previous High)
- **Result: Addressed**
- `GET /traceability/:id/steps` now enforces auditor visibility to published only.
- Evidence:
  - policy check: `repo/src/backend/modules/traceability/handlers.rs:211`
  - role/status guard: `repo/src/backend/modules/traceability/handlers.rs:233`
- Test evidence:
  - auditor blocked on draft/retracted: `repo/API_tests/audit_fixes_test.sh:272`, `repo/API_tests/audit_fixes_test.sh:288`
  - auditor allowed on published: `repo/API_tests/audit_fixes_test.sh:280`

### D) Privacy preferences missing (previous High)
- **Result: Addressed**
- Added persisted user-scoped privacy preferences with API + frontend integration.
- Evidence:
  - migration/table: `repo/migrations/0013_duration_and_privacy.sql:8`
  - routes: `repo/src/backend/app.rs:123`
  - handlers: `repo/src/backend/modules/profile/handlers.rs:13`, `repo/src/backend/modules/profile/handlers.rs:48`
  - shared contracts: `repo/src/shared/lib.rs:458`
  - frontend page wiring: `repo/src/frontend/pages/profile.rs:19`, `repo/src/frontend/pages/profile.rs:115`
  - frontend client API: `repo/src/frontend/api/client.rs:205`
- Test evidence:
  - CRUD + isolation + auth requirement: `repo/API_tests/audit_fixes_test.sh:306`, `repo/API_tests/audit_fixes_test.sh:317`, `repo/API_tests/audit_fixes_test.sh:334`, `repo/API_tests/audit_fixes_test.sh:352`

### E) Supply required field surface gap (previous Medium)
- **Result: Addressed**
- Added stock status, media references, review summary to schema/API/UI.
- Evidence:
  - schema: `repo/migrations/0013_duration_and_privacy.sql:18`
  - contracts: `repo/src/shared/lib.rs:414`
  - backend list/create: `repo/src/backend/modules/supply/handlers.rs:20`, `repo/src/backend/modules/supply/handlers.rs:60`
  - frontend form/display: `repo/src/frontend/pages/supply.rs:39`, `repo/src/frontend/pages/supply.rs:95`
- Test evidence:
  - create/list/validation/default: `repo/API_tests/audit_fixes_test.sh:363`, `repo/API_tests/audit_fixes_test.sh:375`, `repo/API_tests/audit_fixes_test.sh:381`, `repo/API_tests/audit_fixes_test.sh:387`

### F) Cookie hardening (`Secure` config) (previous Medium)
- **Result: Addressed**
- Added config-driven secure cookie attribute.
- Evidence:
  - config field/env: `repo/src/backend/config.rs:15`, `repo/src/backend/config.rs:123`
  - session cookie builder: `repo/src/backend/modules/auth/handlers.rs:353`
  - usage in login/register: `repo/src/backend/modules/auth/handlers.rs:25`, `repo/src/backend/modules/auth/handlers.rs:91`
- Test evidence:
  - header checks: `repo/API_tests/audit_fixes_test.sh:317`

## 4. Remaining Notes
- **Cannot Confirm Statistically:** runtime correctness of all synthetic MP4/WAV duration extraction paths across real-world files; manual verification with representative media is still recommended.
- Docs were updated for the fixed behaviors:
  - `repo/README.md:226`
  - `repo/docs/api-spec.md:95`

## 5. Final Recheck Judgment
- Relative to the previously reported issues, remediation is materially complete in static code evidence.
- Status upgrade from prior report outcome is justified: **Partial Pass**.
