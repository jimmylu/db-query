#!/bin/bash
# Comprehensive Unified SQL Test Script
# Tests both MySQL and PostgreSQL with unified SQL syntax

set -e

echo "========================================="
echo "Unified SQL Semantic Layer Test Suite"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BACKEND_URL="http://localhost:3000"
FRONTEND_URL="http://localhost:5173"

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Function to test endpoint
test_endpoint() {
    local test_name="$1"
    local url="$2"
    local method="${3:-GET}"
    local data="$4"

    echo -n "Testing: $test_name ... "

    if [ "$method" = "GET" ]; then
        response=$(curl -s "$url")
    else
        response=$(curl -s -X "$method" -H "Content-Type: application/json" -d "$data" "$url")
    fi

    if echo "$response" | jq -e . >/dev/null 2>&1; then
        if echo "$response" | grep -q "error"; then
            echo -e "${RED}FAILED${NC}"
            echo "  Error: $(echo "$response" | jq -r '.error.message')"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            return 1
        else
            echo -e "${GREEN}PASSED${NC}"
            TESTS_PASSED=$((TESTS_PASSED + 1))
            return 0
        fi
    else
        echo -e "${RED}FAILED${NC}"
        echo "  Invalid JSON response"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to run unified SQL test
test_unified_sql() {
    local test_name="$1"
    local conn_id="$2"
    local query="$3"
    local db_type="$4"

    echo ""
    echo "----------------------------------------"
    echo "Test: $test_name"
    echo "Database: $db_type"
    echo "Query: $query"
    echo "----------------------------------------"

    data="{\"query\":\"$query\",\"database_type\":\"$db_type\",\"timeout_secs\":30,\"apply_limit\":true,\"limit_value\":10}"

    response=$(curl -s -X POST "$BACKEND_URL/api/connections/$conn_id/unified-query" \
        -H "Content-Type: application/json" \
        -d "$data")

    if echo "$response" | jq -e . >/dev/null 2>&1; then
        if echo "$response" | grep -q "error"; then
            echo -e "${RED}FAILED${NC}"
            echo "Error: $(echo "$response" | jq -r '.error.message')"
            TESTS_FAILED=$((TESTS_FAILED + 1))
            return 1
        else
            echo -e "${GREEN}PASSED${NC}"

            # Display results
            original=$(echo "$response" | jq -r '.original_query')
            translated=$(echo "$response" | jq -r '.translated_query')
            row_count=$(echo "$response" | jq -r '.row_count')
            exec_time=$(echo "$response" | jq -r '.execution_time_ms')

            echo "  Original SQL: $original"
            if [ "$original" != "$translated" ]; then
                echo -e "  ${YELLOW}Translated SQL: $translated${NC}"
            fi
            echo "  Results: $row_count rows in ${exec_time}ms"

            TESTS_PASSED=$((TESTS_PASSED + 1))
            return 0
        fi
    else
        echo -e "${RED}FAILED${NC}"
        echo "Invalid JSON response"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

echo "Step 1: Checking backend health..."
test_endpoint "Backend Health" "$BACKEND_URL/health"

echo ""
echo "Step 2: Checking frontend..."
test_endpoint "Frontend Health" "$FRONTEND_URL"

echo ""
echo "Step 3: Getting MySQL connection ID..."
MYSQL_CONN_ID=$(curl -s "$BACKEND_URL/api/connections" | jq -r '.connections[] | select(.database_type == "mysql") | .id' | head -1)

if [ -z "$MYSQL_CONN_ID" ]; then
    echo -e "${RED}ERROR: No MySQL connection found${NC}"
    exit 1
fi

echo -e "${GREEN}MySQL Connection ID: $MYSQL_CONN_ID${NC}"

echo ""
echo "========================================="
echo "MySQL Unified SQL Tests"
echo "========================================="

# Test 1: Basic SELECT
test_unified_sql \
    "Basic SELECT" \
    "$MYSQL_CONN_ID" \
    "SELECT id, username, email FROM users" \
    "mysql"

# Test 2: DataFusion CURRENT_DATE
test_unified_sql \
    "DataFusion CURRENT_DATE → MySQL CURDATE()" \
    "$MYSQL_CONN_ID" \
    "SELECT id, title, due_date FROM todos WHERE due_date >= CURRENT_DATE - INTERVAL '7' DAY" \
    "mysql"

# Test 3: Aggregation
test_unified_sql \
    "GROUP BY Aggregation" \
    "$MYSQL_CONN_ID" \
    "SELECT status, COUNT(*) as total FROM todos GROUP BY status ORDER BY total DESC" \
    "mysql"

# Test 4: JOIN
test_unified_sql \
    "Multi-table JOIN" \
    "$MYSQL_CONN_ID" \
    "SELECT u.username, t.title, t.status FROM users u JOIN todos t ON u.id = t.user_id WHERE t.status = 'pending'" \
    "mysql"

# Test 5: Complex WHERE with OR
test_unified_sql \
    "Complex WHERE clause" \
    "$MYSQL_CONN_ID" \
    "SELECT title, priority, status FROM todos WHERE priority = 'high' OR priority = 'urgent'" \
    "mysql"

echo ""
echo "========================================="
echo "PostgreSQL Tests (Optional)"
echo "========================================="

PG_CONN_ID=$(curl -s "$BACKEND_URL/api/connections" | jq -r '.connections[] | select(.database_type == "postgresql") | .id' | head -1)

if [ -n "$PG_CONN_ID" ]; then
    echo -e "${GREEN}PostgreSQL Connection ID: $PG_CONN_ID${NC}"

    test_unified_sql \
        "PostgreSQL Basic SELECT" \
        "$PG_CONN_ID" \
        "SELECT id, title FROM tickets" \
        "postgresql"
else
    echo -e "${YELLOW}PostgreSQL connection not available - skipping tests${NC}"
fi

echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
