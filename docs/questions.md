# Questions and Clarifications — FieldTrace Rescue & Supply Chain

---

## 1. Intake Record — What Is It Exactly?

**Question:** The prompt mentions "intake record" as something evidence can be linked to. What does an intake record represent in this system?

**Assumption:** An intake record represents a single animal or supply batch entering the facility. For animals: species, intake date, condition, source. For supplies: item type, quantity, source. Each intake gets a unique ID that can be linked to evidence, inspections, and traceability codes.

**Solution:** `intake_records` table: `id`, `type` (animal/supply), `facility_id`, `intake_date`, `status`, `details` (JSON), `created_by`. Evidence and traceability records FK to `intake_id`.

---

## 2. Traceability Code — What Does It Encode?

**Question:** Traceability codes are generated locally with a checksum. What information is encoded in the code, and what format is it?

**Assumption:** The code encodes: facility code + intake ID + sequence number + checksum digit. Format: `{FACILITY}-{YYYYMMDD}-{SEQ4}-{CHECK}` e.g. `FAC01-20260401-0042-7`. The checksum is a Luhn-style digit computed from the preceding segments. Verification is purely local — no external lookup.

**Solution:** `generateTraceabilityCode(facilityCode, intakeId, seq)` produces the code. `verifyTraceabilityCode(code)` recomputes the checksum and compares. Codes are stored in `traceability_codes` table with `status` (active/retracted) and `version`.

---

## 3. Publish vs Retract — What Does "Public-Facing" Mean?

**Question:** Retraction "immediately hides public-facing views." Since this is an offline system with no internet, what is the public-facing view?

**Assumption:** "Public-facing" means the read-only view accessible to Auditors and non-staff users within the local network. Published traceability records are visible to Auditors. Retracted records are hidden from Auditor views but remain in the database with full audit trail.

**Solution:** `traceability_codes.status` enum: `draft | published | retracted`. Auditor queries filter `status = published`. Admin/Operations Staff can see all statuses. Every publish/retract stores a `traceability_events` record with the mandatory comment and actor.

---

## 4. Anti-Passback — What Scope Does It Apply To?

**Question:** Anti-passback prevents re-entry within 2 minutes. Is this per facility, per door/gate, or system-wide?

**Assumption:** Per facility. A member ID cannot check in to the same facility twice within 2 minutes. Different facilities are independent. The 2-minute window is measured from the last successful check-in timestamp for that member+facility combination.

**Solution:** `checkin_ledger` table: `member_id`, `facility_id`, `checked_in_at`. Before allowing check-in, query: `SELECT MAX(checked_in_at) WHERE member_id=? AND facility_id=? AND checked_in_at > now() - 2 minutes`. If found, return 409 with `retryAfterSeconds`.

---

## 5. Account Deletion Cooling-Off — What Happens During 7 Days?

**Question:** Account deletion has a 7-day cooling-off period. Can the user still log in and use the system during those 7 days? Can they cancel the deletion?

**Assumption:** During the 7-day period the account is marked `pending_deletion`. The user can still log in and cancel the deletion. After 7 days a scheduled job permanently deletes the account and all associated personal data. Evidence linked to traceability records is anonymized rather than deleted.

**Solution:** `users.deletion_requested_at` timestamp. Login checks: if `deletion_requested_at` is set, show a warning banner with a "Cancel deletion" button. `POST /account/cancel-deletion` clears the field. A daily cron job hard-deletes accounts where `deletion_requested_at < now() - 7 days`.

---

## 6. Configuration Rollback — What Is a "Configuration"?

**Question:** Operators can roll back configuration to any of the last 10 saved versions. What counts as a configuration?

**Assumption:** Configuration includes: risk keyword library, parsing rules (color normalization map, size conversion rules), system settings (session timeout, retention period, max file sizes), and role permission assignments. Each save creates a versioned snapshot. Rolling back restores all config fields to the selected snapshot.

**Solution:** `config_versions` table: `id`, `version_number`, `snapshot` (JSON), `saved_by`, `created_at`. Max 10 versions kept (oldest deleted on overflow). `POST /admin/config/rollback/:versionId` restores the snapshot and creates a new version entry recording the rollback.

