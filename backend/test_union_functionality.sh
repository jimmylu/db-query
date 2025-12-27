#!/bin/bash
# Test cross-database UNION functionality

BACKEND_URL="http://localhost:3000"
MYSQL_CONN_ID="1bb2bc4c-b575-49c2-a382-6032a3abe23e"

echo "=================================="
echo "Cross-Database UNION Tests"
echo "=================================="
echo ""

echo "Test 1: Simple UNION (users UNION todos usernames)"
echo "---------------------------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT username FROM db1.users UNION SELECT title FROM db2.todos LIMIT 10",
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
    echo "✓ Test 1 PASSED: UNION query succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Execution time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
    echo "  Sub-queries executed: $(echo "$RESPONSE" | jq -r '.sub_queries | length')"
    echo ""
    echo "Sub-query 1:"
    echo "  $(echo "$RESPONSE" | jq -r '.sub_queries[0].query')"
    echo "  Rows: $(echo "$RESPONSE" | jq -r '.sub_queries[0].row_count')"
    echo ""
    echo "Sub-query 2:"
    echo "  $(echo "$RESPONSE" | jq -r '.sub_queries[1].query')"
    echo "  Rows: $(echo "$RESPONSE" | jq -r '.sub_queries[1].row_count')"
    echo ""
    echo "First 3 results:"
    echo "$RESPONSE" | jq -r '.results[0:3]'
else
    echo "✗ Test 1 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 2: UNION ALL (preserve duplicates)"
echo "---------------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT username FROM db1.users UNION ALL SELECT title FROM db2.todos LIMIT 20",
  "connection_ids": ["$MYSQL_CONN_ID", "$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID",
    "db2": "$MYSQL_CONN_ID"
  }
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 2 PASSED: UNION ALL query succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Execution time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
else
    echo "✗ Test 2 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 3: UNION with WHERE clause"
echo "-------------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT username FROM db1.users WHERE id > 2 UNION SELECT title FROM db2.todos WHERE status = 'completed'",
  "connection_ids": ["$MYSQL_CONN_ID", "$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID",
    "db2": "$MYSQL_CONN_ID"
  }
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 3 PASSED: UNION with WHERE succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo "  Execution time: $(echo "$RESPONSE" | jq -r '.execution_time_ms')ms"
else
    echo "✗ Test 3 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "Test 4: Multi-column UNION"
echo "-------------------------"
RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/cross-database/query" \
  -H "Content-Type: application/json" \
  --data-binary @- <<EOF
{
  "query": "SELECT id, username AS name FROM db1.users UNION SELECT id, title AS name FROM db2.todos LIMIT 15",
  "connection_ids": ["$MYSQL_CONN_ID", "$MYSQL_CONN_ID"],
  "database_aliases": {
    "db1": "$MYSQL_CONN_ID",
    "db2": "$MYSQL_CONN_ID"
  }
}
EOF
)

if echo "$RESPONSE" | jq -e '.results' >/dev/null 2>&1; then
    echo "✓ Test 4 PASSED: Multi-column UNION succeeded!"
    echo "  Rows returned: $(echo "$RESPONSE" | jq -r '.row_count')"
    echo ""
    echo "Sample result:"
    echo "$RESPONSE" | jq -r '.results[0]'
else
    echo "✗ Test 4 FAILED"
    echo "$RESPONSE" | jq '.'
fi

echo ""
echo "=================================="
echo "UNION Test Summary"
echo "=================================="
echo "Note: These tests use the same MySQL database with aliases to simulate"
echo "cross-database UNION operations. The UNION functionality is working correctly if"
echo "the results show properly merged data from users and todos tables."
echo "=================================="
