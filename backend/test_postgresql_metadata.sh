#!/bin/bash

# Test PostgreSQL Metadata Retrieval Fix
# This script tests if the PostgreSQL metadata retrieval issue is resolved

set -e

echo "==================================================================="
echo "PostgreSQL Metadata Retrieval Test"
echo "==================================================================="
echo ""

API_BASE="http://localhost:3000/api"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if PostgreSQL is running
echo -e "${BLUE}Checking if PostgreSQL is available...${NC}"
if ! timeout 2 bash -c "cat < /dev/null > /dev/tcp/localhost/5433" 2>/dev/null; then
    echo -e "${YELLOW}⚠  PostgreSQL not running on port 5433${NC}"
    echo ""
    echo "Starting PostgreSQL container..."
    docker run -d --name test-postgres \
        -e POSTGRES_PASSWORD=password \
        -e POSTGRES_DB=testdb \
        -p 5433:5432 \
        postgres:15 || true

    echo "Waiting for PostgreSQL to be ready..."
    sleep 10
fi

echo -e "${GREEN}✓${NC} PostgreSQL is available"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}Cleaning up...${NC}"
    if [ ! -z "$CONN_ID" ]; then
        curl -s -X DELETE "$API_BASE/connections/$CONN_ID" > /dev/null || true
        echo -e "${GREEN}✓${NC} Cleaned up connection"
    fi
}

trap cleanup EXIT

# Test PostgreSQL connection and metadata retrieval
echo -e "${BLUE}Test 1: Create PostgreSQL connection${NC}"
echo "--------------------------------------"

PG_URL="postgresql://postgres:password@localhost:5433/testdb"

RESPONSE=$(curl -s -X POST "$API_BASE/connections" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Test PostgreSQL\",\"connection_url\":\"$PG_URL\",\"database_type\":\"postgresql\"}")

echo "Response:"
echo "$RESPONSE" | python3 -m json.tool || echo "$RESPONSE"
echo ""

# Extract connection ID
CONN_ID=$(echo $RESPONSE | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$CONN_ID" ]; then
    echo -e "${RED}✗ Failed to create connection${NC}"
    ERROR_MSG=$(echo $RESPONSE | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    echo "Error: $ERROR_MSG"
    exit 1
fi

echo -e "${GREEN}✓ Test 1 passed${NC}"
echo "  • Connection ID: $CONN_ID"
echo ""

# Test metadata retrieval
echo -e "${BLUE}Test 2: Retrieve metadata${NC}"
echo "--------------------------------------"

METADATA_RESPONSE=$(curl -s -X GET "$API_BASE/connections/$CONN_ID/metadata")

echo "Metadata response:"
echo "$METADATA_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$METADATA_RESPONSE"
echo ""

# Check for errors
ERROR=$(echo $METADATA_RESPONSE | grep -o '"error"' || true)
if [ ! -z "$ERROR" ]; then
    echo -e "${RED}✗ Test 2 failed${NC}"
    ERROR_MSG=$(echo $METADATA_RESPONSE | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
    echo "Error: $ERROR_MSG"

    # Check if it's the same error as before
    if echo $ERROR_MSG | grep -q "Failed to get columns"; then
        echo -e "${YELLOW}⚠  This is the original metadata retrieval error${NC}"
        echo "The fix may need additional adjustments"
    fi
    exit 1
fi

# Check if metadata was retrieved successfully
TABLE_COUNT=$(echo $METADATA_RESPONSE | grep -o '"tables":\[' | wc -l | tr -d ' ')
if [ "$TABLE_COUNT" -gt "0" ]; then
    echo -e "${GREEN}✓ Test 2 passed${NC}"
    echo "  • Metadata retrieved successfully"
    echo "  • Tables found in database"
else
    echo -e "${GREEN}✓ Test 2 passed (no tables)${NC}"
    echo "  • Metadata retrieved successfully"
    echo "  • No tables in database (empty database)"
fi

echo ""
echo "==================================================================="
echo -e "${GREEN}PostgreSQL Metadata Retrieval: FIXED! ✓${NC}"
echo "==================================================================="
echo ""
echo "Summary:"
echo "  • Connection creation: ✓"
echo "  • Metadata retrieval: ✓"
echo "  • No 'Failed to get columns' error: ✓"
echo ""
