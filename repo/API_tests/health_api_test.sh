#!/bin/bash
set -e

PASS=0
FAIL=0

echo "=== API Tests: Health + Trace ID ==="

# Test 1: Health endpoint returns correct JSON
echo "--- Test: GET /health returns {\"status\":\"ok\"} ---"
RESPONSE=$(wget -qO- http://localhost:8080/health 2>&1)
if echo "$RESPONSE" | grep -q '"status":"ok"'; then
    echo "PASS: Health returns correct JSON"
    PASS=$((PASS + 1))
else
    echo "FAIL: Unexpected response: $RESPONSE"
    FAIL=$((FAIL + 1))
fi

# Test 2: X-Trace-Id header is present on health
echo "--- Test: X-Trace-Id header present ---"
HEADERS=$(wget -S -qO /dev/null http://localhost:8080/health 2>&1)
if echo "$HEADERS" | grep -qi "X-Trace-Id"; then
    echo "PASS: X-Trace-Id header present"
    PASS=$((PASS + 1))
else
    echo "FAIL: X-Trace-Id header missing"
    echo "Response: $HEADERS"
    FAIL=$((FAIL + 1))
fi

# Test 3: X-Trace-Id is a valid UUID
echo "--- Test: X-Trace-Id is UUID format ---"
TRACE_ID=$(echo "$HEADERS" | grep -i "X-Trace-Id" | sed 's/.*: *//' | tr -d '\r\n ')
if echo "$TRACE_ID" | grep -qE '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'; then
    echo "PASS: Trace ID is valid UUID: $TRACE_ID"
    PASS=$((PASS + 1))
else
    echo "FAIL: Trace ID not a valid UUID: '$TRACE_ID'"
    FAIL=$((FAIL + 1))
fi

# Test 4: Two requests produce different trace IDs
echo "--- Test: Unique trace IDs per request ---"
HEADERS2=$(wget -S -qO /dev/null http://localhost:8080/health 2>&1)
TRACE_ID2=$(echo "$HEADERS2" | grep -i "X-Trace-Id" | sed 's/.*: *//' | tr -d '\r\n ')
if [ "$TRACE_ID" != "$TRACE_ID2" ] && [ -n "$TRACE_ID" ] && [ -n "$TRACE_ID2" ]; then
    echo "PASS: Trace IDs are unique ($TRACE_ID vs $TRACE_ID2)"
    PASS=$((PASS + 1))
else
    echo "FAIL: Trace IDs not unique or empty"
    FAIL=$((FAIL + 1))
fi

# Test 5: Frontend index.html is served at root
echo "--- Test: Frontend static files served ---"
FRONTEND=$(wget -qO- http://localhost:8080/ 2>&1)
if echo "$FRONTEND" | grep -q "FieldTrace"; then
    echo "PASS: Frontend index.html served at /"
    PASS=$((PASS + 1))
else
    echo "FAIL: Frontend not served at /"
    FAIL=$((FAIL + 1))
fi

# Test 6: WASM file referenced in HTML
echo "--- Test: WASM referenced in HTML ---"
if echo "$FRONTEND" | grep -q "\.wasm\|\.js"; then
    echo "PASS: WASM/JS assets referenced in HTML"
    PASS=$((PASS + 1))
else
    echo "FAIL: No WASM/JS reference in HTML"
    FAIL=$((FAIL + 1))
fi

# Test 7: Health endpoint has correct content-type
echo "--- Test: Content-Type is application/json ---"
if echo "$HEADERS" | grep -qi "content-type.*application/json"; then
    echo "PASS: Content-Type is application/json"
    PASS=$((PASS + 1))
else
    echo "FAIL: Content-Type not application/json"
    FAIL=$((FAIL + 1))
fi

# Summary
echo ""
echo "========================================"
echo "  API Tests - Passed: $PASS  Failed: $FAIL"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
