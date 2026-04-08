#!/bin/bash
# Audit fixes verification suite
# Tests all 6 findings: fingerprint integrity, duration fail-safe,
# traceability steps visibility, privacy preferences, supply fields,
# cookie secure flag.
set -e

PASS=0; FAIL=0; BASE="http://localhost:8080"
ADMIN_CK="/tmp/af_admin"; STAFF_CK="/tmp/af_staff"; AUDITOR_CK="/tmp/af_auditor"
DB="/app/storage/app.db"

check() {
    local name="$1" expected="$2" actual="$3"
    if [ "$actual" = "$expected" ]; then
        echo "PASS: $name"; PASS=$((PASS+1))
    else
        echo "FAIL: $name (expected $expected, got $actual)"; FAIL=$((FAIL+1))
    fi
}

contains() {
    local name="$1" needle="$2" haystack="$3"
    if echo "$haystack" | grep -q "$needle"; then
        echo "PASS: $name"; PASS=$((PASS+1))
    else
        echo "FAIL: $name (expected to find '$needle')"; FAIL=$((FAIL+1))
    fi
}

not_contains() {
    local name="$1" needle="$2" haystack="$3"
    if echo "$haystack" | grep -q "$needle"; then
        echo "FAIL: $name (should NOT contain '$needle')"; FAIL=$((FAIL+1))
    else
        echo "PASS: $name"; PASS=$((PASS+1))
    fi
}

sql() {
    sqlite3 "$DB" "$1"
}

# Minimal JPEG header for chunk uploads
JPEG_B64=$(printf '\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00' | base64 -w0 2>/dev/null || printf '\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00' | base64 2>/dev/null)

echo "=== Audit Fixes Verification Suite ==="

# ─── Setup ──────────────────────────────────────────────────────────────
curl -s -c "$ADMIN_CK" -X POST "$BASE/auth/register" -H "Content-Type: application/json" \
  -d '{"username":"afadmin","password":"AuditFixAdm12"}' > /dev/null

curl -s -b "$ADMIN_CK" -X POST "$BASE/users" -H "Content-Type: application/json" \
  -d '{"username":"afstaff","password":"AuditFixStf12","role":"operations_staff"}' > /dev/null
curl -s -b "$ADMIN_CK" -X POST "$BASE/users" -H "Content-Type: application/json" \
  -d '{"username":"afauditor","password":"AuditFixAud12","role":"auditor"}' > /dev/null

curl -s -c "$STAFF_CK" -X POST "$BASE/auth/login" -H "Content-Type: application/json" \
  -d '{"username":"afstaff","password":"AuditFixStf12"}' > /dev/null
curl -s -c "$AUDITOR_CK" -X POST "$BASE/auth/login" -H "Content-Type: application/json" \
  -d '{"username":"afauditor","password":"AuditFixAud12"}' > /dev/null

# ═══════════════════════════════════════════════════════════════════════
# 1. FINGERPRINT INTEGRITY — server-side verification
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "━━━ 1. Fingerprint integrity verification ━━━"

# Upload a photo and provide the CORRECT server-computed fingerprint
UBODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" -H "Content-Type: application/json" \
  -d '{"filename":"fp_test.jpg","media_type":"photo","total_size":1024,"duration_seconds":0}')
UPLOAD_ID=$(echo "$UBODY" | grep -o '"upload_id":"[^"]*"' | cut -d'"' -f4)

curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/chunk" -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$UPLOAD_ID\",\"chunk_index\":0,\"data\":\"$JPEG_B64\"}" > /dev/null

# Compute the correct fingerprint from the JPEG data
JPEG_RAW=$(printf '\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00')
CORRECT_FP=$(echo -n "$JPEG_RAW" | sha256sum | cut -d' ' -f1)

# Happy path: correct fingerprint accepted
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/complete" \
  -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$UPLOAD_ID\",\"fingerprint\":\"$CORRECT_FP\",\"total_size\":1024}")
check "Correct fingerprint → 201" "201" "$R"

# Upload another file, provide WRONG fingerprint
UBODY2=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" -H "Content-Type: application/json" \
  -d '{"filename":"fp_bad.jpg","media_type":"photo","total_size":1024,"duration_seconds":0}')
UPLOAD_ID2=$(echo "$UBODY2" | grep -o '"upload_id":"[^"]*"' | cut -d'"' -f4)

curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/chunk" -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$UPLOAD_ID2\",\"chunk_index\":0,\"data\":\"$JPEG_B64\"}" > /dev/null

# Wrong fingerprint: must get 409 CONFLICT
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/complete" \
  -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$UPLOAD_ID2\",\"fingerprint\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"total_size\":1024}")
check "Wrong fingerprint → 409 CONFLICT" "409" "$R"

RBODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/complete" -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$UPLOAD_ID2\",\"fingerprint\":\"deadbeefdeadbeefdeadbeefdeadbeef\",\"total_size\":1024}")
contains "Mismatch error contains 'Fingerprint mismatch'" "Fingerprint mismatch" "$RBODY"

# ═══════════════════════════════════════════════════════════════════════
# 2. DURATION FAIL-SAFE — server-side enforcement
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "━━━ 2. Duration policy enforcement ━━━"

# Video with valid duration (30s) — accepted at start
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" \
  -H "Content-Type: application/json" \
  -d '{"filename":"dur_ok.mp4","media_type":"video","total_size":1048576,"duration_seconds":30}')
check "Video 30s at start → 200" "200" "$R"

# Video with over-limit duration (90s) — rejected at start
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" \
  -H "Content-Type: application/json" \
  -d '{"filename":"dur_bad.mp4","media_type":"video","total_size":1048576,"duration_seconds":90}')
check "Video 90s at start → 400" "400" "$R"

# Audio with over-limit duration (180s) — rejected at start
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" \
  -H "Content-Type: application/json" \
  -d '{"filename":"dur_bad.wav","media_type":"audio","total_size":1048576,"duration_seconds":180}')
check "Audio 180s at start → 400" "400" "$R"

# Video with duration=0 — accepted at start but REJECTED at complete (fail-safe)
VBODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" -H "Content-Type: application/json" \
  -d '{"filename":"dur_bypass.mp4","media_type":"video","total_size":1048576,"duration_seconds":0}')
VID=$(echo "$VBODY" | grep -o '"upload_id":"[^"]*"' | cut -d'"' -f4)

# Create a minimal MP4-like chunk (ftyp box magic)
MP4_B64=$(printf '\x00\x00\x00\x20ftypisom\x00\x00\x00\x00isom\x00\x00\x00\x00' | base64 -w0 2>/dev/null || printf '\x00\x00\x00\x20ftypisom\x00\x00\x00\x00isom\x00\x00\x00\x00' | base64 2>/dev/null)
curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/chunk" -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$VID\",\"chunk_index\":0,\"data\":\"$MP4_B64\"}" > /dev/null

R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/complete" \
  -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$VID\",\"fingerprint\":\"abc12345abc12345\",\"total_size\":1048576}")
check "Video duration=0 at complete → 400 (fail-safe)" "400" "$R"

# Audio with duration=0 — also rejected at complete
ABODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" -H "Content-Type: application/json" \
  -d '{"filename":"dur_bypass.wav","media_type":"audio","total_size":1048576,"duration_seconds":0}')
AID=$(echo "$ABODY" | grep -o '"upload_id":"[^"]*"' | cut -d'"' -f4)

WAV_B64=$(printf 'RIFF\x00\x00\x00\x00WAVE\x00\x00\x00\x00' | base64 -w0 2>/dev/null || printf 'RIFF\x00\x00\x00\x00WAVE\x00\x00\x00\x00' | base64 2>/dev/null)
curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/chunk" -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$AID\",\"chunk_index\":0,\"data\":\"$WAV_B64\"}" > /dev/null

R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/complete" \
  -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$AID\",\"fingerprint\":\"abc12345abc12345\",\"total_size\":1048576}")
check "Audio duration=0 at complete → 400 (fail-safe)" "400" "$R"

# Photo with duration=0 — should still succeed (no duration constraint)
PBODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/start" -H "Content-Type: application/json" \
  -d '{"filename":"photo_ok.jpg","media_type":"photo","total_size":1024,"duration_seconds":0}')
PID=$(echo "$PBODY" | grep -o '"upload_id":"[^"]*"' | cut -d'"' -f4)

curl -s -b "$ADMIN_CK" -X POST "$BASE/media/upload/chunk" -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$PID\",\"chunk_index\":0,\"data\":\"$JPEG_B64\"}" > /dev/null

