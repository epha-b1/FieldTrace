#!/bin/bash
set -e
PASS=0; FAIL=0; BASE="http://localhost:8080"; CK="/tmp/intake_ck"

echo "=== API Tests: Intake + Inspections (Slice 4) ==="

# Setup: register + login
curl -s -c "$CK" -X POST "$BASE/auth/register" -H "Content-Type: application/json" \
  -d '{"username":"intadmin","password":"SecurePass12"}' > /dev/null

# Test 1: Create intake record
echo "--- Test: Create intake (animal) ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" -X POST "$BASE/intake" \
  -H "Content-Type: application/json" -d '{"intake_type":"animal","details":"{}"}')
CODE=$(echo "$R" | tail -1); BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "201" ]; then
    echo "PASS: Intake created (201)"; PASS=$((PASS+1))
    INTAKE_ID=$(echo "$BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
else echo "FAIL: Got $CODE $BODY"; FAIL=$((FAIL+1)); fi

# Test 2: List intake
echo "--- Test: List intake ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" "$BASE/intake")
if [ "$R" = "200" ]; then echo "PASS: List intake (200)"; PASS=$((PASS+1))
else echo "FAIL: Got $R"; FAIL=$((FAIL+1)); fi

# Test 3: Get single intake
echo "--- Test: Get intake by ID ---"
if [ -n "$INTAKE_ID" ]; then
    R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" "$BASE/intake/$INTAKE_ID")
    if [ "$R" = "200" ]; then echo "PASS: Get intake (200)"; PASS=$((PASS+1))
    else echo "FAIL: Got $R"; FAIL=$((FAIL+1)); fi
else echo "SKIP"; FAIL=$((FAIL+1)); fi

# Test 4: Valid state transition (received → in_care)
echo "--- Test: Valid transition received→in_care ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X PATCH "$BASE/intake/$INTAKE_ID/status" \
  -H "Content-Type: application/json" -d '{"status":"in_care"}')
if [ "$R" = "200" ]; then echo "PASS: Valid transition (200)"; PASS=$((PASS+1))
else echo "FAIL: Got $R"; FAIL=$((FAIL+1)); fi

# Test 5: Invalid state transition (in_care → received) → 409
echo "--- Test: Invalid transition in_care→received → 409 ---"
R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X PATCH "$BASE/intake/$INTAKE_ID/status" \
  -H "Content-Type: application/json" -d '{"status":"received"}')
if [ "$R" = "409" ]; then echo "PASS: Invalid transition rejected (409)"; PASS=$((PASS+1))
else echo "FAIL: Expected 409, got $R"; FAIL=$((FAIL+1)); fi

# Test 6: Create inspection linked to intake
echo "--- Test: Create inspection ---"
R=$(curl -s -w "\n%{http_code}" -b "$CK" -X POST "$BASE/inspections" \
  -H "Content-Type: application/json" -d "{\"intake_id\":\"$INTAKE_ID\"}")
CODE=$(echo "$R" | tail -1); BODY=$(echo "$R" | sed '$d')
if [ "$CODE" = "201" ]; then
    echo "PASS: Inspection created (201)"; PASS=$((PASS+1))
    INSP_ID=$(echo "$BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
else echo "FAIL: Got $CODE $BODY"; FAIL=$((FAIL+1)); fi

# Test 7: Resolve inspection
echo "--- Test: Resolve inspection ---"
if [ -n "$INSP_ID" ]; then
    R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X PATCH "$BASE/inspections/$INSP_ID/resolve" \
      -H "Content-Type: application/json" -d '{"status":"passed","outcome_notes":"All good"}')
    if [ "$R" = "200" ]; then echo "PASS: Inspection resolved (200)"; PASS=$((PASS+1))
    else echo "FAIL: Got $R"; FAIL=$((FAIL+1)); fi
else echo "SKIP"; FAIL=$((FAIL+1)); fi

# Test 8: Re-resolve inspection → 409
echo "--- Test: Re-resolve → 409 ---"
if [ -n "$INSP_ID" ]; then
    R=$(curl -s -o /dev/null -w "%{http_code}" -b "$CK" -X PATCH "$BASE/inspections/$INSP_ID/resolve" \
      -H "Content-Type: application/json" -d '{"status":"failed","outcome_notes":"Oops"}')
    if [ "$R" = "409" ]; then echo "PASS: Re-resolve blocked (409)"; PASS=$((PASS+1))
    else echo "FAIL: Expected 409, got $R"; FAIL=$((FAIL+1)); fi
else echo "SKIP"; FAIL=$((FAIL+1)); fi

# Test 9: No auth → 401
echo "--- Test: No auth → 401 ---"
R=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/intake")
if [ "$R" = "401" ]; then echo "PASS: No auth → 401"; PASS=$((PASS+1))
else echo "FAIL: Got $R"; FAIL=$((FAIL+1)); fi

rm -f "$CK"
echo ""
echo "========================================"
echo "  Intake+Inspections Tests - Passed: $PASS  Failed: $FAIL"
echo "========================================"
[ $FAIL -gt 0 ] && exit 1; exit 0
