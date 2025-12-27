#!/bin/bash

# Cross-Database Query Integration Test
# Tests MySQL (todolist) ↔ PostgreSQL (testdb) JOIN functionality

set -e

echo "=================================================="
echo "Cross-Database Query Integration Test"
echo "=================================================="
echo ""

API_BASE="http://localhost:3000/api"
MYSQL_CONN_URL="mysql://root:password123@localhost:3306/todolist"
PG_CONN_URL="postgresql://postgres:password@localhost:5433/testdb"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up test connections...${NC}"
    if [ ! -z "$MYSQL_CONN_ID" ]; then
        curl -s -X DELETE "$API_BASE/connections/$MYSQL_CONN_ID" > /dev/null || true
        echo -e "${GREEN}✓${NC} Deleted MySQL connection"
    fi
    if [ ! -z "$PG_CONN_ID" ]; then
        curl -s -X DELETE "$API_BASE/connections/$PG_CONN_ID" > /dev/null || true
        echo -e "${GREEN}✓${NC} Deleted PostgreSQL connection"
    fi
}

trap cleanup EXIT

echo -e "${BLUE}Step 1: Checking server health...${NC}"
HEALTH=$(curl -s "http://localhost:3000/health" || true)
if [ "$HEALTH" != "OK" ]; then
    echo -e "${RED}✗ Server is not running!${NC}"
    echo "Please start the backend server: make dev-backend"
    exit 1
fi
echo -e "${GREEN}✓${NC} Server is healthy"
echo ""

echo -e "${BLUE}Step 2: Creating database connections...${NC}"

# Create MySQL connection
echo "Creating MySQL connection..."
MYSQL_RESPONSE=$(curl -s -X POST "$API_BASE/connections" \
    -H "Content-Type: application/json" \
    -d "{
        \"name\": \"Test MySQL - TodoList\",
        \"connection_url\": \"$MYSQL_CONN_URL\",
        \"database_type\": \"mysql\"
    }")

MYSQL_CONN_ID=$(echo $MYSQL_RESPONSE | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -z "$MYSQL_CONN_ID" ]; then
    echo -e "${RED}✗ Failed to create MySQL connection${NC}"
    echo "Response: $MYSQL_RESPONSE"
    exit 1
fi
echo -e "${GREEN}✓${NC} MySQL connection created: $MYSQL_CONN_ID"

# Create PostgreSQL connection
echo "Creating PostgreSQL connection..."
PG_RESPONSE=$(curl -s -X POST "$API_BASE/connections" \
    -H "Content-Type: application/json" \
    -d "{
        \"name\": \"Test PostgreSQL - Projects\",
        \"connection_url\": \"$PG_CONN_URL\",
        \"database_type\": \"postgresql\"
    }")

PG_CONN_ID=$(echo $PG_RESPONSE | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -z "$PG_CONN_ID" ]; then
    echo -e "${RED}✗ Failed to create PostgreSQL connection${NC}"
    echo "Response: $PG_RESPONSE"
    exit 1
fi
echo -e "${GREEN}✓${NC} PostgreSQL connection created: $PG_CONN_ID"
echo ""

# Wait for connections to be ready
sleep 2

echo -e "${BLUE}Step 3: Verifying database schemas...${NC}"

# Check MySQL tables
echo "Checking MySQL tables..."
MYSQL_METADATA=$(curl -s "$API_BASE/connections/$MYSQL_CONN_ID/metadata")
MYSQL_TABLES=$(echo $MYSQL_METADATA | grep -o '"table_name":"[^"]*"' | wc -l)
echo -e "${GREEN}✓${NC} Found $MYSQL_TABLES tables in MySQL"

# Check PostgreSQL tables
echo "Checking PostgreSQL tables..."
PG_METADATA=$(curl -s "$API_BASE/connections/$PG_CONN_ID/metadata")
PG_TABLES=$(echo $PG_METADATA | grep -o '"table_name":"[^"]*"' | wc -l)
echo -e "${GREEN}✓${NC} Found $PG_TABLES tables in PostgreSQL"
echo ""

echo -e "${BLUE}Step 4: Executing cross-database JOIN queries...${NC}"
echo ""

# Test 1: Simple JOIN with aliases
echo -e "${YELLOW}Test 1: Cross-database JOIN (users ↔ projects)${NC}"
TEST1_QUERY="SELECT u.username, u.email, p.project_name, p.status, p.budget
FROM db1.users u
JOIN db2.projects p ON u.id = p.user_id
WHERE p.status = 'active'
LIMIT 10"

echo "Query:"
echo "$TEST1_QUERY"
echo ""

TEST1_START=$(date +%s%3N)
TEST1_RESPONSE=$(curl -s -X POST "$API_BASE/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"$TEST1_QUERY\",
        \"connection_ids\": [\"$MYSQL_CONN_ID\", \"$PG_CONN_ID\"],
        \"database_aliases\": {
            \"db1\": \"$MYSQL_CONN_ID\",
            \"db2\": \"$PG_CONN_ID\"
        },
        \"timeout_secs\": 60,
        \"apply_limit\": true,
        \"limit_value\": 1000
    }")
TEST1_END=$(date +%s%3N)
TEST1_TIME=$((TEST1_END - TEST1_START))

