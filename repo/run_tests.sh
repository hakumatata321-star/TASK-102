#!/bin/bash
##############################################################################
# RetailOps — Unified Test Runner
#
# Runs both unit tests and API integration tests.
# Usage: bash run_tests.sh
#
# Prerequisites:
#   - Docker and Docker Compose installed
#   - Ports 8081 and 5433 available (or adjust docker-compose.yml)
##############################################################################

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

UNIT_EXIT=0
COVERAGE_EXIT=0
API_EXIT=0

echo "╔════════════════════════════════════════════════════╗"
echo "║     RetailOps — Full Test Suite                    ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

###########################################################################
# PHASE 1: Unit Tests
###########################################################################
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  PHASE 1: Unit Tests (cargo test in Docker)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

PROJ_PATH="$(pwd -W 2>/dev/null || pwd)"
export MSYS_NO_PATHCONV=1

# Clean stale build artifacts to ensure fresh compilation
rm -rf target/release/.fingerprint/retailops-* target/release/deps/retailops-* 2>/dev/null || true

docker run --rm \
  -v "${PROJ_PATH}:/app" \
  -w /app \
  rust:1.88-bookworm \
  bash -c "
    apt-get update -qq && apt-get install -y -qq libpq-dev > /dev/null 2>&1
    cargo test --release 2>&1
  "
UNIT_EXIT=$?

echo ""
if [ $UNIT_EXIT -eq 0 ]; then
  echo "[UNIT] ALL UNIT TESTS PASSED"
else
  echo "[UNIT] SOME UNIT TESTS FAILED"
fi
echo ""

###########################################################################
# PHASE 1b: Coverage
###########################################################################
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  PHASE 1b: Coverage (cargo llvm-cov in Docker)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

docker run --rm \
  -v "${PROJ_PATH}:/app" \
  -w /app \
  rust:1.88-bookworm \
  bash -c "
    apt-get update -qq && apt-get install -y -qq libpq-dev > /dev/null 2>&1
    cargo install cargo-llvm-cov --locked 2>&1 | tail -3
    cargo llvm-cov test 2>&1
  "
COVERAGE_EXIT=$?

echo ""
if [ $COVERAGE_EXIT -eq 0 ]; then
  echo "[COVERAGE] Coverage report generated successfully"
else
  echo "[COVERAGE] Coverage run encountered issues (non-fatal)"
fi
echo ""

###########################################################################
# PHASE 2: Start Application
###########################################################################
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  PHASE 2: Starting Application (docker compose)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

docker compose down -v > /dev/null 2>&1 || true
echo "[INFO] Starting services..."
docker compose up -d --build 2>&1 | tail -5

echo "[INFO] Waiting for API to become healthy..."
for i in $(seq 1 60); do
  HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:8081/api/v1/health" 2>/dev/null || true)
  if [ "$HEALTH" = "200" ]; then
    echo "[INFO] API healthy after ${i}s"
    break
  fi
  if [ "$i" = "60" ]; then
    echo "[ERROR] API not healthy after 60s."
    docker compose logs api 2>&1 | tail -20
    docker compose down -v > /dev/null 2>&1 || true
    exit 1
  fi
  sleep 1
done
echo ""

###########################################################################
# PHASE 3: API Integration Tests
###########################################################################
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  PHASE 3: API Integration Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

unset MSYS_NO_PATHCONV
bash API_tests/run_api_tests.sh
API_EXIT=$?

###########################################################################
# PHASE 4: Teardown
###########################################################################
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  PHASE 4: Teardown"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
docker compose down -v > /dev/null 2>&1
echo "[INFO] Services stopped and volumes cleaned."

###########################################################################
# COMBINED SUMMARY
###########################################################################
echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║              COMBINED TEST RESULTS                 ║"
echo "╠════════════════════════════════════════════════════╣"
if [ $UNIT_EXIT -eq 0 ]; then
  echo "║  Unit Tests:    PASSED                             ║"
else
  echo "║  Unit Tests:    FAILED                             ║"
fi
if [ $COVERAGE_EXIT -eq 0 ]; then
  echo "║  Coverage:      PASSED                             ║"
else
  echo "║  Coverage:      WARNING (non-fatal)                ║"
fi
if [ $API_EXIT -eq 0 ]; then
  echo "║  API Tests:     PASSED                             ║"
else
  echo "║  API Tests:     FAILED                             ║"
fi
echo "╚════════════════════════════════════════════════════╝"
echo ""

if [ $UNIT_EXIT -ne 0 ] || [ $API_EXIT -ne 0 ]; then
  echo "RESULT: SOME TESTS FAILED"
  exit 1
else
  echo "RESULT: ALL TESTS PASSED"
  exit 0
fi
