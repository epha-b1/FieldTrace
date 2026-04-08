# FieldTrace Static Audit Report

## 1. Verdict
- **Overall conclusion: Fail**
- Rationale: the repository is substantial and mostly structured, but at least one core flow is statically broken (frontend evidence upload cannot satisfy backend fingerprint verification), and multiple High-severity requirement-fit issues remain.

## 2. Scope and Static Verification Boundary
- Reviewed: `README.md`, `docs/api-spec.md`, `docs/design.md`, backend entry/middleware/modules, frontend app/pages/client, migrations, Docker/build config, and test scripts under `API_tests/` and `unit_tests/`.
- Excluded from evidence by rule: everything under `./.tmp/`.
- Not executed: app runtime, Docker, tests, browser rendering, network interactions, background timers.
- Manual verification required for: real media processing quality, actual UI rendering/interaction behavior, and time-based runtime effects.

## 3. Repository / Requirement Mapping Summary
- Prompt core goals mapped: offline Axum+Leptos+SQLite operations; RBAC; intake/inspections/evidence/supply/traceability/check-in/dashboard/admin; security controls (password/session/roles/encryption); diagnostics/observability.
- Main mapped implementation areas: router + middleware (`src/backend/app.rs`), auth/session/RBAC (`src/backend/modules/auth/handlers.rs`, `src/backend/middleware/*`), business modules (`src/backend/modules/*/handlers.rs`), schema (`migrations/*.sql`), frontend flows (`src/frontend/pages/*.rs`), tests (`API_tests/*.sh`, `unit_tests/*.sh`).

## 4. Section-by-section Review

### 1. Hard Gates

#### 1.1 Documentation and static verifiability
- **Conclusion: Pass**
- Rationale: startup/test/config instructions and API/role documentation are present and mostly consistent with routes/modules.
- Evidence: `repo/README.md:57`, `repo/README.md:68`, `repo/docs/api-spec.md:7`, `repo/src/backend/app.rs:70`, `repo/docker-compose.yml:1`.

#### 1.2 Material deviation from Prompt
- **Conclusion: Fail**
- Rationale: core media ingestion UX is materially weakened/broken against Prompt intent.
- Evidence:
  - Frontend sends non-SHA fingerprint (`FNV` style) while backend requires SHA-256 compare, causing upload-finalize failure from UI path: `repo/src/frontend/pages/evidence_upload.rs:127`, `repo/src/backend/modules/evidence/handlers.rs:444`, `repo/src/backend/modules/evidence/handlers.rs:469`.
  - “Compression” is metadata math, not actual media compression pipeline: `repo/src/backend/modules/evidence/handlers.rs:51`, `repo/src/backend/modules/evidence/handlers.rs:64`, `repo/src/backend/modules/evidence/handlers.rs:531`.

### 2. Delivery Completeness

#### 2.1 Core requirement coverage
- **Conclusion: Partial Pass**
- Rationale: broad feature surface exists, but critical core behavior gaps remain (media upload from real UI path, adoption semantics).
- Evidence: `repo/src/backend/app.rs:77`, `repo/src/frontend/pages/dashboard.rs:69`, `repo/src/frontend/pages/evidence_upload.rs:83`, `repo/src/backend/modules/intake/handlers.rs:13`, `repo/src/backend/modules/dashboard/handlers.rs:267`.

#### 2.2 End-to-end 0→1 deliverable shape
- **Conclusion: Partial Pass**
- Rationale: complete multi-module project with docs/migrations/tests exists, but key business path defects prevent credible acceptance.
- Evidence: `repo/Cargo.toml:1`, `repo/src/backend/modules/mod.rs:1`, `repo/src/frontend/pages/mod.rs:1`, `repo/migrations/0001_init.sql:1`, `repo/README.md:415`.

### 3. Engineering and Architecture Quality

#### 3.1 Structure and module decomposition
- **Conclusion: Pass**
- Rationale: clear modular backend/frontend separation and route wiring by domain modules.
- Evidence: `repo/src/backend/modules/mod.rs:1`, `repo/src/backend/app.rs:12`, `repo/src/frontend/pages/mod.rs:1`.

#### 3.2 Maintainability and extensibility
- **Conclusion: Partial Pass**
- Rationale: modular design is maintainable overall, but several business-critical behaviors are encoded in brittle shortcuts (simulated compression, non-durable key rotation defaults).
- Evidence: `repo/src/backend/modules/evidence/handlers.rs:51`, `repo/src/backend/modules/admin/handlers.rs:442`, `repo/docker-compose.yml:12`.

