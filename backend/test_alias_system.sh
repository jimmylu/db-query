#!/bin/bash
# Test database alias system

BACKEND_URL="http://localhost:3000"
MYSQL_CONN_ID="1bb2bc4c-b575-49c2-a382-6032a3abe23e"

echo "================================"
echo "Database Alias System Tests"
echo "================================"
echo ""

echo "Test 1: Query with alias 'db1' -> MySQL connection"
echo "---------------------------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT * FROM db1.users",
  "connection_ids": ["$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID"
  },
  "timeout_secs": 30,
  "apply_limit": true,
  "limit_value": 5
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 1 PASSED: Alias system working!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Execution time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
    echo "  Sub-query: $(echo "$RESPONSE" | jq -r '.sub_queries[0].query')"
else
    echo "✗ Test 1 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 2: Unqualified table (fallback to first connection)"
echo "---------------------------------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT * FROM users LIMIT 3",
  "connection_ids": ["$MYSQL_CONN_ID"],
  "timeout_secs": 30,
  "apply_limit": false
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 2 PASSED: Unqualified table works!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Sub-query: $(echo "$RESPONSE" | jq -r '.sub_queries[0].query')"
else
    echo "✗ Test 2 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 3: Invalid alias (should fail)"
echo "------------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT * FROM unknown_alias.users",
  "connection_ids": ["$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID"
  },
  "timeout_secs": 30
}
EOF
)

if echo "$RESPONSE" | jq -e '.error' >/dev/null 2>&1; then
    echo "✓ Test 3 PASSED: Invalid alias properly rejected!"
    echo "  Error message: $(echo "$RESPONSE" | jq -r '.error.message')"
else
    echo "✗ Test 3 FAILED: Should have returned an error"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "================================"
echo "Alias System Test Summary"
echo "================================"
