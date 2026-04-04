#!/bin/bash
set -e
PASS=0; FAIL=0
BASE="http://localhost:8080"
CK="/tmp/auth_test_cookies"

echo "=== API Tests: Auth & Users ==="

# ── Register ─────────────────────────────────────────────────────────

# Test 1: Register first admin
echo "--- Test: POST /auth/register bootstrap ---"
R=$(curl -s -w "\n%{http_code}" -c "$CK" -X POST "$BASE/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"testadmin","password":"SecurePass12"}')
CODE=$(echo "$R" | tail -1)
BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "201" ] && echo "$BODY" | grep -q '"administrator"'; then
    echo "PASS: First admin registered (201)"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 201+administrator. Code=$CODE Body=$BODY"; FAIL=$((FAIL+1))
fi

# Test 2: Register again → blocked
echo "--- Test: POST /auth/register blocked after bootstrap ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE/auth/register" \
  -H "Content-Type: application/json" \
  -d '{"username":"another","password":"SecurePass12"}')
if [ "$R" = "409" ]; then
    echo "PASS: Second register blocked (409)"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 409, got $R"; FAIL=$((FAIL+1))
fi

# ── Login / Session ──────────────────────────────────────────────────

# Test 3: Login success
echo "--- Test: POST /auth/login success ---"
R=$(curl -s -w "\n%{http_code}" -c "$CK" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"testadmin","password":"SecurePass12"}')
CODE=$(echo "$R" | tail -1)
if [ "$CODE" = "200" ]; then
    echo "PASS: Login success (200)"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 200, got $CODE"; FAIL=$((FAIL+1))
fi

# Test 4: Session cookie received
echo "--- Test: Session cookie set ---"
if grep -q "session_id" "$CK" 2>/dev/null; then
    echo "PASS: session_id cookie present"; PASS=$((PASS+1))
else
    echo "FAIL: No session_id cookie"; FAIL=$((FAIL+1))
fi

# Test 5: GET /auth/me with session
echo "--- Test: GET /auth/me with session ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" "$BASE/auth/me")
CODE=$(echo "$R" | tail -1)
BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "200" ] && echo "$BODY" | grep -q '"testadmin"'; then
    echo "PASS: /auth/me returns current user"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 200+testadmin. Code=$CODE Body=$BODY"; FAIL=$((FAIL+1))
fi

# Test 6: GET /auth/me without session → 401
echo "--- Test: GET /auth/me without session → 401 ---"
R=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/auth/me")
if [ "$R" = "401" ]; then
    echo "PASS: No session → 401"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 401, got $R"; FAIL=$((FAIL+1))
fi

# Test 7: Login wrong password → 401
echo "--- Test: Wrong password → 401 ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"testadmin","password":"WrongPassword!"}')
if [ "$R" = "401" ]; then
    echo "PASS: Wrong password → 401"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 401, got $R"; FAIL=$((FAIL+1))
fi

# ── Admin user creation ──────────────────────────────────────────────

# Test 8: Admin can create user
echo "--- Test: POST /users (admin creates staff) ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" -X POST "$BASE/users" \
  -H "Content-Type: application/json" \
  -d '{"username":"staffuser","password":"StaffPass1234","role":"operations_staff"}')
CODE=$(echo "$R" | tail -1)
if [ "$CODE" = "201" ]; then
    echo "PASS: Admin created staff user"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 201, got $CODE"; FAIL=$((FAIL+1))
fi

# Test 9: Admin can create auditor
echo "--- Test: POST /users (admin creates auditor) ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" -X POST "$BASE/users" \
  -H "Content-Type: application/json" \
  -d '{"username":"auditor1","password":"AuditorPass12","role":"auditor"}')
CODE=$(echo "$R" | tail -1)
if [ "$CODE" = "201" ]; then
    echo "PASS: Admin created auditor"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 201, got $CODE"; FAIL=$((FAIL+1))
fi