### 4. Engineering Details and Professionalism

#### 4.1 Error handling/logging/validation/API quality
- **Conclusion: Partial Pass**
- Rationale: standardized error envelopes, trace IDs, structured logs, and many validations exist; however, some key validations/flows are incomplete or semantically wrong.
- Evidence: `repo/src/backend/error.rs:42`, `repo/src/backend/middleware/trace_id.rs:11`, `repo/src/backend/common.rs:43`, `repo/src/backend/modules/checkin/handlers.rs:97`.

#### 4.2 Product-like quality vs demo shape
- **Conclusion: Fail**
- Rationale: despite substantial codebase, core user-facing evidence upload workflow is statically incompatible end-to-end.
- Evidence: `repo/src/frontend/pages/evidence_upload.rs:127`, `repo/src/backend/modules/evidence/handlers.rs:469`.

### 5. Prompt Understanding and Requirement Fit

#### 5.1 Business understanding and semantic fit
- **Conclusion: Fail**
- Rationale: key semantics are inconsistent with rescue/adoption business intent (non-animal adoption path + adoption metric query logic).
- Evidence: `repo/src/backend/modules/intake/handlers.rs:13`, `repo/src/backend/modules/intake/handlers.rs:121`, `repo/src/backend/modules/dashboard/handlers.rs:157`, `repo/src/backend/modules/dashboard/handlers.rs:267`.

### 6. Aesthetics (frontend/full-stack)

#### 6.1 Visual and interaction quality
- **Conclusion: Cannot Confirm Statistically**
- Rationale: static CSS/component structure exists, but final rendering, responsive behavior, and interaction polish require runtime/browser verification.
- Evidence: `repo/src/frontend/index.html:8`, `repo/src/frontend/pages/dashboard.rs:43`, `repo/src/frontend/pages/workspace.rs:38`.
- Manual verification: browser-based responsive and interaction pass (desktop/mobile, hover/focus/disabled states).

## 5. Issues / Suggestions (Severity-Rated)

### Blocker / High first

1) **Severity: Blocker**  
**Title:** Frontend evidence upload cannot complete due fingerprint algorithm mismatch  
**Conclusion:** Confirmed  
**Evidence:** `repo/src/frontend/pages/evidence_upload.rs:127`, `repo/src/backend/modules/evidence/handlers.rs:444`, `repo/src/backend/modules/evidence/handlers.rs:469`  
**Impact:** Core evidence capture flow from actual UI path fails at finalize, blocking a prompt-critical feature.  
**Minimum actionable fix:** Compute SHA-256 fingerprint in frontend (or server ignore client fingerprint and return computed hash), and add an integration test that exercises frontend upload path end-to-end.

2) **Severity: High**  
**Title:** Key rotation is non-durable by default and can cause post-restart decryption failures  
**Conclusion:** Confirmed  
**Evidence:** `repo/docker-compose.yml:12`, `repo/src/backend/app.rs:58`, `repo/src/backend/modules/admin/handlers.rs:442`, `repo/src/backend/config.rs:48`  
**Impact:** Rotated data may become unreadable after restart when `ENCRYPTION_KEY_FILE` is not configured/persisted.  
**Minimum actionable fix:** Make `ENCRYPTION_KEY_FILE` mandatory for rotation in production paths, persist rotated key atomically, and fail rotation if durable key persistence is unavailable.

3) **Severity: High**  
**Title:** Adoption business logic is type-agnostic and can produce invalid statuses/metrics  
**Conclusion:** Confirmed  
**Evidence:** `repo/src/backend/modules/intake/handlers.rs:13`, `repo/src/backend/modules/intake/handlers.rs:121`, `repo/src/backend/modules/dashboard/handlers.rs:157`, `repo/src/backend/modules/dashboard/handlers.rs:267`  
**Impact:** Non-animal records can be transitioned to `adopted`; adoption KPI can be skewed/misleading for operations.  
**Minimum actionable fix:** Enforce intake-type-aware status transitions and constrain adoption numerator/denominator to `intake_type='animal'` consistently.

4) **Severity: High**  
**Title:** “Local compression” is implemented as metadata simulation, not actual media compression  
**Conclusion:** Confirmed  
**Evidence:** `repo/src/backend/modules/evidence/handlers.rs:51`, `repo/src/backend/modules/evidence/handlers.rs:64`, `repo/migrations/0005_evidence.sql:3`  
**Impact:** Prompt requirement for local media compression is materially weakened; storage/quality/cost behavior is not what docs imply.  
**Minimum actionable fix:** Implement real compression/transcoding step (or clearly scope to metadata-only by requirement update) and persist actual resulting media artifact references.

