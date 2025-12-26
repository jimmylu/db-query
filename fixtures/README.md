# API Test Fixtures

This directory contains REST Client test files for testing the Database Query Tool API.

## Prerequisites

1. Install the [REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) extension in VS Code
2. Ensure the backend server is running on `http://localhost:3000`

## Usage

1. Open `test.rest` in VS Code
2. Update the variables at the top of the file:
   - `@connectionId`: Set this after creating a connection (copy from response)
   - `@testConnectionUrl`: Update with your actual database connection URL
   - `@testConnectionName`: Optional name for test connection

3. Click "Send Request" above any request to execute it

## File Structure

- `test.rest`: Complete API test suite with all endpoints

## Test Coverage

### Connections API
- ✅ List all connections
- ✅ Create new connection
- ✅ Get connection details
- ✅ Delete connection

### Metadata API
- ✅ Get cached metadata
- ✅ Force refresh metadata
- ✅ Use cached metadata

### Query API
- ✅ Simple SELECT queries
- ✅ Queries with WHERE, JOIN, aggregation
- ✅ Auto LIMIT appending (queries without LIMIT)
- ✅ Invalid SQL syntax handling
- ✅ Non-SELECT statement rejection

### Natural Language Query API
- ✅ Simple questions
- ✅ Complex queries with filters
- ✅ Aggregation queries
- ✅ Join queries

### Error Scenarios
- ✅ 404 errors (non-existent resources)
- ✅ 400 errors (invalid input)
- ✅ Missing required fields

## Workflow Example

The file includes a complete workflow example showing:
1. Create connection
2. Get metadata
3. Execute SQL query
4. Execute natural language query
5. Clean up (delete connection)

## Notes

- All requests use the `{{baseUrl}}` variable (default: `http://localhost:3000/api`)
- Connection IDs should be copied from create connection responses
- Test database URLs should be updated to match your environment
- Some requests are marked as error scenarios to test error handling

