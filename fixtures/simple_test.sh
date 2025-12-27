#!/bin/bash

# Simple Cross-Database Query Test - MySQL only
# Tests self-join capability on MySQL todolist database

set -e

echo "======================================================================"
echo "Simple Cross-Database Query Test (MySQL Self-Join)"
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
echo -e "${BLUE}Executing cross-database query (users JOIN todos)...${NC}"

QUERY="SELECT u.username, u.email, t.title, t.status, t.priority
FROM db1.users u
JOIN db2.todos t ON u.id = t.user_id
WHERE t.status IN ('pending', 'in_progress')
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

# Parse results
ROW_COUNT=$(echo $RESPONSE | grep -o '"row_count":[0-9]*' | grep -o '[0-9]*')
EXEC_TIME=$(echo $RESPONSE | grep -o '"execution_time_ms":[0-9]*' | grep -o '[0-9]*')
SUB_QUERIES=$(echo $RESPONSE | grep -o '"database_type":"[^"]*"' | wc -l | tr -d ' ')

echo -e "${GREEN}✓${NC} Query executed successfully!"
echo ""
echo "Results:"
echo "  • Rows returned: $ROW_COUNT"
echo "  • Execution time: ${EXEC_TIME}ms"
echo "  • Sub-queries executed: $SUB_QUERIES"
echo ""

# Show first few results
echo "Sample results:"
echo $RESPONSE | grep -o '"username":"[^"]*"' | head -3 | sed 's/"username":"/ • User: /' | sed 's/"$//'

echo ""
echo -e "${GREEN}======================================================================"
echo "✓ Test completed successfully!"
echo "======================================================================${NC}"