### Medium / Low

5) **Severity: Medium**  
**Title:** Anti-passback override does not enforce non-empty reason  
**Conclusion:** Confirmed  
**Evidence:** `repo/src/backend/modules/checkin/handlers.rs:97`  
**Impact:** Policy/audit quality degrades because override “with reason” can be bypassed with empty string.  
**Minimum actionable fix:** Validate `override_reason.trim().is_empty()` as `400`.

6) **Severity: Medium**  
**Title:** Frontend media capture flow under-implements Prompt capture semantics  
**Conclusion:** Confirmed (static)  
**Evidence:** `repo/src/frontend/pages/evidence_upload.rs:195`, `repo/src/frontend/pages/evidence_upload.rs:89`, `repo/src/frontend/pages/evidence_search.rs:63`  
**Impact:** No direct device-capture semantics/duration handling in UI; watermark is shown as text metadata rather than visibly stamped media preview in frontend flow.  
**Minimum actionable fix:** Add capture-oriented input constraints/UX and explicit watermark preview/stamping behavior aligned with backend policy.

## 6. Security Review Summary

- **Authentication entry points:** **Pass** — register/login/session management + lockout checks are implemented. Evidence: `repo/src/backend/app.rs:72`, `repo/src/backend/modules/auth/handlers.rs:94`, `repo/src/backend/middleware/session.rs:39`.
- **Route-level authorization:** **Pass** — protected/admin routers apply auth and role guards. Evidence: `repo/src/backend/app.rs:136`, `repo/src/backend/app.rs:152`.
- **Object-level authorization:** **Partial Pass** — strong for address book/evidence ownership, but not uniformly needed/defined across all resources. Evidence: `repo/src/backend/modules/address_book/handlers.rs:33`, `repo/src/backend/modules/evidence/handlers.rs:674`, `repo/src/backend/modules/evidence/handlers.rs:736`.
- **Function-level authorization:** **Pass** — mutating handlers consistently call role helper checks. Evidence: `repo/src/backend/common.rs:98`, `repo/src/backend/modules/supply/handlers.rs:42`, `repo/src/backend/modules/checkin/handlers.rs:67`.
- **Tenant / user data isolation:** **Partial Pass** — per-user isolation is clear for address book/privacy preferences; global resources remain broadly visible by design. Evidence: `repo/src/backend/modules/address_book/handlers.rs:33`, `repo/src/backend/modules/profile/handlers.rs:33`.
- **Admin / internal / debug protection:** **Pass** — admin endpoints gated; audit endpoints role-checked. Evidence: `repo/src/backend/app.rs:139`, `repo/src/backend/modules/audit/handlers.rs:17`.

## 7. Tests and Logging Review

- **Unit tests:** **Partial Pass** — Rust unit tests exist in backend/shared modules, plus shell “unit” scripts; coverage is uneven and some “unit” tests are API-level shell checks. Evidence: `repo/Dockerfile:23`, `repo/src/backend/common.rs:180`, `repo/src/shared/lib.rs:85`, `repo/unit_tests/auth_test.sh:1`.
- **API/integration tests:** **Pass (static presence), Partial (quality)** — extensive shell suites cover many endpoints/roles/boundaries. Evidence: `repo/run_tests.sh:156`, `repo/API_tests/full_stack_test.sh:1`, `repo/API_tests/acceptance_boundary_test.sh:1`.
- **Logging/observability:** **Pass** — trace IDs, structured log table, diagnostics export, job metrics exist. Evidence: `repo/src/backend/middleware/trace_id.rs:11`, `repo/src/backend/common.rs:55`, `repo/src/backend/modules/admin/handlers.rs:131`, `repo/src/backend/jobs.rs:29`.
- **Sensitive-data leakage risk in logs/responses:** **Partial Pass** — sanitization exists but marker-based filtering is heuristic and not comprehensive by construction. Evidence: `repo/src/backend/common.rs:33`, `repo/src/backend/common.rs:43`, `repo/src/backend/modules/auth/handlers.rs:109`.

## 8. Test Coverage Assessment (Static Audit)

### 8.1 Test Overview
- Unit tests exist: Rust module tests and shell scripts (`repo/Dockerfile:23`, `repo/src/backend/zip.rs:165`, `repo/src/shared/lib.rs:85`, `repo/unit_tests/bootstrap_test.sh:1`).
- API/integration tests exist: multiple bash suites (`repo/run_tests.sh:156`, `repo/API_tests/blockers_api_test.sh:1`, `repo/API_tests/audit_fixes_test.sh:1`).
- Frontend tests are limited to static bundle/string checks, not interactive/browser flow tests (`repo/API_tests/frontend_draft_test.sh:9`).
- Docs provide test command (`repo/README.md:68`).

