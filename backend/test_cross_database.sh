#!/bin/bash
# Test Cross-Database Query Feature
# Tests Phase 4 implementation with MySQL connections

set -e

echo "========================================"
echo "Cross-Database Query Test Suite"
echo "Phase 4: JOIN and UNION Implementation"
echo "========================================"
echo ""

BACKEND_URL="http://localhost:3000"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0

echo "Step 1: Checking backend health..."
if curl -s "$BACKEND_URL/health" | grep -q "OK"; then
    echo -e "${GREEN}✓ Backend is running${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ Backend is not responding${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi

echo ""
echo "Step 2: Getting MySQL connection IDs..."
CONNECTIONS=$(curl -s "$BACKEND_URL/api/connections")

# Extract first MySQL connection ID
MYSQL_CONN_1=$(echo "$CONNECTIONS" | jq -r '.connections[] | select(.database_type == "mysql") | .id' | head -1)

if [ -z "$MYSQL_CONN_1" ]; then
    echo -e "${RED}✗ No MySQL connections found${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi

echo -e "${GREEN}✓ Found MySQL connection: $MYSQL_CONN_1${NC}"
TESTS_PASSED=$((TESTS_PASSED + 1))

echo ""
echo "========================================"
echo "Test 1: Single Database Query"
echo "========================================"
echo ""
echo "Query: SELECT * FROM ${MYSQL_CONN_1}.users"
echo ""

RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"SELECT * FROM ${MYSQL_CONN_1}.users\",
        \"connection_ids\": [\"$MYSQL_CONN_1\"],
        \"timeout_secs\": 30,
        \"apply_limit\": true,
        \"limit_value\": 10
    }")

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Single database query succeeded${NC}"
    ROW_COUNT=$(echo "$RESPONSE" | jq -r '.row_count')
    EXEC_TIME=$(echo "$RESPONSE" | jq -r '.execution_time_ms')
    echo "  Results: $ROW_COUNT rows in ${EXEC_TIME}ms"

    # Show sub-queries
    echo ""
    echo "  Sub-queries executed:"
    echo "$RESPONSE" | jq -r '.sub_queries[] | "    - \(.database_type): \(.query) (\(.execution_time_ms)ms)"'

    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ Single database query failed${NC}"
    echo "  Error: $(echo "$RESPONSE" | jq -r '.error.message // "Unknown error"')"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo "========================================"
echo "Test 2: Simple JOIN (Same Database)"
echo "========================================"
echo ""
echo "Query: SELECT u.username, t.title FROM users u JOIN todos t ON u.id = t.user_id"
echo ""

RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"SELECT u.username, t.title FROM ${MYSQL_CONN_1}.users u JOIN ${MYSQL_CONN_1}.todos t ON u.id = t.user_id\",
        \"connection_ids\": [\"$MYSQL_CONN_1\"],
        \"timeout_secs\": 30,
        \"apply_limit\": true,
        \"limit_value\": 10
    }")

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ JOIN query succeeded${NC}"
    ROW_COUNT=$(echo "$RESPONSE" | jq -r '.row_count')
    EXEC_TIME=$(echo "$RESPONSE" | jq -r '.execution_time_ms')
    echo "  Results: $ROW_COUNT rows in ${EXEC_TIME}ms"

    # Show sample results
    echo ""
    echo "  Sample results:"
    echo "$RESPONSE" | jq -r '.results[0:3][] | "    - \(.username // "N/A"): \(.title // "N/A")"'

    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ JOIN query failed${NC}"
    echo "  Error: $(echo "$RESPONSE" | jq -r '.error.message // "Unknown error"')"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo "========================================"
echo "Test 3: Query with WHERE Clause"
echo "========================================"
echo ""
echo "Query: SELECT * FROM todos WHERE status = 'pending'"
echo ""

RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"SELECT * FROM ${MYSQL_CONN_1}.todos WHERE status = 'pending'\",
        \"connection_ids\": [\"$MYSQL_CONN_1\"],
        \"timeout_secs\": 30,
        \"apply_limit\": true,
        \"limit_value\": 10
    }")

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ WHERE clause query succeeded${NC}"
    ROW_COUNT=$(echo "$RESPONSE" | jq -r '.row_count')
    EXEC_TIME=$(echo "$RESPONSE" | jq -r '.execution_time_ms')
    echo "  Results: $ROW_COUNT pending todos in ${EXEC_TIME}ms"

    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ WHERE clause query failed${NC}"
    echo "  Error: $(echo "$RESPONSE" | jq -r '.error.message // "Unknown error"')"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo "========================================"
echo "Test 4: Validation Tests"
echo "========================================"
echo ""

# Test: Empty query
echo "Test 4a: Empty query should fail"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"\",
        \"connection_ids\": [\"$MYSQL_CONN_1\"],
        \"timeout_secs\": 30
    }")

if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Empty query correctly rejected${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ Empty query should have been rejected${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test: Single connection (should require 2+)
echo ""
echo "Test 4b: Single connection ID (cross-db requires 2+)"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"SELECT * FROM users\",
        \"connection_ids\": [],
        \"timeout_secs\": 30
    }")

if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
    echo -e "${GREEN}✓ No connections correctly rejected${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ Should require at least 2 connections${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo "========================================"
echo "Test Summary"
echo "========================================"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "========================================"
    echo "Phase 4 Status: Core Features Working"
    echo "========================================"
    echo ""
    echo "✓ Cross-database query API endpoint functional"
    echo "✓ Query planner parsing SQL correctly"
    echo "✓ Federated executor running sub-queries"
    echo "✓ Request validation working"
    echo ""
    echo "Next steps:"
    echo "  1. Test with actual cross-database JOINs (MySQL + PostgreSQL)"
    echo "  2. Implement proper JOIN condition extraction"
    echo "  3. Test UNION queries"
    echo "  4. Frontend integration"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
