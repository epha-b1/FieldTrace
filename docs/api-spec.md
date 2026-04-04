# FieldTrace API Spec

Base URL: `http://localhost:8080`
Auth model: HttpOnly session cookie after login.

## Auth and Identity

| Method | Path | Auth | Notes |
| --- | --- | --- | --- |
| POST | `/auth/register` | public | First admin bootstrap only; disabled after initialization |
| POST | `/auth/login` | public | Creates session cookie |
| POST | `/auth/logout` | session | Invalidates active session |
| GET | `/auth/me` | session | Returns current principal |
| PATCH | `/auth/change-password` | session | Enforces min length and policy checks |

## Core Domain Endpoints

| Area | Endpoints |
| --- | --- |
| Users | `GET/POST /users`, `PATCH/DELETE /users/:id`, `POST /account/delete`, `POST /account/cancel-deletion` |
| Address Book | `GET/POST /address-book`, `PATCH/DELETE /address-book/:id` |
| Intake | `GET/POST /intake`, `GET/PATCH /intake/:id`, `PATCH /intake/:id/status` |
| Inspections | `GET/POST /inspections`, `PATCH /inspections/:id/resolve` |
| Evidence | `POST /media/upload/start`, `POST /media/upload/chunk`, `POST /media/upload/complete`, `GET /evidence`, `GET /evidence/:id`, `DELETE /evidence/:id`, `POST /evidence/:id/link`, `PATCH /evidence/:id/legal-hold` |
| Supply | `GET/POST /supply-entries`, `PATCH /supply-entries/:id/resolve` |
| Traceability | `GET/POST /traceability`, `POST /traceability/:id/publish`, `POST /traceability/:id/retract`, `GET /traceability/verify/:code` |
| Check-In | `GET/POST /members`, `POST /checkin`, `GET /checkin/history` |
| Dashboard | `GET /reports/summary`, `GET /reports/export`, `GET /reports/adoption-conversion` |
| Admin | `GET/PATCH /admin/config`, `GET /admin/config/versions`, `POST /admin/config/rollback/:id`, `POST /admin/diagnostics/export`, `GET /admin/jobs`, `POST /admin/security/rotate-key` |
| Audit | `GET /audit-logs`, `GET /audit-logs/export` |

## Security and Behavior Rules

- Role-based access enforced on every protected route.
- Object-level authorization enforced in DB queries for user-scoped resources.
- Idempotency applies to retryable mutating operations using `Idempotency-Key`.
- Invalid state transitions return `409 CONFLICT`.
- Standard error shape: `{status, code, message, trace_id}`.

## Error Codes

- `VALIDATION_ERROR` (400)
- `UNAUTHORIZED` (401)
- `FORBIDDEN` (403)
- `NOT_FOUND` (404)
- `CONFLICT` (409)
- `INTERNAL_ERROR` (500)