PHOTO_FP=$(printf '\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00' | sha256sum | cut -d' ' -f1)
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/media/upload/complete" \
  -H "Content-Type: application/json" \
  -d "{\"upload_id\":\"$PID\",\"fingerprint\":\"$PHOTO_FP\",\"total_size\":1024}")
check "Photo duration=0 at complete → 201 (no constraint)" "201" "$R"

# ═══════════════════════════════════════════════════════════════════════
# 3. TRACEABILITY STEPS — auditor visibility policy
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "━━━ 3. Traceability steps visibility ━━━"

# Create a traceability code (draft status)
TBODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/traceability" -H "Content-Type: application/json" \
  -d '{"intake_id":null}')
TRACE_ID=$(echo "$TBODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

# Admin can see steps of draft code
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" "$BASE/traceability/$TRACE_ID/steps")
check "Admin can see draft code steps → 200" "200" "$R"

# Staff can see steps of draft code
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$STAFF_CK" "$BASE/traceability/$TRACE_ID/steps")
check "Staff can see draft code steps → 200" "200" "$R"

# Auditor CANNOT see steps of draft code
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$AUDITOR_CK" "$BASE/traceability/$TRACE_ID/steps")
check "Auditor cannot see draft code steps → 403" "403" "$R"

# Publish the code
curl -s -b "$ADMIN_CK" -X POST "$BASE/traceability/$TRACE_ID/publish" -H "Content-Type: application/json" \
  -d '{"comment":"Publishing for audit test"}' > /dev/null

# Auditor CAN see steps of published code
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$AUDITOR_CK" "$BASE/traceability/$TRACE_ID/steps")
check "Auditor can see published code steps → 200" "200" "$R"

# Retract the code
curl -s -b "$ADMIN_CK" -X POST "$BASE/traceability/$TRACE_ID/retract" -H "Content-Type: application/json" \
  -d '{"comment":"Retracting for audit test"}' > /dev/null

# Auditor CANNOT see steps of retracted code
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$AUDITOR_CK" "$BASE/traceability/$TRACE_ID/steps")
check "Auditor cannot see retracted code steps → 403" "403" "$R"

# Admin CAN still see retracted code steps
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" "$BASE/traceability/$TRACE_ID/steps")
check "Admin can see retracted code steps → 200" "200" "$R"

# 404 for nonexistent code
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$AUDITOR_CK" "$BASE/traceability/nonexistent/steps")
check "Nonexistent code steps → 404" "404" "$R"

# ═══════════════════════════════════════════════════════════════════════
# 4. PRIVACY PREFERENCES — CRUD + user isolation
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "━━━ 4. Privacy preferences ━━━"

# GET default preferences (lazy initialization)
PREFS=$(curl -s -b "$ADMIN_CK" "$BASE/profile/privacy-preferences")
R=$(echo "$PREFS" | grep -c '"show_email":true' || true)
check "Default show_email is true" "1" "$R"

R=$(echo "$PREFS" | grep -c '"show_phone":false' || true)
check "Default show_phone is false" "1" "$R"

R=$(echo "$PREFS" | grep -c '"allow_data_sharing":false' || true)
check "Default allow_data_sharing is false" "1" "$R"

# PATCH to update preferences
PREFS=$(curl -s -b "$ADMIN_CK" -X PATCH "$BASE/profile/privacy-preferences" \
  -H "Content-Type: application/json" \
  -d '{"show_phone":true,"allow_data_sharing":true}')
R=$(echo "$PREFS" | grep -c '"show_phone":true' || true)
check "Updated show_phone to true" "1" "$R"
R=$(echo "$PREFS" | grep -c '"allow_data_sharing":true' || true)
check "Updated allow_data_sharing to true" "1" "$R"
# show_email should remain unchanged
R=$(echo "$PREFS" | grep -c '"show_email":true' || true)
check "show_email unchanged after partial update" "1" "$R"

# GET to verify persistence
PREFS=$(curl -s -b "$ADMIN_CK" "$BASE/profile/privacy-preferences")
R=$(echo "$PREFS" | grep -c '"show_phone":true' || true)
check "show_phone persisted on re-read" "1" "$R"

# User isolation: staff user has separate preferences
PREFS_STAFF=$(curl -s -b "$STAFF_CK" "$BASE/profile/privacy-preferences")
R=$(echo "$PREFS_STAFF" | grep -c '"show_phone":false' || true)
check "Staff has own default show_phone=false (isolation)" "1" "$R"

# Staff updates own preferences
curl -s -b "$STAFF_CK" -X PATCH "$BASE/profile/privacy-preferences" \
  -H "Content-Type: application/json" \
  -d '{"allow_audit_log_export":false}' > /dev/null
PREFS_STAFF=$(curl -s -b "$STAFF_CK" "$BASE/profile/privacy-preferences")
R=$(echo "$PREFS_STAFF" | grep -c '"allow_audit_log_export":false' || true)
check "Staff updated own allow_audit_log_export" "1" "$R"

# Admin's preferences unaffected by staff's changes
PREFS=$(curl -s -b "$ADMIN_CK" "$BASE/profile/privacy-preferences")
R=$(echo "$PREFS" | grep -c '"allow_audit_log_export":true' || true)
check "Admin's allow_audit_log_export unchanged (isolation)" "1" "$R"

# Requires auth
R=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/profile/privacy-preferences")
check "Privacy prefs without auth → 401" "401" "$R"

# ═══════════════════════════════════════════════════════════════════════
# 5. SUPPLY — new fields (stock_status, media_references, review_summary)
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "━━━ 5. Supply new fields ━━━"

# Create with all new fields
SBODY=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/supply-entries" -H "Content-Type: application/json" \
  -d '{"name":"Dog Food Premium","sku":"DF-001","size":"large","color":"brown","price_cents":2499,"discount_cents":0,"notes":"bulk","stock_status":"in_stock","media_references":"ev-001,ev-002","review_summary":"Good quality"}')
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/supply-entries" -H "Content-Type: application/json" \
  -d '{"name":"Test Item","sku":null,"size":"small","color":"red","price_cents":100,"discount_cents":0,"notes":"","stock_status":"in_stock","media_references":"","review_summary":""}')
check "Supply create with stock_status → 201" "201" "$R"

# Verify new fields in response
contains "stock_status in create response" "in_stock" "$SBODY"
contains "media_references in create response" "ev-001,ev-002" "$SBODY"
contains "review_summary in create response" "Good quality" "$SBODY"

# Verify list returns new fields
SLIST=$(curl -s -b "$ADMIN_CK" "$BASE/supply-entries")
contains "stock_status in list response" "in_stock" "$SLIST"
contains "media_references in list response" "ev-001,ev-002" "$SLIST"
contains "review_summary in list response" "Good quality" "$SLIST"

# Invalid stock_status rejected
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$ADMIN_CK" -X POST "$BASE/supply-entries" \
  -H "Content-Type: application/json" \
  -d '{"name":"Bad Status","sku":null,"size":"M","color":"blue","price_cents":100,"discount_cents":0,"notes":"","stock_status":"invalid_status","media_references":"","review_summary":""}')
check "Invalid stock_status → 400" "400" "$R"

# Default stock_status when not provided
R2=$(curl -s -b "$ADMIN_CK" -X POST "$BASE/supply-entries" -H "Content-Type: application/json" \
  -d '{"name":"Default Status","sku":null,"size":"S","color":"green","price_cents":50,"discount_cents":0,"notes":""}')
contains "Default stock_status is unknown" "unknown" "$R2"

# ═══════════════════════════════════════════════════════════════════════
# 6. COOKIE HARDENING — Secure flag behavior
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "━━━ 6. Cookie hardening ━━━"

# In the test environment, COOKIE_SECURE defaults to false.
# Verify session cookie has HttpOnly, SameSite=Strict, Path=/
HEADERS=$(curl -s -D - -o /dev/null -X POST "$BASE/auth/login" -H "Content-Type: application/json" \
  -d '{"username":"afadmin","password":"AuditFixAdm12"}')
contains "Cookie has HttpOnly" "HttpOnly" "$HEADERS"
contains "Cookie has SameSite=Strict" "SameSite=Strict" "$HEADERS"
contains "Cookie has Path=/" "Path=/" "$HEADERS"

# Without COOKIE_SECURE=true, Secure attribute should NOT be present
# (local dev mode). This is correct behavior — Secure is only added
# in production HTTPS mode.
not_contains "Cookie does not have Secure in HTTP mode" "; Secure" "$HEADERS"

# ═══════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════
echo ""
echo "========================================"
echo "  Audit Fixes: $PASS passed, $FAIL failed"
echo "========================================"
[ $FAIL -eq 0 ]
