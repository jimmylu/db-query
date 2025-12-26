export interface DatabaseConnection {
  id: string;
  name?: string;
  connection_url: string;
  database_type: string;
  status: 'connected' | 'disconnected' | 'error';
  created_at: string;
  last_connected_at?: string;
  metadata_cache_id?: string;
}

export interface CreateConnectionRequest {
  name?: string;
  connection_url: string;
  database_type?: string;
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