### 8.2 Coverage Mapping Table

| Requirement / Risk Point | Mapped Test Case(s) | Key Assertion / Fixture / Mock | Coverage Assessment | Gap | Minimum Test Addition |
|---|---|---|---|---|---|
| Auth bootstrap/login/lockout/session | `API_tests/auth_api_test.sh`, `API_tests/acceptance_boundary_test.sh` | 201/401/429 checks + rolling-window SQL time-shift (`repo/API_tests/acceptance_boundary_test.sh:103`) | sufficient | None major | Add malformed cookie/session fixation edge tests |
| Route authorization (admin-only surfaces) | `API_tests/acceptance_boundary_test.sh` | exhaustive admin route 403 matrix (`repo/API_tests/acceptance_boundary_test.sh:143`) | sufficient | None major | Add negative tests for newly added admin endpoints per release |
| Object-level auth (address/evidence ownership) | `API_tests/address_book_api_test.sh`, `API_tests/acceptance_boundary_test.sh` | cross-user delete/link blocked (`repo/API_tests/address_book_api_test.sh:59`, `repo/API_tests/acceptance_boundary_test.sh:193`) | basically covered | Not all resources have explicit object-level tests | Add cross-user tests for profile/account-delete side effects |
| Traceability publish/retract comment + role matrix | `API_tests/full_stack_test.sh`, `API_tests/blockers_api_test.sh` | publish no comment 400 + auditor create 403 (`repo/API_tests/full_stack_test.sh:176`, `repo/API_tests/blockers_api_test.sh:506`) | sufficient | None major | Add version monotonicity checks in same suite |
| Evidence upload integrity/duration | `API_tests/audit_fixes_test.sh` | SHA mismatch 409 + duration fail-safe (`repo/API_tests/audit_fixes_test.sh:95`, `repo/API_tests/audit_fixes_test.sh:223`) | sufficient (API) | Frontend uploader path not exercised | Add browser/E2E test using actual frontend upload controls |
| Dashboard filters/export consistency | `API_tests/blockers_api_test.sh`, `API_tests/acceptance_boundary_test.sh` | region/tags/q and summary-export consistency (`repo/API_tests/blockers_api_test.sh:658`, `repo/API_tests/acceptance_boundary_test.sh:449`) | basically covered | Adoption semantics not asserted for animal-only | Add tests that seed non-animal adopted rows and assert exclusion |
| Key rotation durability across restart | No meaningful test found | Only API call behavior asserted (`repo/API_tests/remediation_api_test.sh` static presence) | missing | Post-rotation restart behavior untested | Add integration test: rotate key, restart process, decrypt existing data |
| Frontend core flows (media upload/task closure) | `API_tests/frontend_draft_test.sh` only | bundle string checks (`repo/API_tests/frontend_draft_test.sh:32`) | missing | No interactive frontend flow assertions | Add component/E2E tests for evidence upload, check-in entry, traceability actions |

### 8.3 Security Coverage Audit
- **Authentication:** basically covered by API suites and boundary time-shifts (`repo/API_tests/auth_api_test.sh:37`, `repo/API_tests/acceptance_boundary_test.sh:103`).
- **Route authorization:** covered with explicit role matrix (`repo/API_tests/acceptance_boundary_test.sh:143`).
- **Object-level authorization:** partially covered (address/evidence), potential undetected defects may remain in less-tested domains.
- **Tenant/data isolation:** partially covered (address + privacy isolation), but global-resource isolation assumptions are not deeply tested (`repo/API_tests/audit_fixes_test.sh:334`).
- **Admin/internal protection:** covered for major admin routes (`repo/API_tests/acceptance_boundary_test.sh:143`).

### 8.4 Final Coverage Judgment
- **Partial Pass**
- Covered: auth, RBAC route guards, many domain happy paths, multiple boundary scenarios.
- Uncovered high-risk area: real frontend core-flow closure (especially evidence upload path) and key-rotation durability after restart; severe defects can remain undetected while suites still pass.

## 9. Final Notes
- This audit is static-only; no runtime success is claimed.
- The largest acceptance blockers are requirement-fit defects in core media and data-protection workflows, not repository size or module count.
- Manual verification should focus first on: frontend evidence upload end-to-end, post-rotation restart behavior, and KPI correctness for adoption metrics.
