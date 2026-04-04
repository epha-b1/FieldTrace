#!/bin/bash
set -e
PASS=0; FAIL=0; BASE="http://localhost:8080"; CK="/tmp/ab_ck"

echo "=== API Tests: Address Book (Slice 3) ==="

# Setup: register + login admin
curl -s -c "$CK" -X POST "$BASE/auth/register" -H "Content-Type: application/json" \
  -d '{"username":"abadmin","password":"SecurePass12"}' > /dev/null

# Test 1: Create address with valid ZIP+4
echo "--- Test: Create address (valid ZIP+4) ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" -X POST "$BASE/address-book" \
  -H "Content-Type: application/json" \
  -d '{"label":"Home","street":"123 Main St","city":"Anytown","state":"CA","zip_plus4":"90210-1234","phone":"555-867-5309"}')
CODE=$(echo "$R" | tail -1); BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "201" ] && echo "$BODY" | grep -q '"Home"'; then
    echo "PASS: Address created (201)"; PASS=$((PASS+1))
    ADDR_ID=$(echo "$BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
else echo "FAIL: Got $CODE $BODY"; FAIL=$((FAIL+1)); fi

# Test 2: Phone is masked in response
echo "--- Test: Phone masked in response ---"
if echo "$BODY" | grep -q '***-***-5309'; then
    echo "PASS: Phone masked correctly"; PASS=$((PASS+1))
else echo "FAIL: Phone not masked: $BODY"; FAIL=$((FAIL+1)); fi

# Test 3: Invalid ZIP+4 → 400
echo "--- Test: Invalid ZIP+4 rejected ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X POST "$BASE/address-book" \
  -H "Content-Type: application/json" \
  -d '{"label":"Bad","street":"x","city":"x","state":"x","zip_plus4":"1234","phone":"555"}')
if [ "$R" = "400" ]; then
    echo "PASS: Invalid ZIP rejected (400)"; PASS=$((PASS+1))
else echo "FAIL: Expected 400, got $R"; FAIL=$((FAIL+1)); fi

# Test 4: List addresses returns entries
echo "--- Test: List own addresses ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" "$BASE/address-book")
CODE=$(echo "$R" | tail -1); BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "200" ] && echo "$BODY" | grep -q '"Home"'; then
    echo "PASS: List returns own entries"; PASS=$((PASS+1))
else echo "FAIL: Got $CODE"; FAIL=$((FAIL+1)); fi

# Test 5: Object-level auth — other user cannot see my addresses
echo "--- Test: User B cannot see User A addresses ---"
CK2="/tmp/ab_ck2"
curl -s -b "$CK" -X POST "$BASE/users" -H "Content-Type: application/json" \
  -d '{"username":"abstaff","password":"StaffPass1234","role":"operations_staff"}' > /dev/null
curl -s -c "$CK2" -X POST "$BASE/auth/login" -H "Content-Type: application/json" \
  -d '{"username":"abstaff","password":"StaffPass1234"}' > /dev/null
R=$(curl -s -b "$CK2" "$BASE/address-book")
if echo "$R" | grep -q '\[\]'; then
    echo "PASS: Other user sees empty list"; PASS=$((PASS+1))
else echo "FAIL: Other user saw data: $R"; FAIL=$((FAIL+1)); fi

# Test 6: User B cannot delete User A's address
echo "--- Test: User B cannot delete User A address ---"
if [ -n "$ADDR_ID" ]; then
    R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK2" -X DELETE "$BASE/address-book/$ADDR_ID")
    if [ "$R" = "404" ]; then
        echo "PASS: Cross-user delete blocked (404)"; PASS=$((PASS+1))
    else echo "FAIL: Expected 404, got $R"; FAIL=$((FAIL+1)); fi
else echo "SKIP: No ADDR_ID"; FAIL=$((FAIL+1)); fi

# Test 7: Owner can delete own address
echo "--- Test: Owner can delete own address ---"
if [ -n "$ADDR_ID" ]; then
    R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X DELETE "$BASE/address-book/$ADDR_ID")
    if [ "$R" = "200" ]; then
        echo "PASS: Owner deleted address"; PASS=$((PASS+1))
    else echo "FAIL: Expected 200, got $R"; FAIL=$((FAIL+1)); fi
else echo "SKIP: No ADDR_ID"; FAIL=$((FAIL+1)); fi

# Test 8: No auth → 401
echo "--- Test: No auth → 401 ---"
R=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/address-book")
if [ "$R" = "401" ]; then
    echo "PASS: Unauthenticated → 401"; PASS=$((PASS+1))
else echo "FAIL: Expected 401, got $R"; FAIL=$((FAIL+1)); fi

rm -f "$CK" "$CK2"
echo ""
echo "========================================"
echo "  Address Book API Tests - Passed: $PASS  Failed: $FAIL"
echo "========================================"
[ $FAIL -gt 0 ] && exit 1; exit 0