---

## 7. Watermark — Is It Burned Into the File or Overlaid in UI?

**Question:** The UI stamps a visible watermark on captured media. Is this burned into the image/video file, or just displayed as an overlay in the UI?

**Assumption:** For photos, the watermark is burned into the image file server-side using image processing (text overlay with facility code and timestamp). For videos and audio, the watermark metadata is stored as a separate record rather than re-encoding the file. The UI always displays the watermark data alongside the media.

**Solution:** On photo upload, the server applies a text watermark using an image library. `evidence_records.watermark_text` stores the stamped text. For video/audio, `watermark_text` is stored but not burned in. The frontend renders the watermark as an overlay for video/audio.

---

## 8. Resumable Uploads — How Are Chunks Tracked?

**Question:** Media ingestion supports resumable uploads in 2 MB chunks. How does the server track which chunks have been received?

**Assumption:** The client sends a unique `upload_id` with each chunk request, along with `chunk_index` and `total_chunks`. The server stores received chunks and returns a list of received chunk indices. The client can resume by re-sending missing chunks. Once all chunks are received, the server assembles the file.

**Solution:** `upload_sessions` table: `upload_id`, `filename`, `total_chunks`, `received_chunks` (JSON array of indices), `status` (in_progress/complete/failed). `POST /media/upload/chunk` accepts `upload_id`, `chunk_index`, binary data. `POST /media/upload/complete` triggers assembly and fingerprint validation.

---

## 9. Structured Parsing Conflicts — What Triggers "Needs Review"?

**Question:** Conflicts in structured parsing trigger a "needs review" state. What exactly constitutes a conflict?

**Assumption:** A conflict occurs when: (1) a field value cannot be mapped to a canonical enum (e.g., color "teal" has no mapping), (2) a size value has ambiguous units (e.g., "12" with no unit specified), or (3) a required field is missing and no local default exists. Items in "needs review" are visible to Operations Staff for manual resolution.

**Solution:** `supply_entries.parse_status` enum: `ok | needs_review`. `parse_conflicts` JSON field lists each conflicting field with the raw value and reason. Operations Staff can resolve conflicts via `PATCH /supply-entries/:id/resolve`.

---

## 10. Diagnostic Package — What Exactly Is Included?

**Question:** The one-click diagnostic package exports logs, metrics snapshot, and config history as a ZIP. What time range of logs is included?

**Assumption:** The last 7 days of structured logs, the current metrics snapshot (job run counts, error rates, queue depths), and all 10 config version snapshots. The ZIP is generated on demand and available for download for 1 hour before being deleted.

**Solution:** `POST /admin/diagnostics/export` triggers ZIP generation: collects log entries from `structured_logs` where `created_at > now() - 7 days`, current metrics from `job_metrics`, all rows from `config_versions`. Returns a download URL. A cleanup job deletes the file after 1 hour.

---

## 11. Adoption Conversion Metric — What Does It Measure?

**Question:** The dashboard shows "adoption conversion" as a metric. What is the conversion from and to?

**Assumption:** Adoption conversion = number of animals that moved from intake status to "adopted" status, divided by total intakes in the period. Expressed as a percentage. Only applies to animal intake records, not supply batches.

**Solution:** `GET /reports/adoption-conversion?from=&to=&facilityId=` queries `intake_records` where `type=animal`, counts total and those with `status=adopted` in the period. Returns `{ total, adopted, conversionRate }`.

---

## 12. Evidence Immutability — Can Evidence Be Deleted?

**Question:** Evidence links are immutable once referenced by a traceability record. But what about evidence that has never been linked? Can it be deleted?

**Assumption:** Unlinked evidence can be deleted by the Operations Staff who uploaded it, or by an Administrator. Once linked to any traceability record, the evidence record becomes immutable — it cannot be deleted or modified, only the link can be retracted (which hides it from public view but preserves the record). After the 365-day retention period, unlinked evidence is auto-deleted unless a legal hold is active.

**Solution:** `evidence_records.linked` boolean. Delete endpoint checks `linked = false`. Retention job skips records where `legal_hold = true` or `linked = true`. Retraction sets `traceability_evidence_links.retracted = true` without touching the evidence record.
