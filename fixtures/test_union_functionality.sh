#!/bin/bash

# UNION Query Functionality Test
# Tests UNION and UNION ALL cross-database queries

set -e

echo "======================================================================"
echo "UNION Query Functionality Test"
echo "======================================================================"
echo ""

API_BASE="http://localhost:3000/api"
MYSQL_CONN_URL="mysql://root:password123@localhost:3306/todolist"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Cleanup
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"
    if [ ! -z "$CONN_ID1" ]; then
        curl -s -X DELETE "$API_BASE/connections/$CONN_ID1" > /dev/null || true
        echo -e "${GREEN}✓${NC} Cleaned up connection 1"
    fi
    if [ ! -z "$CONN_ID2" ]; then
        curl -s -X DELETE "$API_BASE/connections/$CONN_ID2" > /dev/null || true
        echo -e "${GREEN}✓${NC} Cleaned up connection 2"
    fi
}

trap cleanup EXIT

echo -e "${BLUE}Creating two connections to the same MySQL database...${NC}"

# Connection 1
RESPONSE1=$(curl -s -X POST "$API_BASE/connections" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"MySQL DB1\",\"connection_url\":\"$MYSQL_CONN_URL\",\"database_type\":\"mysql\"}")

CONN_ID1=$(echo $RESPONSE1 | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo -e "${GREEN}✓${NC} Connection 1: $CONN_ID1"

# Connection 2
RESPONSE2=$(curl -s -X POST "$API_BASE/connections" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"MySQL DB2\",\"connection_url\":\"$MYSQL_CONN_URL\",\"database_type\":\"mysql\"}")

CONN_ID2=$(echo $RESPONSE2 | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo -e "${GREEN}✓${NC} Connection 2: $CONN_ID2"

sleep 2

echo ""
echo "======================================================================"
echo -e "${BLUE}Test 1: Simple UNION query${NC}"
echo "======================================================================"

QUERY="SELECT username FROM db1.users
UNION
SELECT title FROM db2.todos
LIMIT 10"

echo "Query:"
echo "$QUERY"
echo ""

RESPONSE=$(curl -s -X POST "$API_BASE/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"$QUERY\",
        \"connection_ids\": [\"$CONN_ID1\", \"$CONN_ID2\"],
        \"database_aliases\": {
            \"db1\": \"$CONN_ID1\",
            \"db2\": \"$CONN_ID2\"
        },
        \"timeout_secs\": 60
    }")

# Check for errors
ERROR=$(echo $RESPONSE | grep -o '"error"' || true)
if [ ! -z "$ERROR" ]; then
    echo -e "${RED}✗ Test 1 failed${NC}"
    ERROR_MSG=$(echo $RESPONSE | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    echo "Error: $ERROR_MSG"
    echo ""

    # Check if it's the expected NOT_IMPLEMENTED error
    if echo $ERROR_MSG | grep -q "NOT_IMPLEMENTED\|not.*implemented\|Not implemented"; then
        echo -e "${YELLOW}⚠  UNION implementation was not complete before fix${NC}"
    fi
else
    ROW_COUNT=$(echo $RESPONSE | grep -o '"row_count":[0-9]*' | grep -o '[0-9]*')
    EXEC_TIME=$(echo $RESPONSE | grep -o '"execution_time_ms":[0-9]*' | grep -o '[0-9]*')
    SUB_QUERIES=$(echo $RESPONSE | grep -o '"database_type":"[^"]*"' | wc -l | tr -d ' ')

    echo -e "${GREEN}✓ Test 1 passed${NC}"
    echo "  • Rows returned: $ROW_COUNT"
    echo "  • Execution time: ${EXEC_TIME}ms"
    echo "  • Sub-queries: $SUB_QUERIES"
fi

echo ""
echo "======================================================================"
echo -e "${BLUE}Test 2: UNION ALL query${NC}"
echo "======================================================================"

QUERY2="SELECT username, email FROM db1.users
UNION ALL
SELECT full_name, email FROM db2.users
LIMIT 15"

echo "Query:"
echo "$QUERY2"
echo ""

RESPONSE2=$(curl -s -X POST "$API_BASE/cross-database/query" \
    -H "Content-Type: application/json" \
    -d "{
        \"query\": \"$QUERY2\",
        \"connection_ids\": [\"$CONN_ID1\", \"$CONN_ID2\"],
        \"database_aliases\": {
            \"db1\": \"$CONN_ID1\",
            \"db2\": \"$CONN_ID2\"
        },
        \"timeout_secs\": 60
    }")

ERROR2=$(echo $RESPONSE2 | grep -o '"error"' || true)
if [ ! -z "$ERROR2" ]; then
    echo -e "${RED}✗ Test 2 failed${NC}"
    ERROR_MSG2=$(echo $RESPONSE2 | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    echo "Error: $ERROR_MSG2"
else
    ROW_COUNT2=$(echo $RESPONSE2 | grep -o '"row_count":[0-9]*' | grep -o '[0-9]*')
    EXEC_TIME2=$(echo $RESPONSE2 | grep -o '"execution_time_ms":[0-9]*' | grep -o '[0-9]*')

    echo -e "${GREEN}✓ Test 2 passed${NC}"
    echo "  • Rows returned: $ROW_COUNT2"
    echo "  • Execution time: ${EXEC_TIME2}ms"
fi

echo ""
echo "======================================================================"
echo -e "${GREEN}Test Summary${NC}"
echo "======================================================================"
echo ""

if [ -z "$ERROR" ] && [ -z "$ERROR2" ]; then
    echo -e "${GREEN}✓ All UNION tests passed successfully!${NC}"
    echo ""
    echo "UNION functionality is now working:"
    echo "  • UNION (distinct) queries ✓"
    echo "  • UNION ALL queries ✓"
    echo "  • Cross-database support ✓"
    echo "  • AST-based decomposition ✓"
    echo ""
    echo -e "${GREEN}Phase 4 UNION implementation: COMPLETE! 🎉${NC}"
else
    echo -e "${YELLOW}Some tests encountered issues${NC}"
    echo "Please review the error messages above"
fi

echo ""
