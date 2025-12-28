export interface DatabaseConnection {
  id: string;
  name?: string;
  connection_url: string;
  database_type: string;
  domain_id: string;
  status: 'connected' | 'disconnected' | 'error';
  created_at: string;
  last_connected_at?: string;
  metadata_cache_id?: string;
}

export interface CreateConnectionRequest {
  name?: string;
  connection_url: string;
  database_type?: string;
  domain_id?: string;
}

export interface DatabaseMetadata {
  id: string;
  connection_id: string;
  tables: Table[];
  views: View[];
  schemas: string[];
  metadata_json: string;
  retrieved_at: string;
  version: number;
}

export interface Table {
  name: string;
  schema?: string;
  columns: Column[];
  row_count?: number;
  description?: string;
}

export interface View {
  name: string;
  schema?: string;
  columns: Column[];
  definition?: string;
  description?: string;
}

export interface Column {
  name: string;
  data_type: string;
  is_nullable: boolean;
  is_primary_key: boolean;
  is_foreign_key: boolean;
  default_value?: string;
  max_length?: number;
  description?: string;
}

export interface QueryResult {
  id: string;
  connection_id: string;
  query_text: string;
  is_llm_generated: boolean;
  status: 'pending' | 'executing' | 'completed' | 'failed';
  results?: any[];
  row_count?: number;
  execution_time_ms?: number;
  error_message?: string;
  executed_at?: string;
  limit_applied: boolean;
}

// Domain types
export interface Domain {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface DomainResponse {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
  connection_count: number;
  saved_query_count: number;
  query_history_count: number;
}

export interface CreateDomainRequest {
  name: string;
  description?: string;
}

export interface UpdateDomainRequest {
  name?: string;
  description?: string;
}

// Saved Query types
export interface SavedQuery {
  id: string;
  domain_id: string;
  connection_id: string;
  name: string;
  query_text: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateSavedQueryRequest {
  connection_id: string;
  name: string;
  query_text: string;
  description?: string;
}

export interface UpdateSavedQueryRequest {
  name?: string;
  query_text?: string;
  description?: string;
}

// Query History types
export interface QueryHistory {
  id: string;
  domain_id: string;
  connection_id: string;
  query_text: string;
  row_count: number;
  execution_time_ms: number;
  status: 'success' | 'failed';
  error_message?: string;
  executed_at: string;
  is_llm_generated: boolean;
}

// Re-export cross-database types
export type {
  CrossDatabaseQueryRequest,
  CrossDatabaseQueryResponse,
  SubQueryResult,
  DatabaseAlias,
  CrossDatabaseQueryError,
} from './cross-database';

