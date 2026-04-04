#!/bin/bash
set -e

PASS=0
FAIL=0

echo "=== Unit Tests: Bootstrap ==="

# Test 1: App started successfully (migrations ran, DB connected)
echo "--- Test: Migration bootstrap ---"
RESPONSE=$(wget -qO- http://localhost:8080/health 2>&1)
if echo "$RESPONSE" | grep -q '"status":"ok"'; then
    echo "PASS: App bootstrapped, migrations ran, DB connected"
    PASS=$((PASS + 1))
else
    echo "FAIL: App did not bootstrap correctly"
    FAIL=$((FAIL + 1))
fi

# Test 2: SQLite database file exists
echo "--- Test: SQLite DB file created ---"
if [ -f /app/storage/app.db ]; then
    echo "PASS: SQLite database file exists at /app/storage/app.db"
    PASS=$((PASS + 1))
else
    echo "FAIL: SQLite database file not found"
    FAIL=$((FAIL + 1))
fi

# Summary
echo ""
echo "========================================"
echo "  Unit Tests - Passed: $PASS  Failed: $FAIL"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
