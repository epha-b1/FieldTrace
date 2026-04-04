#!/bin/bash
set -e

PROJECT="w2t52"
DC="docker compose -p $PROJECT"
HEALTH_URL="http://localhost:8080/health"
MAX_WAIT=120

wait_healthy() {
    local elapsed=0
    while [ $elapsed -lt $MAX_WAIT ]; do
        if $DC exec -T api wget -qO- "$HEALTH_URL" 2>/dev/null | grep -q '"status"'; then
            echo "      Ready (${elapsed}s)"
            return 0
        fi
        sleep 2; elapsed=$((elapsed + 2))
    done
    echo "ERROR: API not healthy after ${MAX_WAIT}s"
    $DC logs --tail 50 api
    return 1
}

reset_db() {
    $DC exec -T api rm -f /app/storage/app.db /app/storage/app.db-wal /app/storage/app.db-shm 2>/dev/null
    $DC restart api >/dev/null 2>&1
    sleep 3
    wait_healthy
}

TOTAL_FAIL=0

run_suite() {
    local name="$1" script="$2"
    echo "  [$name]..."
    local EXIT=0
    $DC exec -T api bash "/app/$script" || EXIT=$?
    if [ $EXIT -eq 0 ]; then
        echo "  $name: PASSED"
    else
        echo "  $name: FAILED"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
}

# Step 1: Start
echo "[Step 1] Starting containers..."
$DC up -d --build 2>&1 | tail -3

echo "[Step 2] Waiting for API..."
wait_healthy

echo "[Step 3] Slice 1 tests..."
run_suite "S1-Unit" "unit_tests/bootstrap_test.sh"
run_suite "S1-API" "API_tests/health_api_test.sh"

echo "[Step 4] Slice 2 tests..."
reset_db
run_suite "S2-API-Auth" "API_tests/auth_api_test.sh"
reset_db
run_suite "S2-Unit-Auth" "unit_tests/auth_test.sh"

echo "[Step 5] Slice 3 tests..."
reset_db
run_suite "S3-API-AddrBook" "API_tests/address_book_api_test.sh"

# Summary
echo "========================================"
if [ $TOTAL_FAIL -eq 0 ]; then
    echo "  ALL SUITES PASSED"
else
    echo "  FAILED SUITES: $TOTAL_FAIL"
fi
echo "========================================"
exit $TOTAL_FAIL
