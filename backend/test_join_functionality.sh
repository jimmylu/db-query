#!/bin/bash
# Test cross-database JOIN functionality

BACKEND_URL="http://localhost:3000"
MYSQL_CONN_ID="1bb2bc4c-b575-49c2-a382-6032a3abe23e"

echo "=================================="
echo "Cross-Database JOIN Tests"
echo "=================================="
echo ""

echo "Test 1: Simple JOIN (users JOIN todos)"
echo "----------------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT u.username, t.title FROM db1.users u JOIN db2.todos t ON u.id = t.user_id LIMIT 5",
  "connection_ids": ["$MYSQL_CONN_ID", "$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID",
    "db2": "$MYSQL_CONN_ID"
  },
  "timeout_secs": 60,
  "apply_limit": false
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 1 PASSED: JOIN query succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Execution time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
    echo "  Sub-queries executed: $(echo "$RESPONSE" | jq -r '.sub_queries | length')"
    echo ""
    echo "Sub-query 1:"
    echo "  $(echo "$RESPONSE" | jq -r '.sub_queries[0].query')"
    echo "  Rows: $(echo "$RESPONSE" | jq -r '.sub_queries[0].row_count')"
    echo "  Time: $(echo "$RESPONSE" | jq -r '.sub_queries[0].execution_time_ms')ms"
    echo ""
    echo "Sub-query 2:"
    echo "  $(echo "$RESPONSE" | jq -r '.sub_queries[1].query')"
    echo "  Rows: $(echo "$RESPONSE" | jq -r '.sub_queries[1].row_count')"
    echo "  Time: $(echo "$RESPONSE" | jq -r '.sub_queries[1].execution_time_ms')ms"
    echo ""
    echo "First joined result:"
    echo "$RESPONSE" | jq -r '.results[0]'
else
    echo "✗ Test 1 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 2: JOIN with WHERE clause"
echo "--------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT u.username, t.title, t.status FROM db1.users u JOIN db2.todos t ON u.id = t.user_id WHERE t.status = 'pending' LIMIT 3",
  "connection_ids": ["$MYSQL_CONN_ID", "$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID",
    "db2": "$MYSQL_CONN_ID"
  }
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 2 PASSED: JOIN with WHERE succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Execution time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
else
    echo "✗ Test 2 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 3: Multi-column JOIN"
echo "-------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT u.username, u.email, t.title, t.priority FROM db1.users u JOIN db2.todos t ON u.id = t.user_id LIMIT 5",
  "connection_ids": ["$MYSQL_CONN_ID", "$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID",
    "db2": "$MYSQL_CONN_ID"
  }
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 3 PASSED: Multi-column JOIN succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo ""
    echo "Sample joined result:"
    echo "$RESPONSE" | jq -r '.results[0]'
else
    echo "✗ Test 3 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "=================================="
echo "JOIN Test Summary"
echo "=================================="
echo "Note: These tests use the same MySQL database with aliases to simulate"
echo "cross-database JOINs. The JOIN functionality is working correctly if"
echo "the results show properly merged data from users and todos tables."
echo "=================================="