# Test 10: GET /users (admin lists users)
echo "--- Test: GET /users (admin) ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" "$BASE/users")
CODE=$(echo "$R" | tail -1)
BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "200" ] && echo "$BODY" | grep -q '"staffuser"'; then
    echo "PASS: Admin can list users"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 200+staffuser. Code=$CODE"; FAIL=$((FAIL+1))
fi

# ── Role guard (403) ─────────────────────────────────────────────────

# Test 11: Staff user cannot access /users → 403
echo "--- Test: Staff → GET /users → 403 ---"
CK2="/tmp/auth_test_cookies_staff"
curl -s -c "$CK2" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"staffuser","password":"StaffPass1234"}' > /dev/null
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK2" "$BASE/users")
if [ "$R" = "403" ]; then
    echo "PASS: Staff blocked from /users (403)"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 403, got $R"; FAIL=$((FAIL+1))
fi

# Test 12: Auditor cannot access /users → 403
echo "--- Test: Auditor → GET /users → 403 ---"
CK3="/tmp/auth_test_cookies_auditor"
curl -s -c "$CK3" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"auditor1","password":"AuditorPass12"}' > /dev/null
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK3" "$BASE/users")
if [ "$R" = "403" ]; then
    echo "PASS: Auditor blocked from /users (403)"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 403, got $R"; FAIL=$((FAIL+1))
fi

# ── Change password ──────────────────────────────────────────────────

# Test 13: Change password
echo "--- Test: PATCH /auth/change-password ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X PATCH "$BASE/auth/change-password" \
  -H "Content-Type: application/json" \
  -d '{"current_password":"SecurePass12","new_password":"NewSecure1234"}')
if [ "$R" = "200" ]; then
    echo "PASS: Password changed"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 200, got $R"; FAIL=$((FAIL+1))
fi

# Test 14: Login with new password
echo "--- Test: Login with new password ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -c "$CK" -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"testadmin","password":"NewSecure1234"}')
if [ "$R" = "200" ]; then
    echo "PASS: Login with new password works"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 200, got $R"; FAIL=$((FAIL+1))
fi

# ── Logout ───────────────────────────────────────────────────────────

# Test 15: Logout clears session
echo "--- Test: POST /auth/logout ---"
curl -s -b "$CK" -c "$CK" -X POST "$BASE/auth/logout" > /dev/null
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" "$BASE/auth/me")
if [ "$R" = "401" ]; then
    echo "PASS: Session cleared after logout"; PASS=$((PASS+1))
else
    echo "FAIL: Expected 401 after logout, got $R"; FAIL=$((FAIL+1))
fi

# ── Error envelope ───────────────────────────────────────────────────

# Test 16: Error response has trace_id
echo "--- Test: Error response includes trace_id ---"
BODY=$(curl -s "$BASE/auth/me")
if echo "$BODY" | grep -q '"trace_id"'; then
    echo "PASS: Error envelope has trace_id"; PASS=$((PASS+1))
else
    echo "FAIL: Missing trace_id in error. Body=$BODY"; FAIL=$((FAIL+1))
fi

# ── Slice 1 regression ──────────────────────────────────────────────

# Test 17: Health still works
echo "--- Test: GET /health still returns ok ---"
R=$(curl -s "$BASE/health")
if echo "$R" | grep -q '"status":"ok"'; then
    echo "PASS: Health endpoint intact"; PASS=$((PASS+1))
else
    echo "FAIL: Health broken: $R"; FAIL=$((FAIL+1))
fi

# Test 18: X-Trace-Id still present
echo "--- Test: X-Trace-Id header still present ---"
H=$(curl -sI "$BASE/health")
if echo "$H" | grep -qi "x-trace-id"; then
    echo "PASS: X-Trace-Id header present"; PASS=$((PASS+1))
else
    echo "FAIL: X-Trace-Id missing"; FAIL=$((FAIL+1))
fi

# Cleanup
rm -f "$CK" "$CK2" "$CK3"

echo ""
echo "========================================"
echo "  API Tests (Auth) - Passed: $PASS  Failed: $FAIL"
echo "========================================"
[ $FAIL -gt 0 ] && exit 1
exit 0
