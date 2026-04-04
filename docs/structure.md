# FieldTrace Repository Structure

Proposed full layout aligned to EaglePoint workflow constraints (Docker-first, tests runnable in container).

```text
repo/
├── Dockerfile
├── docker-compose.yml                     # all env vars inline, no env_file
├── run_tests.sh                           # host entrypoint, runs tests in container
├── README.md
├── Cargo.toml                             # Rust workspace manifest
├── Cargo.lock
├── rust-toolchain.toml
├── .gitignore
├── docs/
│   ├── design.md
│   ├── api-spec.md
│   ├── architecture.md
│   ├── api.md
│   ├── features.md
│   ├── questions.md
│   ├── acceptance-checklist.md
│   ├── build-order.md
│   ├── structure.md
│   └── AI-self-test.md
├── migrations/
│   ├── 0001_init.sql
│   ├── 0002_auth_users.sql
│   ├── 0003_intake_inspections.sql
│   ├── 0004_evidence_uploads.sql
│   ├── 0005_supply_traceability.sql
│   ├── 0006_checkin_dashboard.sql
│   └── 0007_admin_security.sql
├── src/
│   ├── backend/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   ├── config.rs
│   │   ├── error.rs
│   │   ├── common/
│   │   ├── db/
│   │   ├── modules/
│   │   │   ├── auth/
│   │   │   ├── users/
│   │   │   ├── address_book/
│   │   │   ├── intake/
│   │   │   ├── inspections/
│   │   │   ├── evidence/
│   │   │   ├── supply/
│   │   │   ├── traceability/
│   │   │   ├── checkin/
│   │   │   ├── dashboard/
│   │   │   ├── admin/
│   │   │   └── audit/
│   │   └── jobs/
│   ├── frontend/
│   │   ├── app.rs
│   │   ├── router.rs
│   │   ├── components/
│   │   ├── pages/
│   │   └── api/
│   └── shared/
│       ├── dto/
│       ├── enums/
│       └── validation/
├── unit_tests/
│   ├── auth.spec.rs
│   ├── state-machine.spec.rs
│   ├── encryption.spec.rs
│   └── idempotency.spec.rs
├── API_tests/
│   ├── auth.api.spec.rs
│   ├── intake.api.spec.rs
│   ├── traceability.api.spec.rs
│   └── security.api.spec.rs
└── storage/
    ├── app.db
    ├── uploads/
    ├── diagnostics/
    └── keystore.bin
```

## Notes

- Keep domain logic inside backend module services, not in route handlers.
- Keep API request/response DTOs in `src/shared` to avoid drift between frontend and backend.
- Keep SQL migrations strictly additive and timestamped once implementation begins.
- Keep tests runnable entirely through Docker (`./run_tests.sh`) from a cold start.
