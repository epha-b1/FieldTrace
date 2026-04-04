# FieldTrace — Feature Checklist

Use this to verify all prompt requirements are implemented before submission.

---

## Authentication and Sessions

- [ ] Bootstrap registration endpoint for first Administrator only
- [ ] Local username/password login (no OAuth, no external services)
- [ ] Passwords minimum 12 characters, stored as salted hash
- [ ] Account lockout enforced with rolling failure window
- [ ] Sessions expire after 30 minutes of inactivity
- [ ] Three roles: administrator, operations_staff, auditor
- [ ] Role-based access enforced server-side on every endpoint
- [ ] Object-level authorization enforced in DB queries for user-owned resources
- [ ] Post-bootstrap account provisioning is admin-only (`POST /users`)
- [ ] User profile management and privacy preferences
- [ ] Account deletion request with 7-day cooling-off period
- [ ] User can cancel deletion during cooling-off
- [ ] After 7 days account is permanently removed

## Address Book

- [ ] Per-user shipping destination address book
- [ ] US address format with ZIP+4 validation (NNNNN-NNNN)
- [ ] Phone number masking (last 4 digits shown only)
- [ ] Addresses and phone numbers encrypted at rest

## Home Workspace

- [ ] Today's intake and transfer queue visible on login
- [ ] Pending inspections count shown
- [ ] Exceptions highlighted
- [ ] Immediate in-app feedback: "saved locally", "needs review", "blocked by policy"

## Intake Records

- [ ] Create intake records for animals, supplies, and donations
- [ ] Each intake gets a unique ID
- [ ] Status transitions (received → in_care/in_stock → adopted/transferred/disposed)
- [ ] Donor reference field (encrypted, for donation type)
- [ ] Linkable to evidence, inspections, and traceability codes

## Inspections

- [ ] Create inspections linked to intake records
- [ ] Status: pending / passed / failed
- [ ] Outcome notes
- [ ] Operations Staff create and resolve; Auditors view only

## Evidence Capture

- [ ] Photo upload (max 25 MB)
- [ ] Video upload (max 60 seconds, max 150 MB)
- [ ] Audio upload (max 2 minutes, max 20 MB)
- [ ] Format and fingerprint validation on ingest
- [ ] Local compression
- [ ] Resumable uploads in 2 MB chunks
- [ ] Watermark burned into photos (facility code + MM/DD/YYYY hh:mm AM/PM)
- [ ] Watermark stored as metadata for video/audio, shown as overlay in UI
- [ ] Flag files missing EXIF capture time
- [ ] Link evidence to: intake record, inspection, traceability code, check-in event
- [ ] Search by keyword, tag, or capture date
- [ ] Default retention 365 days
- [ ] Manual legal-hold override
- [ ] Unlinked evidence deletable by uploader or administrator
- [ ] Evidence immutable once linked to a traceability record

## Supply Entries

- [ ] Guided parsing: item name, SKU, size, color, price, discount, stock status, media refs, staff notes
- [ ] Color normalization (e.g. "navy" → "blue") via deterministic enum map
- [ ] Size canonical conversion (oz/lb, in/ft)
- [ ] Missing values filled from local defaults only — never external sources
- [ ] Conflicts trigger "needs review" state with per-field details
- [ ] Operations Staff can resolve conflicts
- [ ] Standardized view with unit conversion display

## Traceability

- [ ] Locally generated codes with checksum (format: FAC01-20260401-0042-7)
- [ ] Offline checksum verification
- [ ] Status: draft / published / retracted
- [ ] Publish requires mandatory comment — Administrator and Auditor only
- [ ] Retract requires mandatory comment — Administrator and Auditor only
- [ ] Every publish/retract versioned and stored in audit trail
- [ ] Retraction hides record from Auditor view immediately
- [ ] Audit trail preserved after retraction
- [ ] Operations Staff cannot publish or retract

## Check-In Ledger

- [ ] Member records: barcode ID + name
- [ ] Check-in via barcode scan or manual member ID entry
- [ ] Anti-passback: block re-entry within 2 minutes, return retry time
- [ ] Administrator can override anti-passback with mandatory reason
- [ ] Facial recognition shown as placeholder toggle only — zero biometric processing

## Dashboard and Reports

- [ ] Filters: region, time range, status, tags
- [ ] Full-text search
- [ ] Metrics: rescue volume, adoption conversion, task completion, donations logged, inventory on hand
- [ ] CSV export — Administrator and Auditor only
- [ ] Adoption conversion = adopted animals / total animal intakes in period

## Observability

- [ ] Structured logs with trace IDs on every request and background job
- [ ] Job run reports with failure root-cause notes
- [ ] Config versioning: snapshot on every save, keep last 10
- [ ] Rollback to any of last 10 config versions
- [ ] One-click diagnostic ZIP: last 7 days logs + metrics snapshot + all config versions
- [ ] Diagnostic ZIP auto-deleted after 1 hour

## Security

- [ ] AES-256-GCM encryption for sensitive fields (phone, address, donor reference)
- [ ] Encryption key stored in local keystore file outside database
- [ ] Key rotation re-encrypts all sensitive fields atomically
- [ ] On-screen masking: last 4 digits only
- [ ] Audit log append-only — no delete or update
- [ ] Idempotency on retryable mutating operations (actor-bound dedup scope)
- [ ] Fully offline — no internet, no external APIs, no cloud services

## Docker Runtime

- [ ] App runs via `docker compose up --build`
- [ ] API and DB healthchecks configured in `docker-compose.yml`
- [ ] Environment values are inline in compose (no `.env` dependency)
- [ ] `run_tests.sh` runs tests inside Docker containers