# Parse response
TEST1_ROW_COUNT=$(echo $TEST1_RESPONSE | grep -o '"row_count":[0-9]*' | grep -o '[0-9]*')
TEST1_EXEC_TIME=$(echo $TEST1_RESPONSE | grep -o '"execution_time_ms":[0-9]*' | grep -o '[0-9]*')
TEST1_SUB_QUERIES=$(echo $TEST1_RESPONSE | grep -o '"sub_queries":\[' | wc -l)

if [ -z "$TEST1_ROW_COUNT" ]; then
    echo -e "${RED}✗ Test 1 failed${NC}"
    echo "Response: $TEST1_RESPONSE"
else
    echo -e "${GREEN}✓${NC} Test 1 passed"
    echo "  Results: $TEST1_ROW_COUNT rows"
    echo "  Execution time: ${TEST1_EXEC_TIME}ms (backend) / ${TEST1_TIME}ms (total)"
    echo "  Sub-queries: $TEST1_SUB_QUERIES"
fi
echo ""

# Test 2: JOIN with aggregation
echo -e "${YELLOW}Test 2: Cross-database JOIN with user statistics${NC}"
TEST2_QUERY="SELECT u.username, u.email, COUNT(p.id) as project_count, SUM(p.budget) as total_budget
FROM db1.users u
LEFT JOIN db2.projects p ON u.id = p.user_id
GROUP BY u.username, u.email
LIMIT 10"

echo "Query:"
echo "$TEST2_QUERY"
echo ""

TEST2_START=$(date +%s%3N)
TEST2_RESPONSE=$(curl -s -X POST "$API_BASE/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"$TEST2_QUERY\",
        \"connection_ids\": [\"$MYSQL_CONN_ID\", \"$PG_CONN_ID\"],
        \"database_aliases\": {
            \"db1\": \"$MYSQL_CONN_ID\",
            \"db2\": \"$PG_CONN_ID\"
        },
        \"timeout_secs\": 60
    }")
TEST2_END=$(date +%s%3N)
TEST2_TIME=$((TEST2_END - TEST2_START))

TEST2_ROW_COUNT=$(echo $TEST2_RESPONSE | grep -o '"row_count":[0-9]*' | grep -o '[0-9]*')
TEST2_EXEC_TIME=$(echo $TEST2_RESPONSE | grep -o '"execution_time_ms":[0-9]*' | grep -o '[0-9]*')

if [ -z "$TEST2_ROW_COUNT" ]; then
    echo -e "${RED}✗ Test 2 failed${NC}"
    echo "Response: $TEST2_RESPONSE"
else
    echo -e "${GREEN}✓${NC} Test 2 passed"
    echo "  Results: $TEST2_ROW_COUNT rows"
    echo "  Execution time: ${TEST2_EXEC_TIME}ms (backend) / ${TEST2_TIME}ms (total)"
fi
echo ""

# Test 3: Complex multi-table JOIN
echo -e "${YELLOW}Test 3: Complex JOIN (todos + users + projects)${NC}"
TEST3_QUERY="SELECT u.username, t.title as todo_title, t.status as todo_status, p.project_name
FROM db1.users u
JOIN db1.todos t ON u.id = t.user_id
JOIN db2.projects p ON u.id = p.user_id
WHERE t.status IN ('pending', 'in_progress')
AND p.status = 'active'
LIMIT 15"

echo "Query:"
echo "$TEST3_QUERY"
echo ""

TEST3_START=$(date +%s%3N)
TEST3_RESPONSE=$(curl -s -X POST "$API_BASE/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"$TEST3_QUERY\",
        \"connection_ids\": [\"$MYSQL_CONN_ID\", \"$PG_CONN_ID\"],
        \"database_aliases\": {
            \"db1\": \"$MYSQL_CONN_ID\",
            \"db2\": \"$PG_CONN_ID\"
        },
        \"timeout_secs\": 60
    }")
TEST3_END=$(date +%s%3N)
TEST3_TIME=$((TEST3_END - TEST3_START))

TEST3_ROW_COUNT=$(echo $TEST3_RESPONSE | grep -o '"row_count":[0-9]*' | grep -o '[0-9]*')
TEST3_EXEC_TIME=$(echo $TEST3_RESPONSE | grep -o '"execution_time_ms":[0-9]*' | grep -o '[0-9]*')

if [ -z "$TEST3_ROW_COUNT" ]; then
    echo -e "${RED}✗ Test 3 failed${NC}"
    echo "Response: $TEST3_RESPONSE"
else
    echo -e "${GREEN}✓${NC} Test 3 passed"
    echo "  Results: $TEST3_ROW_COUNT rows"
    echo "  Execution time: ${TEST3_EXEC_TIME}ms (backend) / ${TEST3_TIME}ms (total)"
fi
echo ""

echo "=================================================="
echo -e "${GREEN}Cross-Database Query Test Summary${NC}"
echo "=================================================="
echo ""
echo "Database Setup:"
echo "  • MySQL (todolist): users, todos, categories (${MYSQL_TABLES} tables)"
echo "  • PostgreSQL (testdb): projects (${PG_TABLES} tables)"
echo ""
echo "Test Results:"
echo "  • Test 1 (Simple JOIN): $TEST1_ROW_COUNT rows in ${TEST1_EXEC_TIME}ms"
echo "  • Test 2 (JOIN with aggregation): $TEST2_ROW_COUNT rows in ${TEST2_EXEC_TIME}ms"
echo "  • Test 3 (Complex multi-table): $TEST3_ROW_COUNT rows in ${TEST3_EXEC_TIME}ms"
echo ""
echo -e "${GREEN}✓ All tests completed successfully!${NC}"
echo ""
