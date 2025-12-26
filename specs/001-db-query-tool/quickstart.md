# Quick Start Guide: Database Query Tool

**Date**: 2024-12-22  
**Feature**: Database Query Tool

## Overview

This guide provides step-by-step instructions for getting started with the Database Query Tool. It covers installation, setup, and basic usage scenarios.

## Prerequisites

### Backend Requirements

- Rust (latest stable version)
- Cargo (Rust package manager)
- SQLite3 (for metadata storage)
- Access to LLM service (via rig.rs)

### Frontend Requirements

- Node.js 18+ and npm/yarn
- Modern web browser (Chrome, Firefox, Safari, Edge)

### Database Requirements

- PostgreSQL database (for initial support)
- Valid database connection URL

## Installation

### Backend Setup

1. **Clone the repository** (if not already done):
   ```bash
   git clone <repository-url>
   cd db_query
   ```

2. **Navigate to backend directory**:
   ```bash
   cd backend
   ```

3. **Install dependencies**:
   ```bash
   cargo build
   ```

4. **Configure environment variables**:
   Create a `.env` file in the backend directory:
   ```env
   DATABASE_URL=sqlite:./metadata.db
   LLM_GATEWAY_URL=http://localhost:8080
   PORT=3000
   ```

5. **Run the backend server**:
   ```bash
   cargo run
   ```

   The backend should start on `http://localhost:3000`

### Frontend Setup

1. **Navigate to frontend directory**:
   ```bash
   cd frontend
   ```

2. **Install dependencies**:
   ```bash
   npm install
   # or
   yarn install
   ```

3. **Configure API endpoint**:
   Create a `.env` file in the frontend directory:
   ```env
   REACT_APP_API_URL=http://localhost:3000/api
   ```

4. **Start the development server**:
   ```bash
   npm start
   # or
   yarn start
   ```

   The frontend should open in your browser at `http://localhost:3000`

## Basic Usage

### Scenario 1: Connect to a Database

1. **Open the application** in your web browser
2. **Click "Add Connection"** or navigate to the connections page
3. **Enter connection details**:
   - **Name** (optional): Give your connection a friendly name (e.g., "Production DB")
   - **Connection URL**: Enter your PostgreSQL connection string
     ```
     postgresql://username:password@hostname:5432/database_name
     ```
   - **Database Type**: Select "PostgreSQL" (default)
4. **Click "Connect"**
5. **Wait for connection**: The system will:
   - Establish connection to your database
   - Retrieve metadata (tables, views, schemas)
   - Convert metadata to JSON format using LLM
   - Cache metadata in local SQLite storage
6. **View metadata**: Once connected, you'll see:
   - List of tables with their columns
   - List of views with their columns
   - Schema information

### Scenario 2: Execute a SQL Query

1. **Select a connection** from your connections list
2. **Navigate to the Query page**
3. **Enter a SQL query** in the Monaco Editor:
   ```sql
   SELECT * FROM users LIMIT 10;
   ```
4. **Click "Execute Query"**
5. **View results**: The query results will be displayed in a table format below the editor

**Note**: 
- Only SELECT statements are allowed
- If you omit LIMIT, the system automatically adds `LIMIT 1000`
- Invalid SQL will show an error message

### Scenario 3: Query Using Natural Language

1. **Select a connected database**
2. **Navigate to the Natural Language Query section**
3. **Enter your question** in plain language:
   ```
   Show me all users who registered in the last month
   ```
4. **Click "Generate Query"**
5. **Review the generated SQL**: The system will:
   - Use LLM to generate SQL based on your question and database metadata
   - Display the generated SQL query
   - Execute the query automatically
6. **View results**: Results are displayed in the same table format

**Note**: 
- The generated query is validated before execution
- If generation fails, you'll see an error message with suggestions

### Scenario 4: Refresh Metadata

If your database schema has changed:

1. **Select the connection**
2. **Navigate to Metadata section**
3. **Click "Refresh Metadata"**
4. **Wait for refresh**: The system will:
   - Reconnect to the database
   - Retrieve updated metadata
   - Update the cache

## API Usage Examples

### Create a Connection

```bash
curl -X POST http://localhost:3000/api/connections \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Database",
    "connection_url": "postgresql://user:pass@localhost:5432/mydb",
    "database_type": "postgresql"
  }'
```

### Get Metadata

```bash
curl http://localhost:3000/api/connections/{connectionId}/metadata
```

### Execute SQL Query

```bash
curl -X POST http://localhost:3000/api/connections/{connectionId}/query \
  -H "Content-Type: application/json" \
  -d '{
    "query": "SELECT * FROM users LIMIT 10"
  }'
```

### Execute Natural Language Query

```bash
curl -X POST http://localhost:3000/api/connections/{connectionId}/nl-query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "Show me all active users"
  }'
```

## Common Tasks

### View Connection Status

- Green indicator: Connection is active
- Yellow indicator: Connection is disconnected
- Red indicator: Connection error

### Handle Query Errors

**Invalid SQL Syntax**:
- Error message will indicate the specific syntax issue
- Review your SQL query and fix the syntax error
- Try again

**Non-SELECT Statement**:
- Error: "Only SELECT statements are permitted"
- Solution: Rewrite your query as a SELECT statement

**Connection Error**:
- Check your connection URL
- Verify database is accessible
- Check network connectivity

### Optimize Query Performance

- Always include LIMIT clause in your queries
- Use specific column names instead of SELECT *
- Add WHERE clauses to filter data
- Use indexes (if available in your database)

## Troubleshooting

### Backend Won't Start

**Issue**: Port 3000 already in use
**Solution**: Change PORT in `.env` file or stop the process using port 3000

**Issue**: SQLite database error
**Solution**: Ensure write permissions in backend directory

**Issue**: LLM gateway connection failed
**Solution**: Verify LLM_GATEWAY_URL is correct and service is running

### Frontend Won't Connect to Backend

**Issue**: CORS errors
**Solution**: Ensure backend CORS is configured to allow frontend origin

**Issue**: API calls failing
**Solution**: Verify REACT_APP_API_URL matches backend URL

### Database Connection Fails

**Issue**: Invalid connection URL
**Solution**: Verify URL format: `postgresql://user:pass@host:port/dbname`

**Issue**: Authentication failed
**Solution**: Check username and password in connection URL

**Issue**: Database unreachable
**Solution**: Verify network connectivity and firewall settings

### Query Execution Fails

**Issue**: SQL syntax error
**Solution**: Review error message for specific syntax issue

**Issue**: Table not found
**Solution**: Verify table name and schema are correct

**Issue**: Permission denied
**Solution**: Ensure database user has SELECT permissions

## Next Steps

- Explore advanced query features
- Set up multiple database connections
- Review query history (if implemented)
- Customize metadata refresh settings
- Integrate with your workflow

## Support

For issues or questions:
- Check the documentation in `/specs/001-db-query-tool/`
- Review API documentation at `/contracts/openapi.yaml`
- Check error messages for specific guidance

