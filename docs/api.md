# FieldTrace — API Reference

Base URL: `http://localhost:8080`  
Auth: session cookie (HttpOnly) set on login. All endpoints require it unless marked public.

Security notes:
- Protected routes evaluate auth/session before idempotency replay logic.
- Object-level authorization is enforced server-side for user-owned resources.

---

## Health

| Method | Path    | Auth   | Description               |
| ------ | ------- | ------ | ------------------------- |
| GET    | /health | public | Returns `{"status":"ok"}` |

---

## Auth

| Method | Path                  | Auth    | Description                        |
| ------ | --------------------- | ------- | ---------------------------------- |
| POST   | /auth/register        | public  | Bootstrap first Administrator only |
| POST   | /auth/login           | public  | Login, sets session cookie         |
| POST   | /auth/logout          | session | Invalidates session                |
| GET    | /auth/me              | session | Current user info                  |
| PATCH  | /auth/change-password | session | Change own password (min 12 chars) |

Lockout policy (implementation target): 10 failed login attempts within 15 minutes lock the account for 15 minutes.
Registration policy: `POST /auth/register` is allowed only before first account bootstrap; later account creation is admin-only via `POST /users`.

---

## Users

| Method | Path                     | Auth    | Description                                  |
| ------ | ------------------------ | ------- | -------------------------------------------- |
| GET    | /users                   | admin   | List all users                               |
| POST   | /users                   | admin   | Create user                                  |
| PATCH  | /users/:id               | admin   | Update user role or status                   |
| DELETE | /users/:id               | admin   | Delete user                                  |
| POST   | /account/delete          | session | Request account deletion (7-day cooling-off) |
| POST   | /account/cancel-deletion | session | Cancel pending deletion                      |

---

## Address Book

| Method | Path              | Auth    | Description                                     |
| ------ | ----------------- | ------- | ----------------------------------------------- |
| GET    | /address-book     | session | List own entries                                |
| POST   | /address-book     | session | Create entry (ZIP+4 validated, phone encrypted) |
| PATCH  | /address-book/:id | session | Update own entry only                           |
| DELETE | /address-book/:id | session | Delete own entry only                           |

---

## Intake Records

| Method | Path               | Auth         | Description          |
| ------ | ------------------ | ------------ | -------------------- |
| GET    | /intake            | session      | List intake records  |
| POST   | /intake            | admin, staff | Create intake record |
| GET    | /intake/:id        | session      | Get single record    |
| PATCH  | /intake/:id        | admin, staff | Update record        |
| PATCH  | /intake/:id/status | admin, staff | Update status        |

---

## Inspections

| Method | Path                     | Auth         | Description          |
| ------ | ------------------------ | ------------ | -------------------- |
| GET    | /inspections             | session      | List inspections     |
| POST   | /inspections             | admin, staff | Create inspection    |
| PATCH  | /inspections/:id/resolve | admin, staff | Resolve with outcome |

---

## Evidence

| Method | Path                     | Auth         | Description                                    |
| ------ | ------------------------ | ------------ | ---------------------------------------------- |
| POST   | /media/upload/start      | admin, staff | Start chunked upload session                   |
| POST   | /media/upload/chunk      | admin, staff | Upload one chunk                               |
| POST   | /media/upload/complete   | admin, staff | Finalize, watermark, store                     |
| GET    | /evidence                | session      | Search by keyword, tag, date                   |
| GET    | /evidence/:id            | session      | Get evidence record                            |
| DELETE | /evidence/:id            | admin, staff | Delete unlinked evidence only (409 if linked)  |
| POST   | /evidence/:id/link       | admin, staff | Link to intake/inspection/traceability/checkin |
| PATCH  | /evidence/:id/legal-hold | admin        | Set or clear legal hold                        |

---

## Supply Entries

| Method | Path                        | Auth         | Description                |
| ------ | --------------------------- | ------------ | -------------------------- |
| GET    | /supply-entries             | session      | List entries               |
| POST   | /supply-entries             | admin, staff | Create with guided parsing |
| PATCH  | /supply-entries/:id/resolve | admin, staff | Resolve parsing conflicts  |

---

## Traceability

| Method | Path                       | Auth           | Description                 |
| ------ | -------------------------- | -------------- | --------------------------- |
| GET    | /traceability              | session        | List codes                  |
| POST   | /traceability              | admin, auditor | Create code                 |
| POST   | /traceability/:id/publish  | admin, auditor | Publish (mandatory comment) |
| POST   | /traceability/:id/retract  | admin, auditor | Retract (mandatory comment) |
| GET    | /traceability/verify/:code | public         | Verify checksum offline     |

