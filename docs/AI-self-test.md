# FieldTrace AI Self-Test Checklist

Use this checklist to verify every prompt requirement maps to a concrete implementation artifact before marking the project complete.

## How to Use

- For each row, confirm the artifact exists and the verification check passes.
- Mark `[x]` only after both implementation and test evidence are present.
- If a requirement spans multiple modules, all listed artifacts must be implemented.
- Path names are logical targets; adapt `crates/...` to `src/...` if the Docker-aligned layout is used.

## Requirement-to-Artifact Mapping

| Status | Prompt requirement | Primary implementation artifact(s) | Verification artifact(s) |
| --- | --- | --- | --- |
| [ ] | Offline local auth (username/password) | `crates/backend/src/modules/auth/` | `tests/integration/auth_flow.rs` |
| [ ] | Bootstrap registration endpoint (`/auth/register`) with first-admin-only policy | auth bootstrap guard + registration handler | registration policy integration test |
| [ ] | Password min 12 + salted hash | `crates/backend/src/modules/auth/service.rs` | `tests/security/role_guard.rs` + auth validation tests |
| [ ] | Session expiry at 30 min inactivity | `crates/backend/src/common/session.rs` | `tests/integration/auth_flow.rs` |
| [ ] | Roles: admin, operations_staff, auditor | `crates/backend/src/common/role_guard.rs` | route authorization tests |
| [ ] | Profile + privacy preferences | `crates/backend/src/modules/users/` + `crates/frontend/src/pages/profile.rs` | API + UI interaction tests |
| [ ] | Account deletion request + 7-day cooling-off + cancel | `crates/backend/src/modules/users/` + `crates/backend/src/jobs/account_deletion.rs` | `tests/integration/auth_flow.rs` |
| [ ] | Address book per-user | `crates/backend/src/modules/address_book/` | ownership tests |
| [ ] | ZIP+4 validation | `crates/shared/src/validation/address.rs` | validation unit tests |
| [ ] | Phone masking (last 4 only) | frontend formatters + backend serializers | snapshot/UI tests |
| [ ] | Address/phone encrypted at rest | `crates/backend/src/common/crypto.rs` + address repo | `tests/security/encryption.rs` |
| [ ] | Home workspace: intake + transfer queue + pending inspections + exceptions | `crates/frontend/src/pages/workspace.rs` + workspace API | integration workspace tests |
| [ ] | In-app feedback states (saved locally / needs review / blocked by policy) | frontend notification/state components | UI flow tests |
| [ ] | Intake CRUD for animal/supply/donation with unique ID | `crates/backend/src/modules/intake/` | `tests/integration/intake_traceability_flow.rs` |
| [ ] | Intake status transitions with policy | intake service transition rules | transition unit tests |
| [ ] | Inspections linked to intake, status lifecycle | `crates/backend/src/modules/inspections/` | inspection integration tests |
| [ ] | Evidence capture limits (photo/video/audio) | `crates/backend/src/modules/evidence/validation.rs` | `tests/integration/evidence_upload_flow.rs` |
| [ ] | Format + fingerprint validation | evidence ingest service | evidence ingest tests |
| [ ] | Local compression | evidence media pipeline | media pipeline tests |
| [ ] | Resumable uploads in 2 MB chunks | `upload_sessions` handlers in evidence module | resume/retry tests |
| [ ] | Watermark on photos; metadata watermark for video/audio | evidence watermark service + UI overlay | visual/assertion tests |
| [ ] | Flag missing EXIF capture time | evidence metadata parser | metadata edge-case tests |
| [ ] | Evidence link targets (intake, inspection, traceability, check-in) | evidence link table + service | integration linking tests |
| [ ] | Evidence search by keyword/tag/date | evidence query endpoint + frontend search view | search tests |
| [ ] | Retention 365 days + legal hold override | evidence retention job + legal hold endpoint | retention job tests |
| [ ] | Evidence immutable once linked to traceability | evidence policy guard | immutability tests |
| [ ] | Supply guided parsing fields | `crates/backend/src/modules/supply/` | supply parser tests |
| [ ] | Color normalization deterministic map | supply normalization rules | deterministic mapping tests |
| [ ] | Size canonical conversion (oz/lb, in/ft) | unit conversion utility | conversion tests |
| [ ] | Missing values from local defaults only | supply defaults service | no-external-source tests |
| [ ] | Conflicts -> needs_review + per-field details | supply conflict detector + schema | parser conflict tests |
| [ ] | Traceability code generation + checksum verify offline | `crates/backend/src/modules/traceability/code.rs` | checksum tests |
| [ ] | Publish/retract restricted to Admin/Auditor + mandatory comment | traceability policy/service | role + validation tests |
| [ ] | Versioned publish/retract with audit trail | `traceability_events` persistence | `tests/integration/intake_traceability_flow.rs` |
| [ ] | Retraction hides public/auditor view immediately | traceability query filtering by status | query visibility tests |
| [ ] | Process steps + inspection outcomes aggregation | traceability step aggregator | timeline assembly tests |
| [ ] | Check-in via barcode/manual member ID | `crates/backend/src/modules/checkin/` + frontend check-in page | `tests/integration/checkin_policy_flow.rs` |
| [ ] | Anti-passback 2-minute block + retry time | check-in policy service | anti-passback tests |
| [ ] | Admin override with mandatory reason | check-in override endpoint | authorization/validation tests |
| [ ] | Facial recognition placeholder only (no biometrics) | frontend toggle stub + no backend biometric module | static check + API audit |
| [ ] | Dashboard filters + full-text search | dashboard query layer + UI filter state | `tests/integration/dashboard_metrics_flow.rs` |
| [ ] | Metrics: rescue volume, adoption conversion, task completion, donations, inventory on hand | dashboard metric calculators | metric accuracy tests |
| [ ] | CSV export admin/auditor only | dashboard export endpoint + role guard | export auth tests |
| [ ] | Structured logs + trace IDs on requests/jobs | tracing middleware + job logger | `tests/api_contract/error_shape.rs` + log format checks |
| [ ] | Job run reports + failure root-cause notes | jobs reporting module + admin UI | admin observability tests |
| [ ] | Config versioning keep last 10 + rollback | admin config module + `config_versions` table | rollback/version cap tests |
| [ ] | One-click diagnostic ZIP + 1-hour cleanup | diagnostics service + cleanup job | diagnostic export/TTL tests |
| [ ] | AES-256-GCM sensitive field encryption | `crates/backend/src/common/crypto.rs` | `tests/security/encryption.rs` |
| [ ] | Local key file outside DB + rotation | keystore service + rotation endpoint | key rotation tests |
| [ ] | On-screen masking all but last four digits | shared masking util + frontend presenters | UI masking tests |
| [ ] | Audit log append-only | `crates/backend/src/modules/audit/` | `tests/security/audit_append_only.rs` |
| [ ] | Fully offline (no external APIs/cloud) | dependency policy + network-free runtime config | static dependency check + integration smoke |

