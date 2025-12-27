#!/bin/bash
# Simple test for cross-database query API

BACKEND_URL="http://localhost:3000"
MYSQL_CONN_ID="1bb2bc4c-b575-49c2-a382-6032a3abe23e"

echo "Test 1: Single database query (unqualified table)"
echo "==================================================="

RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary '{
    "query": "SELECT * FROM users",
    "connection_ids": ["'$MYSQL_CONN_ID'"],
    "timeout_secs": 30,
    "apply_limit": true,
    "limit_value": 5
  }')

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Query succeeded!"
    echo "  Rows: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
    echo ""
    echo "Sub-query executed:"
    echo "  $(echo "$RESPONSE" | jq -r '.sub_queries[0].query')"
    echo ""
    echo "First result:"
    echo "$RESPONSE" | jq -r '.results[0]'
else
    echo "✗ Query failed"
    echo "$RESPONSE" | jq -r '.error.message // "Unknown error"'
fi
