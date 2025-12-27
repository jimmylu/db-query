// Cross-Database Query Types

export interface CrossDatabaseQueryRequest {
  query: string;
  connection_ids: string[];
  database_aliases?: Record<string, string>;
  timeout_secs?: number;
  apply_limit?: boolean;
  limit_value?: number;
}

export interface SubQueryResult {
  connection_id: string;
  database_type: string;
  query: string;
  row_count: number;
  execution_time_ms: number;
}

export interface CrossDatabaseQueryResponse {
  original_query: string;
  sub_queries: SubQueryResult[];
  results: any[];
  row_count: number;
  execution_time_ms: number;
  limit_applied: boolean;
  executed_at: string;
}

export interface DatabaseAlias {
  alias: string;
  connectionId: string;
  connectionName?: string;
  databaseType?: string;
}

export interface CrossDatabaseQueryError {
  code: string;
  message: string;
}