## Workflow-Critical Requirements

| Status | EaglePoint workflow requirement | Primary implementation artifact(s) | Verification artifact(s) |
| --- | --- | --- | --- |
| [ ] | Docker-only execution path | `Dockerfile`, `docker-compose.yml`, `run_tests.sh` | cold-start compose run + healthcheck |
| [ ] | API healthcheck wired in compose | `docker-compose.yml` service healthcheck stanza | compose health status output |
| [ ] | No `.env` dependency for runtime | inline `environment:` in `docker-compose.yml` | static compose inspection |
| [ ] | Account lockout in rolling time window | auth lockout service + failure-attempt timestamps | lockout boundary tests |
| [ ] | Object-level authorization in DB query | repository methods with ownership predicate | cross-user access denial API tests |
| [ ] | Idempotency scoped to method+route+actor | idempotency table key schema + middleware | duplicate key tests across users/routes |
| [ ] | Idempotency executes after auth on protected routes | middleware order in app bootstrap | replay-without-auth returns 401 test |
| [ ] | Background jobs are registered at startup | startup wiring in backend bootstrap | startup registration test/static inspection |
| [ ] | Every mandatory rule has a test that fails if removed | unit/API test suite mapped to requirement matrix | requirement-to-test traceability review |

## Release Gate

- [ ] All checklist rows marked `[x]`
- [ ] `docs/features.md` fully checked
- [ ] End-to-end smoke run passes for auth, intake, evidence, traceability, check-in, dashboard
- [ ] Security regression suite passes
- [ ] Migration from empty DB to latest succeeds in one command