---

## Check-In

| Method | Path             | Auth         | Description       |
| ------ | ---------------- | ------------ | ----------------- |
| GET    | /members         | session      | List members      |
| POST   | /members         | admin, staff | Create member     |
| POST   | /checkin         | admin, staff | Check in a member |
| GET    | /checkin/history | session      | Check-in history  |

---

## Dashboard

| Method | Path                         | Auth           | Description                |
| ------ | ---------------------------- | -------------- | -------------------------- |
| GET    | /reports/summary             | session        | Metrics with filters       |
| GET    | /reports/export              | admin, auditor | CSV export (staff → 403)   |
| GET    | /reports/adoption-conversion | session        | Adoption conversion detail |

---

## Admin

| Method | Path                              | Auth  | Description                                                                              |
| ------ | --------------------------------- | ----- | ---------------------------------------------------------------------------------------- |
| GET    | /admin/config                     | admin | Get active config                                                                        |
| PATCH  | /admin/config                     | admin | Update config (saves new version, keeps last 10)                                         |
| GET    | /admin/config/versions            | admin | List last 10 versions                                                                    |
| POST   | /admin/config/rollback/:id        | admin | Restore a config version                                                                 |
| POST   | /admin/diagnostics/export         | admin | Generate real diagnostic ZIP (logs, metrics, config, audit summary). Returns download id |
| GET    | /admin/diagnostics/download/:id   | admin | Download the ZIP package. Expires after 1 hour (background cleanup).                     |
| GET    | /admin/jobs                       | admin | Job run history                                                                          |
| POST   | /admin/security/rotate-key        | admin | Rotate AES-256-GCM key. Body: `{"new_key_hex": "..."}`. Re-encrypts every row transactionally. |

---

## Audit Log

| Method | Path               | Auth           | Description                          |
| ------ | ------------------ | -------------- | ------------------------------------ |
| GET    | /audit-logs        | admin, auditor | Query audit log                      |
| GET    | /audit-logs/export | admin, auditor | CSV export (sensitive fields masked) |

---

## Key Error Codes

| Code             | HTTP | When                                                       |
| ---------------- | ---- | ---------------------------------------------------------- |
| VALIDATION_ERROR | 400  | Invalid input                                              |
| UNAUTHORIZED     | 401  | No session or session expired                              |
| FORBIDDEN        | 403  | Wrong role                                                 |
| NOT_FOUND        | 404  | Resource does not exist                                    |
| CONFLICT         | 409  | Anti-passback, linked evidence delete, duplicate           |
| ANTI_PASSBACK    | 409  | Re-entry within 2 minutes (includes `retry_after_seconds`) |
| INTERNAL_ERROR   | 500  | Unexpected server error                                    |

## Idempotency Header

All mutating routes (`POST`/`PATCH`/`PUT`/`DELETE`) on the protected router
accept an optional `Idempotency-Key` header. Deduplication scope:

`method + normalized_route + actor_id + idempotency_key`

Window: **10 minutes**. On replay, the response body and status code from
the original request are returned verbatim and an `Idempotent-Replay: true`
header is set. Records are cleaned up opportunistically on each write.

Cross-actor isolation: two different users sending the same key + route +
body produce two distinct side effects — keys are namespaced by `actor_id`.

## Role Matrix

| Role               | Can do                                                                                                                                                |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `administrator`    | All endpoints, including user management, key rotation, diagnostics, legal hold, account purge                                                        |
| `operations_staff` | Intake/inspections/evidence/supply/checkin/address-book create + update. Cannot publish or retract traceability. Cannot access admin routes.          |
| `auditor`          | **Read-only** across the board, plus CSV report/audit exports and `POST /traceability` / `publish` / `retract`. Cannot mutate any operational resource. |

Enforcement is in-handler (`common::require_write_role` and
`common::require_admin_or_auditor`) in addition to route-level
`require_auth` / `require_admin` middleware.

## Account Deletion Lifecycle

1. `POST /account/delete` marks `users.deletion_requested_at = now()`. The
   user retains full login access during the 7-day cooling-off window.
2. `POST /account/cancel-deletion` clears the marker.
3. The `account_deletion_purge` background job (every 1 hour) transactionally
   deletes any user whose `deletion_requested_at` is older than 7 days,
   anonymizing dependent audit log references and dropping sessions +
   address book entries for that user.
